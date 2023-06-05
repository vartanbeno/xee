use miette::Diagnostic;
use std::fmt;

use xee_xpath::{
    Atomic, DynamicContext, Error, Namespaces, Sequence, StaticContext, Value, ValueError, XPath,
};
use xot::Xot;

use crate::qt;
use crate::serialize::serialize;

#[derive(Debug, PartialEq)]
pub(crate) enum UnexpectedError {
    Code(String),
    Error(Error),
}

#[derive(Debug, PartialEq)]
pub(crate) enum TestOutcome<'a> {
    Passed,
    PassedWithUnexpectedError(UnexpectedError),
    Failed(Failure<'a>),
    RuntimeError(Error),
    CompilationError(Error),
    UnsupportedExpression(Error),
    Unsupported,
    EnvironmentError(String),
}

pub(crate) trait Assertable {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Value, Error>) -> TestOutcome {
        match result {
            Ok(value) => self.assert_value(xot, value.clone()),
            Err(error) => TestOutcome::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, xot: &mut Xot, value: Value) -> TestOutcome;
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertAnyOf(Vec<TestCaseResult>);

impl AssertAnyOf {
    pub(crate) fn new(test_case_results: Vec<TestCaseResult>) -> Self {
        Self(test_case_results)
    }
}

impl Assertable for AssertAnyOf {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Value, Error>) -> TestOutcome {
        let mut failed_test_results = Vec::new();
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(xot, result);
            match result {
                TestOutcome::Passed | TestOutcome::PassedWithUnexpectedError(..) => return result,
                _ => failed_test_results.push(result),
            }
        }
        match result {
            Ok(_value) => TestOutcome::Failed(Failure::AnyOf(self, failed_test_results)),
            Err(error) => TestOutcome::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, _xot: &mut Xot, _value: Value) -> TestOutcome {
        unreachable!();
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertAllOf(Vec<TestCaseResult>);

impl AssertAllOf {
    pub(crate) fn new(test_case_results: Vec<TestCaseResult>) -> Self {
        Self(test_case_results)
    }
}

impl Assertable for AssertAllOf {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Value, Error>) -> TestOutcome {
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(xot, result);
            match result {
                TestOutcome::Passed | TestOutcome::PassedWithUnexpectedError(..) => {}
                _ => return result,
            }
        }
        TestOutcome::Passed
    }

    fn assert_value(&self, _xot: &mut Xot, _value: Value) -> TestOutcome {
        unreachable!();
    }
}

#[derive(PartialEq)]
pub(crate) struct AssertNot(Box<TestCaseResult>);

impl AssertNot {
    pub(crate) fn new(test_case_result: TestCaseResult) -> Self {
        Self(Box::new(test_case_result))
    }
}

impl fmt::Debug for AssertNot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AssertNot({:?})", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Assert(qt::XPathExpr);

#[derive(Debug, PartialEq)]
pub(crate) struct AssertEq(qt::XPathExpr);

impl AssertEq {
    pub(crate) fn new(expr: qt::XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for AssertEq {
    fn assert_value(&self, _xot: &mut Xot, value: Value) -> TestOutcome {
        let expected_value = run_xpath(&self.0);

        match expected_value {
            Ok(expected_value) => {
                if expected_value == value {
                    TestOutcome::Passed
                } else {
                    TestOutcome::Failed(Failure::Eq(self, value))
                }
            }
            Err(error) => TestOutcome::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertCount(usize);

impl AssertCount {
    pub(crate) fn new(count: usize) -> Self {
        Self(count)
    }
}

impl Assertable for AssertCount {
    fn assert_value(&self, _xot: &mut Xot, value: Value) -> TestOutcome {
        let sequence: Result<Sequence, ValueError> = (&value).try_into();
        if let Ok(sequence) = sequence {
            let found_len = sequence.borrow().len();
            if found_len == self.0 {
                TestOutcome::Passed
            } else {
                TestOutcome::Failed(Failure::Count(
                    self,
                    AssertCountFailure::WrongCount(found_len),
                ))
            }
        } else {
            TestOutcome::Failed(Failure::Count(self, AssertCountFailure::WrongValue(value)))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertDeepEq(qt::XPathExpr);

#[derive(Debug, PartialEq)]
pub(crate) struct AssertPermutation(qt::XPathExpr);

#[derive(Debug, PartialEq)]
pub(crate) struct AssertXml(String);

impl AssertXml {
    pub(crate) fn new(xml: String) -> Self {
        Self(xml)
    }
}

impl Assertable for AssertXml {
    fn assert_value(&self, xot: &mut Xot, value: Value) -> TestOutcome {
        let xml = serialize(xot, &value);

        let xml = if let Ok(xml) = xml {
            xml
        } else {
            return TestOutcome::Failed(Failure::Xml(self, AssertXmlFailure::WrongValue(value)));
        };
        // also wrap expected XML in a sequence element
        let expected_xml = format!("<sequence>{}</sequence>", self.0);

        // now parse both with Xot
        let found = xot.parse(&xml).unwrap();
        let expected = xot.parse(&expected_xml).unwrap();

        // and compare
        let c = xot.compare(expected, found);

        // clean up
        xot.remove(found).unwrap();
        xot.remove(expected).unwrap();

        if c {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::Xml(self, AssertXmlFailure::WrongXml(xml)))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertEmpty;

#[derive(Debug, PartialEq)]
pub(crate) struct AssertSerializationMatches;

#[derive(Debug, PartialEq)]
pub(crate) struct AssertSerializationError(String);

#[derive(Debug, PartialEq)]
pub(crate) struct AssertType(String);

#[derive(Debug, PartialEq)]
pub(crate) struct AssertTrue;

impl AssertTrue {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Assertable for AssertTrue {
    fn assert_value(&self, _xot: &mut Xot, value: Value) -> TestOutcome {
        if matches!(value, Value::Atomic(Atomic::Boolean(true))) {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::True(self, value))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertFalse;

impl AssertFalse {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Assertable for AssertFalse {
    fn assert_value(&self, _xot: &mut Xot, value: Value) -> TestOutcome {
        if matches!(value, Value::Atomic(Atomic::Boolean(false))) {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::False(self, value))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertStringValue(String);

impl AssertStringValue {
    pub(crate) fn new(string: String) -> Self {
        Self(string)
    }
}

impl Assertable for AssertStringValue {
    fn assert_value(&self, xot: &mut Xot, value: Value) -> TestOutcome {
        let seq: Result<Sequence, ValueError> = (&value).try_into();
        match seq {
            Ok(seq) => {
                let strings = seq
                    .borrow()
                    .as_slice()
                    .iter()
                    .map(|item| item.string_value(xot))
                    .collect::<Result<Vec<_>, _>>();
                match strings {
                    Ok(strings) => {
                        let joined = strings.join(" ");
                        if joined == self.0 {
                            TestOutcome::Passed
                        } else {
                            // the string value is not what we expected
                            TestOutcome::Failed(Failure::StringValue(
                                self,
                                AssertStringValueFailure::WrongStringValue(joined),
                            ))
                        }
                    }
                    // we weren't able to produce a string value
                    Err(_) => TestOutcome::Failed(Failure::StringValue(
                        self,
                        AssertStringValueFailure::WrongValue(value),
                    )),
                }
            }
            // we weren't able to produce a sequence
            Err(_) => TestOutcome::Failed(Failure::StringValue(
                self,
                AssertStringValueFailure::WrongValue(value),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct AssertError(String);

impl AssertError {
    pub(crate) fn new(code: String) -> Self {
        Self(code)
    }
}

impl Assertable for AssertError {
    fn assert_result(&self, _xot: &mut Xot, result: &Result<Value, Error>) -> TestOutcome {
        match result {
            Ok(value) => TestOutcome::Failed(Failure::Error(self, value.clone())),
            Err(error) => {
                // all errors are officially a pass, but we check whether the error
                // code matches too
                let code = error.code();
                if let Some(code) = code {
                    if code.to_string() == self.0 {
                        TestOutcome::Passed
                    } else {
                        TestOutcome::PassedWithUnexpectedError(UnexpectedError::Code(
                            code.to_string(),
                        ))
                    }
                } else {
                    TestOutcome::PassedWithUnexpectedError(UnexpectedError::Error(error.clone()))
                }
            }
        }
    }

    fn assert_value(&self, _xot: &mut Xot, _value: Value) -> TestOutcome {
        unreachable!();
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TestCaseResult {
    AnyOf(AssertAnyOf),
    AllOf(AssertAllOf),
    Not(AssertNot),
    // The assert element contains an XPath expression whose effective boolean
    // value must be true; usually the expression will use the variable $result
    // which references the result of the expression.
    Assert(Assert),
    // The assert element contains an XPath expression (usually a simple string
    // or numeric literal) which must be equal to the result of the test case
    // under the rules of the XPath 'eq' operator.
    AssertEq(AssertEq),
    // Asserts that the result must be a sequence containing a given number of
    // items. The value of the element is an integer giving the expected length
    // of the sequence.
    AssertCount(AssertCount),
    // Asserts that the result must be a sequence of atomic values that is
    // deep-equal to the supplied sequence under the rules of the deep-equal()
    // function.
    AssertDeepEq(AssertDeepEq),
    //  Asserts that the result must be a sequence of atomic values that has
    //  some permutation (reordering) that is deep-equal to the supplied
    //  sequence under the rules of the deep-equal() function.
    // Note this implies that NaN is equal to NaN.
    AssertPermutation(AssertPermutation),
    // Asserts the result of the query by providing a serialization of the
    // expression result using the default serialization parameters
    // method="xml" indent="no" omit-xml-declaration="yes".
    AssertXml(AssertXml),
    //  Asserts that the result of the test is an empty sequence.
    AssertEmpty(AssertEmpty),
    // Asserts the result of serializing the query matches a given regular
    // expression.
    // XXX values not right
    SerializationMatches(AssertSerializationMatches),
    // Asserts that the query can be executed without error, but serializing
    // the result produces a serialization error. The result of the query must
    // be serialized using the serialization options specified within the query
    // (if any).
    AssertSerializationError(AssertSerializationError),
    // Asserts that the result of the test matches the sequence type given as
    // the value of the assert-type element.
    AssertType(AssertType),
    // Asserts that the result of the test is the singleton boolean value
    // false(). Note, the test expression must actually evaluate to false: this
    // is not an assertion on the effective boolean value.
    AssertTrue(AssertTrue),
    // Asserts that the result of the test is the singleton boolean value
    // false(). Note, the test expression must actually evaluate to false: this
    // is not an assertion on the effective boolean value.
    AssertFalse(AssertFalse),
    // Asserts that the result of the test, after conversion to a string by
    // applying the expression string-join(for $r in $result return string($r),
    // " ") is equal to the string value of the assert-string-value element.
    // Note that this test cannot be used if the result includes items that do
    // not have a string value (elements with element-only content; function
    // items) If the normalize-space attribute is present with the value true,
    // then both the string value of the query result and the value of the
    // assert-string-value element should be processed as if by the XPath
    // normalize-space() function before the comparison.
    AssertStringValue(AssertStringValue),
    //  Asserts that the test is expected to fail with a static or dynamic
    //  error condition. The "code" attribute gives the expected error code.
    //
    // For the purpose of official test reporting, an implementation is
    // considered to pass a test if the test expects and error and the
    // implementation raises an error, regardless whether the error codes
    // match.
    AssertError(AssertError),
    // This assertion type is as of yet unsupported, and will automatically error
    Unsupported,
}

impl TestCaseResult {
    pub(crate) fn assert_result(
        &self,
        xot: &mut Xot,
        result: &Result<Value, Error>,
    ) -> TestOutcome {
        match self {
            TestCaseResult::AnyOf(a) => a.assert_result(xot, result),
            TestCaseResult::AllOf(a) => a.assert_result(xot, result),
            TestCaseResult::AssertEq(a) => a.assert_result(xot, result),
            TestCaseResult::AssertTrue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertFalse(a) => a.assert_result(xot, result),
            TestCaseResult::AssertCount(a) => a.assert_result(xot, result),
            TestCaseResult::AssertStringValue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertXml(a) => a.assert_result(xot, result),
            TestCaseResult::AssertError(a) => a.assert_result(xot, result),
            TestCaseResult::Unsupported => TestOutcome::Unsupported,
            _ => {
                panic!("unimplemented test case result {:?}", self);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum AssertCountFailure {
    WrongCount(usize),
    WrongValue(Value),
}

#[derive(Debug, PartialEq)]
pub(crate) enum AssertStringValueFailure {
    WrongStringValue(String),
    WrongValue(Value),
}

#[derive(Debug, PartialEq)]
pub(crate) enum AssertXmlFailure {
    WrongXml(String),
    WrongValue(Value),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Failure<'a> {
    AnyOf(&'a AssertAnyOf, Vec<TestOutcome<'a>>),
    Not(&'a AssertNot, Box<TestOutcome<'a>>),
    Eq(&'a AssertEq, Value),
    True(&'a AssertTrue, Value),
    False(&'a AssertFalse, Value),
    Count(&'a AssertCount, AssertCountFailure),
    StringValue(&'a AssertStringValue, AssertStringValueFailure),
    Xml(&'a AssertXml, AssertXmlFailure),
    Error(&'a AssertError, Value),
}

impl<'a> fmt::Display for Failure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::AnyOf(a, outcomes) => {
                writeln!(f, "any of:")?;
                for outcome in outcomes {
                    match outcome {
                        TestOutcome::Failed(failure) => {
                            writeln!(f, "  {}", failure);
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                Ok(())
            }
            Failure::Not(_a, _outcome) => {
                writeln!(f, "not:")?;
                // writeln!(f, "  {}", outcome)?;
                Ok(())
            }
            Failure::Eq(a, value) => {
                writeln!(f, "eq:")?;
                writeln!(f, "  expected: {:?}", a.0)?;
                writeln!(f, "  actual: {:?}", value)?;
                Ok(())
            }
            Failure::True(_a, value) => {
                writeln!(f, "true:")?;
                writeln!(f, "  expected: true")?;
                writeln!(f, "  actual: {:?}", value)?;
                Ok(())
            }
            Failure::False(_a, value) => {
                writeln!(f, "false:")?;
                writeln!(f, "  expected: false")?;
                writeln!(f, "  actual: {:?}", value)?;
                Ok(())
            }
            Failure::Count(a, failure) => {
                writeln!(f, "count:")?;
                writeln!(f, "  expected: {:?}", a.0)?;
                writeln!(f, "  actual: {:?}", failure)?;
                Ok(())
            }
            Failure::StringValue(a, failure) => {
                writeln!(f, "string-value:")?;
                writeln!(f, "  expected: {:?}", a.0)?;
                writeln!(f, "  actual: {:?}", failure)?;
                Ok(())
            }
            Failure::Xml(a, failure) => {
                writeln!(f, "xml:")?;
                writeln!(f, "  expected: {:?}", a.0)?;
                writeln!(f, "  actual: {:?}", failure)?;
                Ok(())
            }
            Failure::Error(a, value) => {
                writeln!(f, "error:")?;
                writeln!(f, "  expected: {:?}", a.0)?;
                writeln!(f, "  actual: {:?}", value)?;
                Ok(())
            }
        }
    }
}

fn run_xpath(expr: &qt::XPathExpr) -> Result<Value, Error> {
    let namespaces = Namespaces::default();
    let static_context = StaticContext::new(&namespaces);
    let xpath = XPath::new(&static_context, &expr.0)?;
    let xot = Xot::new();
    let dynamic_context = DynamicContext::new(&xot, &static_context);
    xpath.run(&dynamic_context, None)
}

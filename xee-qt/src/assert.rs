use crossterm::style::Stylize;
use miette::Diagnostic;
use std::fmt;
use xee_xpath::{
    DynamicContext, Error, Name, Namespaces, Occurrence, Result, Sequence, StaticContext, XPath,
};
use xot::Xot;

use crate::qt;
use crate::serialize::serialize;

#[derive(Debug, PartialEq)]
pub enum UnexpectedError {
    Code(String),
    Error(Error),
}

#[derive(Debug, PartialEq)]
pub enum TestOutcome {
    Passed,
    PassedWithUnexpectedError(UnexpectedError),
    Failed(Failure),
    RuntimeError(Error),
    CompilationError(Error),
    UnsupportedExpression(Error),
    Unsupported,
    EnvironmentError(String),
}

#[derive(Debug)]
pub struct TestOutcomes(pub Vec<(String, TestOutcome)>);

impl std::fmt::Display for TestOutcomes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (name, test_outcome) in self.0.iter() {
            writeln!(f, "{} ... {}", name, test_outcome)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOutcome::Passed => write!(f, "{}", "PASS".green()),
            TestOutcome::PassedWithUnexpectedError(error) => match error {
                UnexpectedError::Code(s) => write!(f, "{} code: {}", "PASS".green(), s),
                UnexpectedError::Error(e) => write!(f, "{} error: {}", "PASS".green(), e),
            },
            TestOutcome::Failed(failure) => {
                write!(f, "{} {}", "FAIL".red(), failure)
            }
            TestOutcome::RuntimeError(error) => match error.code() {
                Some(code) => {
                    write!(f, "{} {} {}", "RUNTIME ERROR".red(), code, error)
                }
                None => {
                    write!(f, "{} {}", "RUNTIME ERROR".red(), error)
                }
            },
            TestOutcome::CompilationError(error) => match error.code() {
                Some(code) => {
                    write!(f, "{} {} {}", "COMPILATION ERROR".red(), code, error)
                }
                None => {
                    write!(f, "{} {}", "COMPILATION ERROR".red(), error)
                }
            },
            TestOutcome::UnsupportedExpression(error) => {
                write!(f, "{} {}", "UNSUPPORTED EXPRESSION ERROR".red(), error)
            }
            TestOutcome::Unsupported => {
                write!(f, "{}", "UNSUPPORTED".red())
            }
            TestOutcome::EnvironmentError(error) => {
                write!(f, "{} {}", "CONTEXT ITEM ERROR".red(), error)
            }
        }
    }
}

impl TestOutcome {
    pub(crate) fn is_passed(&self) -> bool {
        matches!(self, Self::Passed | Self::PassedWithUnexpectedError(..))
    }
    pub(crate) fn is_exactly_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

pub(crate) trait Assertable {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Sequence>) -> TestOutcome {
        match result {
            Ok(sequence) => self.assert_value(xot, sequence),
            Err(error) => TestOutcome::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertAnyOf(Vec<TestCaseResult>);

impl AssertAnyOf {
    pub(crate) fn new(test_case_results: Vec<TestCaseResult>) -> Self {
        Self(test_case_results)
    }
}

impl Assertable for AssertAnyOf {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Sequence>) -> TestOutcome {
        let mut failed_test_results = Vec::new();
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(xot, result);
            match result {
                TestOutcome::Passed | TestOutcome::PassedWithUnexpectedError(..) => return result,
                _ => failed_test_results.push(result),
            }
        }
        match result {
            Ok(_value) => TestOutcome::Failed(Failure::AnyOf(self.clone(), failed_test_results)),
            Err(error) => TestOutcome::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, _xot: &mut Xot, _sequence: &Sequence) -> TestOutcome {
        unreachable!();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AssertAllOf(Vec<TestCaseResult>);

impl AssertAllOf {
    pub(crate) fn new(test_case_results: Vec<TestCaseResult>) -> Self {
        Self(test_case_results)
    }
}

impl Assertable for AssertAllOf {
    fn assert_result(&self, xot: &mut Xot, result: &Result<Sequence>) -> TestOutcome {
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(xot, result);
            match result {
                TestOutcome::Passed | TestOutcome::PassedWithUnexpectedError(..) => {}
                _ => return result,
            }
        }
        TestOutcome::Passed
    }

    fn assert_value(&self, _xot: &mut Xot, _sequence: &Sequence) -> TestOutcome {
        unreachable!();
    }
}

#[derive(PartialEq, Clone)]
pub struct AssertNot(Box<TestCaseResult>);

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

#[derive(Debug, Clone, PartialEq)]
pub struct Assert(qt::XPathExpr);

impl Assert {
    pub(crate) fn new(expr: qt::XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for Assert {
    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        let result_sequence = run_xpath_with_result(&self.0, sequence, xot);

        match result_sequence {
            Ok(result_sequence) => match result_sequence.effective_boolean_value() {
                Ok(value) => {
                    if value {
                        TestOutcome::Passed
                    } else {
                        TestOutcome::Failed(Failure::Assert(self.clone(), sequence.clone()))
                    }
                }
                Err(error) => TestOutcome::RuntimeError(error),
            },
            Err(error) => TestOutcome::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertEq(qt::XPathExpr);

impl AssertEq {
    pub(crate) fn new(expr: qt::XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for AssertEq {
    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        let expected_sequence = run_xpath(&self.0, xot);

        match expected_sequence {
            Ok(expected_sequence) => {
                if &expected_sequence == sequence {
                    TestOutcome::Passed
                } else {
                    TestOutcome::Failed(Failure::Eq(self.clone(), sequence.clone()))
                }
            }
            Err(error) => TestOutcome::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertCount(usize);

impl AssertCount {
    pub(crate) fn new(count: usize) -> Self {
        Self(count)
    }
}

impl Assertable for AssertCount {
    fn assert_value(&self, _xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        let found_len = sequence.len();
        if found_len == self.0 {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::Count(
                self.clone(),
                AssertCountFailure::WrongCount(found_len),
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertDeepEq(qt::XPathExpr);

#[derive(Debug, Clone, PartialEq)]
pub struct AssertPermutation(qt::XPathExpr);

#[derive(Debug, Clone, PartialEq)]
pub struct AssertXml(String);

impl AssertXml {
    pub(crate) fn new(xml: String) -> Self {
        Self(xml)
    }
}

impl Assertable for AssertXml {
    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        let xml = serialize(xot, sequence);

        let xml = if let Ok(xml) = xml {
            xml
        } else {
            return TestOutcome::Failed(Failure::Xml(
                self.clone(),
                AssertXmlFailure::WrongValue(sequence.clone()),
            ));
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
            TestOutcome::Failed(Failure::Xml(self.clone(), AssertXmlFailure::WrongXml(xml)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertEmpty;

impl AssertEmpty {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Assertable for AssertEmpty {
    fn assert_value(&self, _xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        if sequence.is_empty() {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::Empty(self.clone(), sequence.clone()))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AssertSerializationMatches;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AssertSerializationError(String);

#[derive(Debug, Clone, PartialEq)]
pub struct AssertType(String);

impl AssertType {
    pub(crate) fn new(type_name: String) -> Self {
        Self(type_name)
    }
}

impl Assertable for AssertType {
    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        // TODO: ugly unwrap in here; what if qt test has sequence type that cannot
        // be parsed?
        if sequence.matches_type(&self.0, xot).unwrap() {
            TestOutcome::Passed
        } else {
            TestOutcome::Failed(Failure::Type(self.clone(), sequence.clone()))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertTrue;

impl AssertTrue {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Assertable for AssertTrue {
    fn assert_value(&self, _xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        if let Ok(item) = sequence.items().one() {
            if let Ok(atomic) = item.to_atomic() {
                let b: Result<bool> = atomic.try_into();
                if let Ok(b) = b {
                    if b {
                        return TestOutcome::Passed;
                    }
                }
            }
        }
        TestOutcome::Failed(Failure::True(self.clone(), sequence.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertFalse;

impl AssertFalse {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Assertable for AssertFalse {
    fn assert_value(&self, _xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        if let Ok(item) = sequence.items().one() {
            if let Ok(atomic) = item.to_atomic() {
                let b: Result<bool> = atomic.try_into();
                if let Ok(b) = b {
                    if !b {
                        return TestOutcome::Passed;
                    }
                }
            }
        }
        TestOutcome::Failed(Failure::False(self.clone(), sequence.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertStringValue(String);

impl AssertStringValue {
    pub(crate) fn new(string: String) -> Self {
        Self(string)
    }
}

impl Assertable for AssertStringValue {
    fn assert_value(&self, xot: &mut Xot, sequence: &Sequence) -> TestOutcome {
        let strings = sequence
            .items()
            .map(|item| item?.string_value(xot))
            .collect::<Result<Vec<_>>>();
        match strings {
            Ok(strings) => {
                let joined = strings.join(" ");
                if joined == self.0 {
                    TestOutcome::Passed
                } else {
                    // the string value is not what we expected
                    TestOutcome::Failed(Failure::StringValue(
                        self.clone(),
                        AssertStringValueFailure::WrongStringValue(joined),
                    ))
                }
            }
            // we weren't able to produce a string value
            Err(_) => TestOutcome::Failed(Failure::StringValue(
                self.clone(),
                AssertStringValueFailure::WrongValue(sequence.clone()),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertError(String);

impl AssertError {
    pub(crate) fn new(code: String) -> Self {
        Self(code)
    }
}

impl Assertable for AssertError {
    fn assert_result(&self, _xot: &mut Xot, result: &Result<Sequence>) -> TestOutcome {
        match result {
            Ok(sequence) => TestOutcome::Failed(Failure::Error(self.clone(), sequence.clone())),
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

    fn assert_value(&self, _xot: &mut Xot, _: &Sequence) -> TestOutcome {
        unreachable!();
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub(crate) fn assert_result(&self, xot: &mut Xot, result: &Result<Sequence>) -> TestOutcome {
        match self {
            TestCaseResult::AnyOf(a) => a.assert_result(xot, result),
            TestCaseResult::AllOf(a) => a.assert_result(xot, result),
            TestCaseResult::AssertEq(a) => a.assert_result(xot, result),
            TestCaseResult::AssertTrue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertFalse(a) => a.assert_result(xot, result),
            TestCaseResult::AssertCount(a) => a.assert_result(xot, result),
            TestCaseResult::AssertStringValue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertXml(a) => a.assert_result(xot, result),
            TestCaseResult::Assert(a) => a.assert_result(xot, result),
            TestCaseResult::AssertError(a) => a.assert_result(xot, result),
            TestCaseResult::AssertEmpty(a) => a.assert_result(xot, result),
            TestCaseResult::Unsupported => TestOutcome::Unsupported,
            _ => {
                panic!("unimplemented test case result {:?}", self);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AssertCountFailure {
    WrongCount(usize),
    WrongValue(Sequence),
}

#[derive(Debug, PartialEq)]
pub enum AssertStringValueFailure {
    WrongStringValue(String),
    WrongValue(Sequence),
}

#[derive(Debug, PartialEq)]
pub enum AssertXmlFailure {
    WrongXml(String),
    WrongValue(Sequence),
}

#[derive(Debug, PartialEq)]
pub enum Failure {
    AnyOf(AssertAnyOf, Vec<TestOutcome>),
    Not(AssertNot, Box<TestOutcome>),
    Eq(AssertEq, Sequence),
    True(AssertTrue, Sequence),
    False(AssertFalse, Sequence),
    Count(AssertCount, AssertCountFailure),
    StringValue(AssertStringValue, AssertStringValueFailure),
    Xml(AssertXml, AssertXmlFailure),
    Assert(Assert, Sequence),
    Empty(AssertEmpty, Sequence),
    Error(AssertError, Sequence),
    Type(AssertType, Sequence),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::AnyOf(_, outcomes) => {
                writeln!(f, "any of:")?;
                for outcome in outcomes {
                    match outcome {
                        TestOutcome::Failed(failure) => {
                            writeln!(f, "  {}", failure)?;
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
            Failure::Assert(_a, failure) => {
                writeln!(f, "assert:")?;
                writeln!(f, "  actual: {:?}", failure)?;
                Ok(())
            }
            Failure::Empty(_a, value) => {
                writeln!(f, "empty:")?;
                writeln!(f, "  actual: {:?}", value)?;
                Ok(())
            }
            Failure::Type(_a, value) => {
                writeln!(f, "type:")?;
                writeln!(f, "  expected type: {:?}", _a.0)?;
                writeln!(f, "  value of wrong type: {:?}", value)?;
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

fn run_xpath(expr: &qt::XPathExpr, xot: &Xot) -> Result<Sequence> {
    let namespaces = Namespaces::default();
    let static_context = StaticContext::new(&namespaces);
    let xpath = XPath::new(&static_context, &expr.0)?;
    let dynamic_context = DynamicContext::new(xot, &static_context);
    xpath.many(&dynamic_context, None)
}

fn run_xpath_with_result(expr: &qt::XPathExpr, sequence: &Sequence, xot: &Xot) -> Result<Sequence> {
    let namespaces = Namespaces::default();
    let name = Name::unprefixed("result");
    let names = vec![name.clone()];
    let static_context = StaticContext::with_variable_names(&namespaces, &names);
    let xpath = XPath::new(&static_context, &expr.0)?;
    let variables = vec![(name, sequence.items().collect::<Result<Vec<_>>>()?)];
    let dynamic_context = DynamicContext::with_variables(xot, &static_context, &variables);
    xpath.many(&dynamic_context, None)
}

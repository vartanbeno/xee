use miette::Diagnostic;
use std::fmt;

use xee_xpath::{Atomic, DynamicContext, Error, Namespaces, StackValue, StaticContext, XPath};
use xot::Xot;

use crate::qt;
use crate::serialize::serialize;

pub(crate) enum UnexpectedError {
    Code(String),
    Error(Error),
}

pub(crate) enum TestResult<'a> {
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
    fn assert_result(&self, xot: &mut Xot, result: &Result<StackValue, Error>) -> TestResult {
        match result {
            Ok(value) => self.assert_value(xot, value.clone()),
            Err(error) => TestResult::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, xot: &mut Xot, value: StackValue) -> TestResult;
}

#[derive(Debug)]
pub(crate) struct AssertAnyOf(Vec<TestCaseResult>);

impl Assertable for AssertAnyOf {
    fn assert_result(&self, xot: &mut Xot, result: &Result<StackValue, Error>) -> TestResult {
        let mut failed_test_results = Vec::new();
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(xot, result);
            match result {
                TestResult::Passed | TestResult::PassedWithUnexpectedError(..) => return result,
                _ => failed_test_results.push(result),
            }
        }
        match result {
            Ok(_value) => TestResult::Failed(Failure::AnyOf(self, failed_test_results)),
            Err(error) => TestResult::RuntimeError(error.clone()),
        }
    }

    fn assert_value(&self, _xot: &mut Xot, _value: StackValue) -> TestResult {
        unreachable!();
    }
}

#[derive(Debug)]
pub(crate) struct AssertAllOf(Vec<TestCaseResult>);

pub(crate) struct AssertNot(Box<TestCaseResult>);

impl fmt::Debug for AssertNot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AssertNot({:?})", self.0)
    }
}

#[derive(Debug)]
pub(crate) struct Assert(qt::XPathExpr);

#[derive(Debug)]
pub(crate) struct AssertEq(qt::XPathExpr);

impl Assertable for AssertEq {
    fn assert_value(&self, _xot: &mut Xot, value: StackValue) -> TestResult {
        let expected_value = run_xpath(&self.0);

        match expected_value {
            Ok(expected_value) => {
                if expected_value == value {
                    TestResult::Passed
                } else {
                    TestResult::Failed(Failure::Eq(self, value))
                }
            }
            Err(error) => TestResult::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertCount(usize);

impl Assertable for AssertCount {
    fn assert_value(&self, _xot: &mut Xot, value: StackValue) -> TestResult {
        let sequence = value.to_sequence();
        if let Ok(sequence) = sequence {
            let found_len = sequence.borrow().len();
            if found_len == self.0 {
                TestResult::Passed
            } else {
                TestResult::Failed(Failure::Count(
                    self,
                    AssertCountFailure::WrongCount(found_len),
                ))
            }
        } else {
            TestResult::Failed(Failure::Count(
                self,
                AssertCountFailure::WrongStackValue(value),
            ))
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertDeepEq(qt::XPathExpr);

#[derive(Debug)]
pub(crate) struct AssertPermutation(qt::XPathExpr);

#[derive(Debug)]
pub(crate) struct AssertXml(String);

impl Assertable for AssertXml {
    fn assert_value(&self, xot: &mut Xot, value: StackValue) -> TestResult {
        let xml = serialize(xot, &value);

        let xml = if let Ok(xml) = xml {
            xml
        } else {
            return TestResult::Failed(Failure::Xml(
                self,
                AssertXmlFailure::WrongStackValue(value),
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
            TestResult::Passed
        } else {
            TestResult::Failed(Failure::Xml(self, AssertXmlFailure::WrongXml(xml)))
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertEmpty;

#[derive(Debug)]
pub(crate) struct AssertSerializationMatches;

#[derive(Debug)]
pub(crate) struct AssertSerializationError(String);

#[derive(Debug)]
pub(crate) struct AssertType(String);

#[derive(Debug)]
pub(crate) struct AssertTrue;

impl Assertable for AssertTrue {
    fn assert_value(&self, _xot: &mut Xot, value: StackValue) -> TestResult {
        if matches!(value, StackValue::Atomic(Atomic::Boolean(true))) {
            TestResult::Passed
        } else {
            TestResult::Failed(Failure::True(self, value))
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertFalse;

impl Assertable for AssertFalse {
    fn assert_value(&self, _xot: &mut Xot, value: StackValue) -> TestResult {
        if matches!(value, StackValue::Atomic(Atomic::Boolean(false))) {
            TestResult::Passed
        } else {
            TestResult::Failed(Failure::False(self, value))
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertStringValue(String);

impl Assertable for AssertStringValue {
    fn assert_value(&self, xot: &mut Xot, value: StackValue) -> TestResult {
        let seq = value.to_sequence();
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
                            TestResult::Passed
                        } else {
                            // the string value is not what we expected
                            TestResult::Failed(Failure::StringValue(
                                self,
                                AssertStringValueFailure::WrongStringValue(joined),
                            ))
                        }
                    }
                    // we weren't able to produce a string value
                    Err(_) => TestResult::Failed(Failure::StringValue(
                        self,
                        AssertStringValueFailure::WrongStackValue(value),
                    )),
                }
            }
            // we weren't able to produce a sequence
            Err(_) => TestResult::Failed(Failure::StringValue(
                self,
                AssertStringValueFailure::WrongStackValue(value),
            )),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AssertError(String);

impl Assertable for AssertError {
    fn assert_result(&self, xot: &mut Xot, result: &Result<StackValue, Error>) -> TestResult {
        match result {
            Ok(value) => TestResult::Failed(Failure::Error(self, value.clone())),
            Err(error) => {
                // all errors are officially a pass, but we check whether the error
                // code matches too
                let code = error.code();
                if let Some(code) = code {
                    if code.to_string() == self.0 {
                        TestResult::Passed
                    } else {
                        TestResult::PassedWithUnexpectedError(UnexpectedError::Code(
                            code.to_string(),
                        ))
                    }
                } else {
                    TestResult::PassedWithUnexpectedError(UnexpectedError::Error(error.clone()))
                }
            }
        }
    }

    fn assert_value(&self, xot: &mut Xot, value: StackValue) -> TestResult {
        unreachable!();
    }
}

#[derive(Debug)]
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
    fn assert_result(&self, xot: &mut Xot, result: &Result<StackValue, Error>) -> TestResult {
        match self {
            TestCaseResult::AssertEq(a) => a.assert_result(xot, result),
            TestCaseResult::AssertTrue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertFalse(a) => a.assert_result(xot, result),
            TestCaseResult::AssertCount(a) => a.assert_result(xot, result),
            TestCaseResult::AssertStringValue(a) => a.assert_result(xot, result),
            TestCaseResult::AssertXml(a) => a.assert_result(xot, result),
            TestCaseResult::AssertError(a) => a.assert_result(xot, result),
            TestCaseResult::Unsupported => TestResult::Unsupported,
            _ => {
                panic!("unimplemented test case result {:?}", self);
            }
        }
    }
}
pub(crate) enum AssertCountFailure {
    WrongCount(usize),
    WrongStackValue(StackValue),
}

pub(crate) enum AssertStringValueFailure {
    WrongStringValue(String),
    WrongStackValue(StackValue),
}

pub(crate) enum AssertXmlFailure {
    WrongXml(String),
    WrongStackValue(StackValue),
}

pub(crate) enum Failure<'a> {
    AnyOf(&'a AssertAnyOf, Vec<TestResult<'a>>),
    AllOf(&'a AssertAllOf, Box<TestResult<'a>>),
    Not(&'a AssertNot, Box<TestResult<'a>>),
    Eq(&'a AssertEq, StackValue),
    True(&'a AssertTrue, StackValue),
    False(&'a AssertFalse, StackValue),
    Count(&'a AssertCount, AssertCountFailure),
    StringValue(&'a AssertStringValue, AssertStringValueFailure),
    Xml(&'a AssertXml, AssertXmlFailure),
    Error(&'a AssertError, StackValue),
}

// fn check_value<'a>(
//     xot: &'a mut Xot,
//     result: &'a TestCaseResult,
//     run_result: &'a Result<StackValue, Error>,
// ) -> TestResult<'a> {
//     // we handle any of and all of first, because we don't
//     // yet want to distinguish between value and error
//     match result {
//         qt::TestCaseResult::AllOf(test_case_results) => {
//             return Self::assert_all_of(xot, test_case_results, run_result)
//         }
//         qt::TestCaseResult::AnyOf(test_case_results) => {
//             return Self::assert_any_of(xot, test_case_results, run_result)
//         }
//         _ => {}
//     }
//     match run_result {
//         Ok(value) => {
//             let value = value.clone();
//             match result {
//                 // qt::TestCaseResult::Assert(xpath_expr) => self.assert_(xpath_expr, run_result),
//                 qt::TestCaseResult::AssertEq(xpath_expr) => Self::assert_eq(xpath_expr, value),
//                 qt::TestCaseResult::AssertTrue => Self::assert_true(value),
//                 qt::TestCaseResult::AssertFalse => Self::assert_false(value),
//                 qt::TestCaseResult::AssertCount(number) => Self::assert_count(*number, value),
//                 qt::TestCaseResult::AssertStringValue(s) => {
//                     Self::assert_string_value(xot, s, value)
//                 }
//                 qt::TestCaseResult::AssertXml(xml) => Self::assert_xml(xot, xml, value),
//                 qt::TestCaseResult::AssertError(error) => {
//                     Self::assert_unexpected_no_error(error, value)
//                 }
//                 qt::TestCaseResult::Unsupported => TestResult::Unsupported,
//                 _ => {
//                     panic!("unimplemented test case result {:?}", result);
//                 }
//             }
//         }
//         Err(error) => match result {
//             qt::TestCaseResult::AssertError(expected_error) => {
//                 Self::assert_expected_error(expected_error, error)
//             }
//             _ => TestResult::RuntimeError(error.clone()),
//         },
//     }
// }

fn run_xpath(expr: &qt::XPathExpr) -> Result<StackValue, Error> {
    let namespaces = Namespaces::default();
    let static_context = StaticContext::new(&namespaces);
    let xpath = XPath::new(&static_context, &expr.0)?;
    let xot = Xot::new();
    let dynamic_context = DynamicContext::new(&xot, &static_context);
    xpath.run(&dynamic_context, None)
}

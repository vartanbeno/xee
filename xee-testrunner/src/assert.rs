use ahash::AHashMap;
use chrono::Offset;
use std::borrow::Cow;
use std::fmt;
use xee_xpath::{
    context::DynamicContext, context::StaticContext, error::Error, error::Result,
    occurrence::Occurrence, parse, sequence::Sequence, string::Collation, Name, Namespaces,
    Runnable, VariableNames,
};
use xot::Xot;

use crate::error as testerror;
use crate::outcome::{OutcomeStatus, UnexpectedError};

type XPathExpr = String;

pub(crate) trait Assertable {
    fn assert_result(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        match result {
            Ok(sequence) => self.assert_value(runnable, xot, sequence),
            Err(error) => OutcomeStatus::RuntimeError(error.clone()),
        }
    }

    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertAnyOf(Vec<TestCaseResult>);

impl AssertAnyOf {
    pub(crate) fn new(test_case_results: Vec<TestCaseResult>) -> Self {
        Self(test_case_results)
    }

    pub(crate) fn assert_error(&self, error: &Error) -> OutcomeStatus {
        let mut failed_test_results = Vec::new();
        for test_case_result in &self.0 {
            if let TestCaseResult::AssertError(assert_error) = test_case_result {
                let result = assert_error.assert_error(error);
                match result {
                    OutcomeStatus::Passed => return result,
                    _ => failed_test_results.push(result),
                }
            } else {
                // any non-error is a failure, as we arrived with an error
                return OutcomeStatus::Failed;
            }
        }
        OutcomeStatus::Failed
    }
}

impl Assertable for AssertAnyOf {
    fn assert_result(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        let mut failed_test_results = Vec::new();
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(runnable, xot, result);
            match result {
                OutcomeStatus::Passed => return result,
                _ => failed_test_results.push(result),
            }
        }
        match result {
            Ok(_value) => OutcomeStatus::Failed,
            Err(error) => OutcomeStatus::RuntimeError(error.clone()),
        }
    }

    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        _sequence: &Sequence,
    ) -> OutcomeStatus {
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
    fn assert_result(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        for test_case_result in &self.0 {
            let result = test_case_result.assert_result(runnable, xot, result);
            match result {
                OutcomeStatus::Passed | OutcomeStatus::PassedWithUnexpectedError(..) => {}
                _ => return result,
            }
        }
        OutcomeStatus::Passed
    }

    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        _sequence: &Sequence,
    ) -> OutcomeStatus {
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

impl Assertable for AssertNot {
    fn assert_result(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        let result = self.0.assert_result(runnable, xot, result);
        match result {
            OutcomeStatus::Passed => OutcomeStatus::Failed,
            OutcomeStatus::Failed => OutcomeStatus::Passed,
            _ => result,
        }
    }

    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        _sequence: &Sequence,
    ) -> OutcomeStatus {
        unreachable!();
    }
}

impl fmt::Debug for AssertNot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AssertNot({:?})", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assert(XPathExpr);

impl Assert {
    pub(crate) fn new(expr: XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for Assert {
    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let result_sequence = run_xpath_with_result(&self.0, sequence, runnable, xot);

        match result_sequence {
            Ok(result_sequence) => match result_sequence.effective_boolean_value() {
                Ok(value) => {
                    if value {
                        OutcomeStatus::Passed
                    } else {
                        OutcomeStatus::Failed
                    }
                }
                Err(error) => OutcomeStatus::RuntimeError(error),
            },
            Err(error) => OutcomeStatus::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertEq(XPathExpr);

impl AssertEq {
    pub(crate) fn new(expr: XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for AssertEq {
    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let expected_sequence = run_xpath(&self.0, runnable, xot);

        match expected_sequence {
            Ok(expected_sequence) => {
                let atom = sequence.atomized(xot).one();
                let atom = match atom {
                    Ok(atom) => atom,
                    Err(error) => return OutcomeStatus::RuntimeError(error),
                };
                let expected_atom = expected_sequence
                    .atomized(xot)
                    .one()
                    .expect("Should get single atom in sequence");
                if expected_atom.simple_equal(&atom) {
                    OutcomeStatus::Passed
                } else {
                    OutcomeStatus::Failed
                }
            }
            Err(error) => OutcomeStatus::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertDeepEq(XPathExpr);

impl AssertDeepEq {
    pub(crate) fn new(expr: XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for AssertDeepEq {
    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let expected_sequence = run_xpath(&self.0, runnable, xot);

        match expected_sequence {
            Ok(expected_sequence) => {
                if expected_sequence
                    .deep_equal(
                        sequence,
                        &Collation::CodePoint,
                        chrono::offset::Utc.fix(),
                        xot,
                    )
                    .unwrap_or(false)
                {
                    OutcomeStatus::Passed
                } else {
                    OutcomeStatus::Failed
                }
            }
            Err(error) => OutcomeStatus::UnsupportedExpression(error),
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
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let found_len = sequence.len();
        if found_len == self.0 {
            OutcomeStatus::Passed
        } else {
            OutcomeStatus::Failed
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertPermutation(XPathExpr);

impl AssertPermutation {
    pub(crate) fn new(expr: XPathExpr) -> Self {
        Self(expr)
    }
}

impl Assertable for AssertPermutation {
    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        // sequence should consist of atoms. sort these so we only have to
        // compare to one permutation.
        let context = runnable.dynamic_context();
        let collation_str = runnable.default_collation_uri();
        let collation = runnable.default_collation().unwrap();
        let default_offset = runnable.implicit_timezone();

        let sequence = sequence.sorted(context, collation_str, xot);

        if let Err(err) = sequence {
            return OutcomeStatus::RuntimeError(err);
        }
        let sequence = sequence.unwrap();

        let result_sequence = run_xpath(&self.0, runnable, xot);

        match result_sequence {
            Ok(result_sequence) => {
                // sort result sequence too.
                let result_sequence = result_sequence.sorted(context, collation_str, xot);

                match result_sequence {
                    Ok(value) => {
                        if let Ok(true) =
                            sequence.deep_equal(&value, collation.as_ref(), default_offset, xot)
                        {
                            OutcomeStatus::Passed
                        } else {
                            OutcomeStatus::Failed
                        }
                    }
                    Err(error) => OutcomeStatus::RuntimeError(error),
                }
            }
            Err(error) => OutcomeStatus::UnsupportedExpression(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertXml(String);

impl AssertXml {
    pub(crate) fn new(xml: String) -> Self {
        Self(xml)
    }
}

impl Assertable for AssertXml {
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let xml = serialize(xot, sequence);

        let xml = if let Ok(xml) = xml {
            xml
        } else {
            return OutcomeStatus::Failed;
        };
        // also wrap expected XML in a sequence element
        let expected_xml = format!("<sequence>{}</sequence>", self.0);

        let mut compare_xot = Xot::new();
        // now parse both with Xot
        let found = compare_xot.parse(&xml).unwrap();
        let expected = compare_xot.parse(&expected_xml).unwrap();

        // and compare
        let c = compare_xot.deep_equal(expected, found);

        if c {
            OutcomeStatus::Passed
        } else {
            OutcomeStatus::Failed
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
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        if sequence.is_empty() {
            OutcomeStatus::Passed
        } else {
            OutcomeStatus::Failed
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
    fn assert_value(
        &self,
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let matches = sequence.matches_type(&self.0, xot, &|function| runnable.signature(function));
        match matches {
            Ok(matches) => {
                if matches {
                    OutcomeStatus::Passed
                } else {
                    OutcomeStatus::Failed
                }
            }
            Err(_) => {
                // we don't support this sequence type expression yet
                // this should resolve itself once we do and we can parse it
                OutcomeStatus::Unsupported
            }
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
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        if let Ok(item) = sequence.items().one() {
            if let Ok(atomic) = item.to_atomic() {
                let b: Result<bool> = atomic.try_into();
                if let Ok(b) = b {
                    if b {
                        return OutcomeStatus::Passed;
                    }
                }
            }
        }
        OutcomeStatus::Failed
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
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        if let Ok(item) = sequence.items().one() {
            if let Ok(atomic) = item.to_atomic() {
                let b: Result<bool> = atomic.try_into();
                if let Ok(b) = b {
                    if !b {
                        return OutcomeStatus::Passed;
                    }
                }
            }
        }
        OutcomeStatus::Failed
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertStringValue(String, bool);

impl AssertStringValue {
    pub(crate) fn new(string: String, normalize_space: bool) -> Self {
        Self(string, normalize_space)
    }
}

impl Assertable for AssertStringValue {
    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        xot: &mut Xot,
        sequence: &Sequence,
    ) -> OutcomeStatus {
        let strings = sequence
            .items()
            .map(|item| item?.string_value(xot))
            .collect::<Result<Vec<_>>>();
        match strings {
            Ok(strings) => {
                let joined = strings.join(" ");
                let joined = if self.1 {
                    // normalize space
                    joined
                        .split_ascii_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    joined
                };
                if joined == self.0 {
                    OutcomeStatus::Passed
                } else {
                    // the string value is not what we expected
                    OutcomeStatus::Failed
                }
            }
            // we weren't able to produce a string value
            Err(_) => OutcomeStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssertError(String);

impl AssertError {
    pub(crate) fn new(code: String) -> Self {
        Self(code)
    }

    pub(crate) fn assert_error(&self, error: &Error) -> OutcomeStatus {
        // all errors are officially a pass, but we check whether the error
        // code matches too
        let code = error.to_string();
        if code == self.0 {
            OutcomeStatus::Passed
        } else {
            OutcomeStatus::PassedWithUnexpectedError(UnexpectedError::Code(code.to_string()))
        }
    }
}

impl Assertable for AssertError {
    fn assert_result(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        match result {
            Ok(sequence) => OutcomeStatus::Failed,
            Err(error) => self.assert_error(error),
        }
    }

    fn assert_value(
        &self,
        _runnable: &Runnable<'_>,
        _xot: &mut Xot,
        _sequence: &Sequence,
    ) -> OutcomeStatus {
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
    // Asserts that the result must be a sequence of atomic values that is
    // deep-equal to the supplied sequence under the rules of the deep-equal()
    // function.
    AssertDeepEq(AssertDeepEq),
    // Asserts that the result must be a sequence containing a given number of
    // items. The value of the element is an integer giving the expected length
    // of the sequence.
    AssertCount(AssertCount),
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
        runnable: &Runnable<'_>,
        xot: &mut Xot,
        result: &Result<Sequence>,
    ) -> OutcomeStatus {
        match self {
            TestCaseResult::AnyOf(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AllOf(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::Not(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertEq(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertDeepEq(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertTrue(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertFalse(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertCount(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertStringValue(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertXml(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::Assert(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertPermutation(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertError(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertEmpty(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::AssertType(a) => a.assert_result(runnable, xot, result),
            TestCaseResult::Unsupported => OutcomeStatus::Unsupported,
            _ => {
                panic!("unimplemented test case result {:?}", self);
            }
        }
    }
}

fn run_xpath(expr: &XPathExpr, runnable: &Runnable<'_>, xot: &mut Xot) -> Result<Sequence> {
    let static_context = StaticContext::default();
    let program = parse(&static_context, expr).map_err(|e| e.error)?;
    let dynamic_context =
        DynamicContext::from_documents(&static_context, runnable.dynamic_context().documents());
    let runnable = program.runnable(&dynamic_context);
    runnable.many(None, xot).map_err(|e| e.error)
}

fn run_xpath_with_result(
    expr: &XPathExpr,
    sequence: &Sequence,
    runnable: &Runnable<'_>,
    xot: &mut Xot,
) -> Result<Sequence> {
    let namespaces = Namespaces::default();
    let name = Name::name("result");
    let names = VariableNames::from_iter([name.clone()]);
    let static_context = StaticContext::new(namespaces, names);
    let program = parse(&static_context, expr).map_err(|e| e.error)?;
    let variables = AHashMap::from([(name, sequence.clone())]);
    let dynamic_context = DynamicContext::new(
        &static_context,
        Cow::Borrowed(runnable.dynamic_context().documents()),
        Cow::Owned(variables),
    );
    let runnable = program.runnable(&dynamic_context);
    runnable.many(None, xot).map_err(|e| e.error)
}

// represent items as XML, if possible, wrapped
// in a sequence tag
pub(crate) fn serialize(xot: &Xot, sequence: &Sequence) -> testerror::Result<String> {
    let mut xmls = Vec::with_capacity(sequence.len());
    for item in sequence.items() {
        if let Ok(node) = item?.to_node() {
            let xml_value = xot.to_string(node);
            if let Ok(xml_value) = xml_value {
                xmls.push(xml_value);
            } else {
                return Err(testerror::Error::CannotRepresentAsXml);
            }
        } else {
            return Err(testerror::Error::CannotRepresentAsXml);
        }
    }
    Ok(format!("<sequence>{}</sequence>", xmls.join("")))
}

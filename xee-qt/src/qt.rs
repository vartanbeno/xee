use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use crate::collection::FxIndexSet;
use crate::environment::SharedEnvironments;

#[derive(Debug)]
pub(crate) struct Catalog {
    pub(crate) full_path: PathBuf,
    pub(crate) test_suite: String,
    pub(crate) version: String,
    pub(crate) shared_environments: SharedEnvironments,
    pub(crate) test_sets: Vec<TestSetRef>,
    pub(crate) file_paths: FxIndexSet<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct TestSet {
    pub(crate) full_path: PathBuf,
    pub(crate) name: String,
    pub(crate) descriptions: Vec<String>,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) shared_environments: SharedEnvironments,
    pub(crate) test_cases: Vec<TestCase>,
}

#[derive(Debug)]
pub(crate) struct TestSetRef {
    pub(crate) name: String,
    pub(crate) file: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct Metadata {
    pub(crate) description: Option<String>,
    pub(crate) created: Option<Attribution>,
    pub(crate) modified: Vec<Modification>,
}

#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) metadata: Metadata,
    // environments can be a reference by name, or a locally defined environment
    pub(crate) environments: Vec<TestCaseEnvironment>,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) modules: Vec<Module>,
    pub(crate) test: String,
    pub(crate) result: TestCaseResult,
}

#[derive(Debug, Clone)]
pub(crate) struct Attribution {
    pub(crate) by: String,
    pub(crate) on: String, // should be a date
}

#[derive(Debug, Clone)]
pub(crate) struct Modification {
    pub(crate) attribution: Attribution,
    pub(crate) description: String,
}

#[derive(Debug)]
pub(crate) struct XPathExpr(pub(crate) String);

#[derive(Debug)]
pub(crate) enum TestCaseResult {
    AnyOf(Vec<TestCaseResult>),
    AllOf(Vec<TestCaseResult>),
    Not(Box<TestCaseResult>),
    // The assert element contains an XPath expression whose effective boolean
    // value must be true; usually the expression will use the variable $result
    // which references the result of the expression.
    Assert(XPathExpr),
    // The assert element contains an XPath expression (usually a simple string
    // or numeric literal) which must be equal to the result of the test case
    // under the rules of the XPath 'eq' operator.
    AssertEq(XPathExpr),
    // Asserts that the result must be a sequence containing a given number of
    // items. The value of the element is an integer giving the expected length
    // of the sequence.
    AssertCount(usize),
    // Asserts that the result must be a sequence of atomic values that is
    // deep-equal to the supplied sequence under the rules of the deep-equal()
    // function.
    AssertDeepEq(XPathExpr),
    //  Asserts that the result must be a sequence of atomic values that has
    //  some permutation (reordering) that is deep-equal to the supplied
    //  sequence under the rules of the deep-equal() function.
    // Note this implies that NaN is equal to NaN.
    AssertPermutation(XPathExpr),
    // Asserts the result of the query by providing a serialization of the
    // expression result using the default serialization parameters
    // method="xml" indent="no" omit-xml-declaration="yes".
    AssertXml(String),
    //  Asserts that the result of the test is an empty sequence.
    AssertEmpty,
    // Asserts the result of serializing the query matches a given regular
    // expression.
    // XXX values not right
    SerializationMatches,
    // Asserts that the query can be executed without error, but serializing
    // the result produces a serialization error. The result of the query must
    // be serialized using the serialization options specified within the query
    // (if any).
    AssertSerializationError(String),
    // Asserts that the result of the test matches the sequence type given as
    // the value of the assert-type element.
    AssertType(String),
    // Asserts that the result of the test is the singleton boolean value
    // false(). Note, the test expression must actually evaluate to false: this
    // is not an assertion on the effective boolean value.
    AssertTrue,
    // Asserts that the result of the test is the singleton boolean value
    // false(). Note, the test expression must actually evaluate to false: this
    // is not an assertion on the effective boolean value.
    AssertFalse,
    // Asserts that the result of the test, after conversion to a string by
    // applying the expression string-join(for $r in $result return string($r),
    // " ") is equal to the string value of the assert-string-value element.
    // Note that this test cannot be used if the result includes items that do
    // not have a string value (elements with element-only content; function
    // items) If the normalize-space attribute is present with the value true,
    // then both the string value of the query result and the value of the
    // assert-string-value element should be processed as if by the XPath
    // normalize-space() function before the comparison.
    AssertStringValue(String),
    //  Asserts that the test is expected to fail with a static or dynamic
    //  error condition. The "code" attribute gives the expected error code.
    //
    // For the purpose of official test reporting, an implementation is
    // considered to pass a test if the test expects and error and the
    // implementation raises an error, regardless whether the error codes
    // match.
    Error(String),
    // This assertion type is as of yet unsupported, and will automatically error
    Unsupported,
}

#[derive(Debug)]
pub(crate) enum TestCaseEnvironment {
    Local(Box<EnvironmentSpec>),
    Ref(EnvironmentRef),
}

#[derive(Debug)]
pub(crate) struct EnvironmentRef {
    pub(crate) ref_: String,
}

impl Display for EnvironmentRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ref_)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct EnvironmentSpec {
    pub(crate) base_dir: PathBuf,
    pub(crate) schemas: Vec<Schema>,
    pub(crate) sources: Vec<Source>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) params: Vec<Param>,
    pub(crate) context_items: Vec<ContextItem>,
    pub(crate) decimal_formats: Vec<DecimalFormat>,
    pub(crate) namespaces: Vec<Namespace>,
    pub(crate) function_libraries: Vec<FunctionLibrary>,
    pub(crate) collections: Vec<Collection>,
    pub(crate) static_base_uris: Vec<StaticBaseUri>,
    pub(crate) collations: Vec<Collation>,
}

impl EnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Schema {}

#[derive(Debug, Clone)]
pub(crate) enum SourceRole {
    Context,
    Var(String),
    Doc(String), // URI
}

#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    pub(crate) role: SourceRole,
    pub(crate) file: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct Resource {}

#[derive(Debug, Clone)]
pub(crate) struct Param {}

#[derive(Debug, Clone)]
pub(crate) struct ContextItem {}

#[derive(Debug, Clone)]
pub(crate) struct DecimalFormat {}

#[derive(Debug, Clone)]
pub(crate) struct Namespace {}

#[derive(Debug, Clone)]
pub(crate) struct FunctionLibrary {}

#[derive(Debug, Clone)]
pub(crate) struct Collection {}

#[derive(Debug, Clone)]
pub(crate) struct StaticBaseUri {}

#[derive(Debug, Clone)]
pub(crate) struct Collation {}

#[derive(Debug)]
pub(crate) struct Dependency {
    pub(crate) spec: DependencySpec,
    pub(crate) satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DependencySpec {
    pub(crate) type_: String,
    pub(crate) value: String,
}

#[derive(Debug)]
pub(crate) struct Module;

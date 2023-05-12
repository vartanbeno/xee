#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) created: Attribution,
    pub(crate) modified: Vec<Modification>,
    pub(crate) environments: Vec<Environment>,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) modules: Vec<Module>,
    pub(crate) test: String,
    pub(crate) result: TestCaseResult,
}

#[derive(Debug)]
pub(crate) struct Attribution {
    pub(crate) by: String,
    pub(crate) on: String, // should be a date
}

#[derive(Debug)]
pub(crate) struct Modification {
    pub(crate) attribution: Attribution,
    pub(crate) description: String,
}

#[derive(Debug)]
pub(crate) enum TestCaseResult {
    AnyOf(Vec<TestCaseResult>),
    AllOf(Vec<TestCaseResult>),
    Not(Box<TestCaseResult>),
    Assert(String),
    AssertEq(String),
    AssertCount(usize),
    AssertDeepEq(String),
    AssertPermutation(String),
    AssertXml(String),
    AssertEmpty,
    AssertTrue,
    AssertFalse,
    AssertStringValue(String),
    Error(String),
}

#[derive(Debug)]
pub(crate) struct Environment {
    name: Option<String>,
    ref_: Option<String>,
    schemas: Vec<Schema>,
    sources: Vec<Source>,
    resources: Vec<Resource>,
    params: Vec<Param>,
    context_items: Vec<ContextItem>,
    decimal_formats: Vec<DecimalFormat>,
    namespaces: Vec<Namespace>,
    function_libraries: Vec<FunctionLibrary>,
    collections: Vec<Collection>,
    static_base_uris: Vec<StaticBaseUri>,
    collations: Vec<Collation>,
}

#[derive(Debug)]
pub(crate) struct Schema {}

#[derive(Debug)]
pub(crate) struct Source {
    role: String,
    file: String, // PathBuf?
    description: Option<String>,
    created: Option<Attribution>,
    modified: Vec<Modification>,
}

#[derive(Debug)]
pub(crate) struct Resource {}

#[derive(Debug)]
pub(crate) struct Param {}

#[derive(Debug)]
pub(crate) struct ContextItem {}

#[derive(Debug)]
pub(crate) struct DecimalFormat {}

#[derive(Debug)]
pub(crate) struct Namespace {}

#[derive(Debug)]
pub(crate) struct FunctionLibrary {}

#[derive(Debug)]
pub(crate) struct Collection {}

#[derive(Debug)]
pub(crate) struct StaticBaseUri {}

#[derive(Debug)]
pub(crate) struct Collation {}

#[derive(Debug)]
pub(crate) struct Dependency {
    pub(crate) type_: String,
    pub(crate) value: String,
    pub(crate) satisfied: bool,
}

#[derive(Debug)]
pub(crate) struct Module;

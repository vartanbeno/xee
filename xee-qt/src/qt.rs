use std::path::PathBuf;

use ahash::HashMap;

#[derive(Debug)]
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
pub(crate) enum TestCaseEnvironment {
    Local(LocalEnvironment),
    Ref(EnvironmentRef),
}

#[derive(Debug)]
pub(crate) struct EnvironmentRef {
    pub(crate) ref_: String,
}

#[derive(Debug)]
pub(crate) struct SharedEnvironments {
    environments: HashMap<String, EnvironmentSpec>,
}

impl SharedEnvironments {
    pub(crate) fn new(mut environments: HashMap<String, EnvironmentSpec>) -> Self {
        // there is always an empty environment
        if !environments.contains_key("empty") {
            let empty = EnvironmentSpec::empty();
            environments.insert("empty".to_string(), empty);
        }
        Self { environments }
    }
}

#[derive(Debug)]
pub(crate) struct LocalEnvironment {
    pub(crate) spec: EnvironmentSpec,
}

#[derive(Debug, Default)]
pub(crate) struct EnvironmentSpec {
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

#[derive(Debug)]
pub(crate) struct Schema {}

#[derive(Debug)]
pub(crate) enum SourceRole {
    Context,
    Var(String),
    Doc(String), // URI
}

#[derive(Debug)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    pub(crate) role: SourceRole,
    pub(crate) file: PathBuf,
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

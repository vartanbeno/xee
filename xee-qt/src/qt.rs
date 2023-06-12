use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

pub(crate) use crate::assert::TestCaseResult;
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

#[derive(Debug, PartialEq)]
pub(crate) struct XPathExpr(pub(crate) String);

#[derive(Debug)]
pub(crate) enum TestCaseEnvironment {
    Local(Box<EnvironmentSpec>),
    Ref(EnvironmentRef),
}

#[derive(Debug, Clone)]
pub struct EnvironmentRef {
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

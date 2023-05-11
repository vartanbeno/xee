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
    AssertStringValue,
    Error(String),
}

#[derive(Debug)]
pub(crate) struct Environment;

#[derive(Debug)]
pub(crate) struct Dependency {
    type_: String,
    value: String,
}

#[derive(Debug)]
pub(crate) struct Module;

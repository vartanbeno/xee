#[derive(Debug)]
struct TestCase {
    name: String,
    description: String,
    created_by: String,
    created_on: String, // should be a date
    environments: Vec<Environment>,
    dependencies: Vec<Dependency>,
    modules: Vec<Module>,
    test: String,
    result: TestCaseResult,
}

#[derive(Debug)]
enum TestCaseResult {
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
struct Environment;

#[derive(Debug)]
struct Dependency;

#[derive(Debug)]
struct Module;

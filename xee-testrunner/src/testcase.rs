use crate::{
    environment::{Environment, TestCaseEnvironment},
    metadata::Metadata,
};

#[derive(Debug)]
pub(crate) struct TestCase<E: Environment> {
    pub(crate) name: String,
    pub(crate) metadata: Metadata,
    // environments can be a reference by name, or a locally defined environment
    pub(crate) environments: Vec<TestCaseEnvironment<E>>,
    // pub(crate) dependencies: Dependencies,
    // pub(crate) modules: Vec<Module>,
    // pub(crate) test: String,
    // pub(crate) result: TestCaseResult,
}

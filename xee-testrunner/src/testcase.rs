use crate::{
    catalog::Catalog,
    environment::{Environment, EnvironmentSpecIterator, TestCaseEnvironment},
    metadata::Metadata,
    testset::TestSet,
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

impl<E: Environment> TestCase<E> {
    fn environment_specs<'a>(
        &'a self,
        catalog: &'a Catalog<E>,
        test_set: &'a TestSet<E>,
    ) -> EnvironmentSpecIterator<'a, E> {
        EnvironmentSpecIterator::new(
            vec![&catalog.shared_environments, &test_set.shared_environments],
            &self.environments,
        )
    }
}

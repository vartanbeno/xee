use xee_xpath::{context::Variables, sequence, xml::Documents};

use crate::{
    catalog::Catalog,
    dependency::Dependencies,
    environment::{Environment, EnvironmentIterator, TestCaseEnvironment},
    error::Result,
    metadata::Metadata,
    outcome::TestOutcome,
    runcontext::RunContext,
    testset::TestSet,
};

pub(crate) trait Runnable<E: Environment>: std::marker::Sized {
    fn run(&self, run_context: &mut RunContext<E>, test_set: &TestSet<Self, E>) -> TestOutcome;
}

#[derive(Debug)]
pub(crate) struct TestCase<E: Environment> {
    pub(crate) name: String,
    pub(crate) metadata: Metadata,
    // environments can be a reference by name, or a locally defined environment
    pub(crate) environments: Vec<TestCaseEnvironment<E>>,
    pub(crate) dependencies: Dependencies,
    // pub(crate) modules: Vec<Module>,
    // pub(crate) test: String,
    // pub(crate) result: TestCaseResult,
}

impl<E: Environment> TestCase<E> {
    pub(crate) fn environments<'a, R: Runnable<E>>(
        &'a self,
        catalog: &'a Catalog<E>,
        test_set: &'a TestSet<R, E>,
    ) -> EnvironmentIterator<'a, E> {
        EnvironmentIterator::new(
            vec![&catalog.shared_environments, &test_set.shared_environments],
            &self.environments,
        )
    }

    pub(crate) fn context_item<R: Runnable<E>>(
        &self,
        run_context: &mut RunContext<E>,
        test_set: &TestSet<R, E>,
    ) -> Result<Option<sequence::Item>> {
        let environments = self
            .environments(&run_context.catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let xot = &mut run_context.xot;
        let documents = &mut run_context.documents;
        for environment in environments {
            let item = environment
                .environment_spec()
                .context_item(xot, documents)?;
            if let Some(item) = item {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    pub(crate) fn variables<R: Runnable<E>>(
        &self,
        run_context: &mut RunContext<E>,
        test_set: &TestSet<R, E>,
    ) -> Result<Variables> {
        let environments = self
            .environments(&run_context.catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let mut variables = Variables::new();
        let xot = &mut run_context.xot;
        let source_cache = &mut run_context.documents;
        for environment in environments {
            variables.extend(
                environment
                    .environment_spec()
                    .variables(xot, source_cache)?,
            );
        }
        Ok(variables)
    }
}

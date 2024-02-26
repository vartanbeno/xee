use xee_xpath::{context::Variables, sequence};

use crate::{
    catalog::Catalog,
    dependency::Dependencies,
    environment::{Environment, EnvironmentIterator, TestCaseEnvironment},
    error::Result,
    metadata::Metadata,
    runcontext::RunContext,
    testset::TestSet,
};

use super::outcome::TestOutcome;

pub(crate) trait Runnable<E: Environment>: std::marker::Sized {
    fn test_case(&self) -> &TestCase<E>;

    fn run(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, Self>,
        test_set: &TestSet<E, Self>,
    ) -> TestOutcome;
}

#[derive(Debug)]
pub(crate) struct TestCase<E: Environment> {
    pub(crate) name: String,
    pub(crate) metadata: Metadata,
    // environments can be a reference by name, or a locally defined environment
    pub(crate) environments: Vec<TestCaseEnvironment<E>>,
    pub(crate) dependencies: Dependencies,
    // pub(crate) modules: Vec<Module>,
}

impl<E: Environment> TestCase<E> {
    pub(crate) fn environments<'a, R: Runnable<E>>(
        &'a self,
        catalog: &'a Catalog<E, R>,
        test_set: &'a TestSet<E, R>,
    ) -> EnvironmentIterator<'a, E> {
        EnvironmentIterator::new(
            vec![&catalog.shared_environments, &test_set.shared_environments],
            &self.environments,
        )
    }

    pub(crate) fn context_item<R: Runnable<E>>(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> Result<Option<sequence::Item>> {
        let environments = self
            .environments(&catalog, test_set)
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
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> Result<Variables> {
        let environments = self
            .environments(catalog, test_set)
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

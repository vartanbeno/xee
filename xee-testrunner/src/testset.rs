use std::{
    fs::File,
    io::{BufReader, Read, Stdout},
    path::{Path, PathBuf},
};

use xee_name::Namespaces;
use xee_xpath::{
    context::{DynamicContext, StaticContext},
    sequence::Item,
    Queries, Query,
};
use xot::Xot;

use crate::{
    catalog::Catalog,
    dependency::{Dependencies, Dependency},
    environment::{Environment, SharedEnvironments, XPathEnvironmentSpec},
    error::Result,
    filter::TestFilter,
    load::{convert_string, ContextLoadable, XPATH_NS},
    outcomes::TestSetOutcomes,
    renderer::Renderer,
    runcontext::RunContext,
    testcase::{Runnable, TestCase, XPathTestCase},
};

#[derive(Debug)]
pub(crate) struct TestSet<E: Environment, R: Runnable<E>> {
    pub(crate) full_path: PathBuf,
    pub(crate) name: String,
    pub(crate) descriptions: Vec<String>,
    pub(crate) dependencies: Dependencies,
    pub(crate) shared_environments: SharedEnvironments<E>,
    pub(crate) test_cases: Vec<R>,
}

impl<E: Environment, R: Runnable<E>> TestSet<E, R> {
    fn base_dir(&self) -> &Path {
        self.full_path.parent().unwrap()
    }

    pub(crate) fn file_path(&self, catalog: &Catalog<E, R>) -> &Path {
        self.full_path.strip_prefix(catalog.base_dir()).unwrap()
    }

    pub(crate) fn run<Ren: Renderer<E, R>>(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_filter: &impl TestFilter<E, R>,
        stdout: &mut Stdout,
        renderer: &Ren,
    ) -> Result<TestSetOutcomes> {
        renderer.render_test_set(stdout, self, catalog)?;

        let mut test_set_outcomes = TestSetOutcomes::new(&self.name);
        for runner in &self.test_cases {
            let test_case = runner.test_case();
            if !test_filter.is_included(self, test_case) {
                test_set_outcomes.add_filtered();
                continue;
            }
            // skip any test case we don't support, either on test set or
            // test case level
            if !self
                .dependencies
                .is_supported(&run_context.known_dependencies)
                || !test_case
                    .dependencies
                    .is_supported(&run_context.known_dependencies)
            {
                test_set_outcomes.add_unsupported();
                continue;
            }
            renderer.render_test_case(stdout, test_case)?;
            let outcome = runner.run(run_context, catalog, self);
            renderer.render_test_outcome(stdout, &outcome)?;
            test_set_outcomes.add_outcome(&test_case.name, outcome);
        }
        renderer.render_test_set_summary(stdout, self)?;
        Ok(test_set_outcomes)
    }
}

impl<E: Environment, R: Runnable<E>> ContextLoadable<Path> for TestSet<E, R> {
    fn query_with_context<'a>(
        mut queries: Queries<'a>,
        path: &'a Path,
    ) -> Result<(Queries<'a>, impl Query<TestSet<E, R>> + 'a)>
    where
        E: 'a,
        R: 'a,
    {
        let name_query = queries.one("@name/string()", convert_string)?;
        let descriptions_query = queries.many("description/string()", convert_string)?;

        let (queries, shared_environments_query) =
            SharedEnvironments::query_with_context(queries, path)?;
        let (queries, dependency_query) = Dependency::query(queries)?;
        let (mut queries, test_case_query) = R::query(queries, path)?;
        let test_cases_query = queries.many("test-case", move |session, item| {
            test_case_query.execute(session, item)
        })?;
        let test_set_query = queries.one("/test-set", move |session, item| {
            let name = name_query.execute(session, item)?;
            let descriptions = descriptions_query.execute(session, item)?;
            let dependencies = dependency_query.execute(session, item)?;
            let shared_environments = shared_environments_query.execute(session, item)?;
            let test_cases = test_cases_query.execute(session, item)?;
            Ok(TestSet {
                full_path: path.to_path_buf(),
                name,
                descriptions,
                dependencies: Dependencies::new(dependencies.into_iter().flatten().collect()),
                shared_environments,
                test_cases,
            })
        })?;
        Ok((queries, test_set_query))
    }
}

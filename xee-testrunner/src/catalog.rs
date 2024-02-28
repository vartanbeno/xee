use std::fs::File;
use std::io::{BufReader, Read, Stdout};
use std::path::{Path, PathBuf};

use xee_name::Namespaces;
use xee_xpath::context::{DynamicContext, StaticContext};
use xee_xpath::sequence::Item;
use xee_xpath::{Queries, Query};
use xot::Xot;

use crate::environment::{Environment, SharedEnvironments, XPathEnvironmentSpec};
use crate::error::Result;
use crate::filter::TestFilter;
use crate::hashmap::FxIndexSet;
use crate::load::{convert_string, ContextLoadable, PathLoadable, XPATH_NS};
use crate::outcomes::CatalogOutcomes;
use crate::renderer::Renderer;
use crate::runcontext::RunContext;
use crate::testcase::{Runnable, XPathTestCase};
use crate::testset::TestSet;

#[derive(Debug)]
pub(crate) struct TestSetRef {
    pub(crate) name: String,
    pub(crate) file: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Catalog<E: Environment, R: Runnable<E>> {
    pub(crate) shared_environments: SharedEnvironments<E>,
    pub(crate) full_path: PathBuf,
    pub(crate) test_suite: String,
    pub(crate) version: String,
    pub(crate) test_sets: Vec<TestSetRef>,
    pub(crate) file_paths: FxIndexSet<PathBuf>,
    _runnable: std::marker::PhantomData<R>,
}

impl<E: Environment, R: Runnable<E>> Catalog<E, R> {
    pub(crate) fn base_dir(&self) -> &Path {
        self.full_path.parent().unwrap()
    }

    pub(crate) fn run(
        &self,
        run_context: &mut RunContext,
        test_filter: &impl TestFilter<E, R>,
        out: &mut Stdout,
        renderer: &dyn Renderer<E, R>,
    ) -> crate::error::Result<CatalogOutcomes> {
        let mut catalog_outcomes = CatalogOutcomes::new();
        for file_path in &self.file_paths {
            let full_path = self.base_dir().join(file_path);
            let test_set = TestSet::load_from_file(run_context, &full_path)?;
            let test_set_outcomes = test_set.run(run_context, self, test_filter, out, renderer)?;
            catalog_outcomes.add_outcomes(test_set_outcomes);
        }
        Ok(catalog_outcomes)
    }
}

impl<E: Environment, R: Runnable<E>> ContextLoadable<Path> for Catalog<E, R> {
    fn query_with_context<'a>(
        mut queries: Queries<'a>,
        path: &'a Path,
    ) -> Result<(Queries<'a>, impl Query<Catalog<E, R>> + 'a)>
    where
        E: 'a,
        R: 'a,
    {
        let test_suite_query = queries.one("@test-suite/string()", convert_string)?;
        let version_query = queries.one("@version/string()", convert_string)?;

        let (mut queries, shared_environments_query) =
            SharedEnvironments::query_with_context(queries, path)?;

        let test_set_name_query = queries.one("@name/string()", convert_string)?;
        let test_set_file_query = queries.one("@file/string()", convert_string)?;
        let test_set_query = queries.many("test-set", move |session, item| {
            let name = test_set_name_query.execute(session, item)?;
            let file = PathBuf::from(test_set_file_query.execute(session, item)?);
            Ok(TestSetRef { name, file })
        })?;
        let catalog_query = queries.one("catalog", move |session, item| {
            let test_suite = test_suite_query.execute(session, item)?;
            let version = version_query.execute(session, item)?;
            let shared_environments = shared_environments_query.execute(session, item)?;
            let test_sets = test_set_query.execute(session, item)?;
            let file_paths = test_sets.iter().map(|t| t.file.clone()).collect();
            Ok(Catalog {
                full_path: path.to_path_buf(),
                test_suite,
                version,
                shared_environments,
                test_sets,
                file_paths,
                _runnable: std::marker::PhantomData,
            })
        })?;
        Ok((queries, catalog_query))
    }
}

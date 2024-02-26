use std::io::{stdout, Stdout};
use std::path::{Path, PathBuf};

use crate::environment::{Environment, SharedEnvironments};
use crate::filter::TestFilter;
use crate::hashmap::FxIndexSet;
use crate::outcomes::CatalogOutcomes;
use crate::renderer::Renderer;
use crate::runcontext::RunContext;
use crate::testcase::Runnable;

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

    pub(crate) fn run<Ren: Renderer<E, R>>(
        &self,
        run_context: &mut RunContext,
        test_filter: &impl TestFilter<E, R>,
        stdout: &mut Stdout,
        renderer: Ren,
    ) -> crate::error::Result<CatalogOutcomes> {
        let mut catalog_outcomes = CatalogOutcomes::new();

        for file_path in &self.file_paths {
            // let test_set_outcomes =

            //     run_path_helper(run_context, test_filter, file_path, &mut stdout)?;
            // catalog_outcomes.add_outcomes(test_set_outcomes);
        }
        Ok(catalog_outcomes)
    }
}

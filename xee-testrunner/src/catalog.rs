use std::path::PathBuf;

use crate::environment::{Environment, SharedEnvironments};
use crate::hashmap::FxIndexSet;

#[derive(Debug)]
pub(crate) struct TestSetRef {
    pub(crate) name: String,
    pub(crate) file: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Catalog<E: Environment> {
    pub(crate) shared_environments: SharedEnvironments<E>,
    pub(crate) full_path: PathBuf,
    pub(crate) test_suite: String,
    pub(crate) version: String,
    pub(crate) test_sets: Vec<TestSetRef>,
    pub(crate) file_paths: FxIndexSet<PathBuf>,
}

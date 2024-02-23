use std::path::PathBuf;

use crate::{
    dependency::Dependencies,
    environment::{Environment, SharedEnvironments},
    testcase::TestCase,
};

#[derive(Debug)]
pub(crate) struct TestSet<E: Environment> {
    pub(crate) full_path: PathBuf,
    pub(crate) name: String,
    pub(crate) descriptions: Vec<String>,
    pub(crate) dependencies: Dependencies,
    pub(crate) shared_environments: SharedEnvironments<E>,
    pub(crate) test_cases: Vec<TestCase<E>>,
}

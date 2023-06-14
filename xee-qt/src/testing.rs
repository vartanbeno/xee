use std::env;
use std::path::PathBuf;
use xot::Xot;

use crate::assert::TestOutcomes;
use crate::error::{Error, Result};
use crate::{path::paths, qt, run::RunContextBuilder};

fn try_test_all(path: &str) -> Result<()> {
    // This environment variable only exists because of the .cargo/config.toml
    // hack described here https://github.com/rust-lang/cargo/issues/3946'
    let workspace_dir = env::var("CARGO_WORKSPACE_DIR")?;
    let workspace_path = PathBuf::from(&workspace_dir);
    let qt3tests_path = workspace_path.join("vendor/qt3tests");
    let path = qt3tests_path.join(path);
    let path = path.with_extension("xml");
    let (catalog_path, relative_path) = paths(&path)?;
    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &catalog_path)?;
    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(false)
        .build()
        .unwrap();
    let full_path = run_context.catalog.base_dir().join(&relative_path);
    let test_set = qt::TestSet::load_from_file(&mut run_context.xot, &full_path)?;
    let mut outcomes = Vec::new();
    for test_case in &test_set.test_cases {
        if !test_case.is_supported(&run_context.known_dependencies) {
            continue;
        }
        let outcome = test_case.run(&mut run_context, &test_set);
        if !outcome.is_passed() {
            outcomes.push((test_case.name.clone(), outcome));
        }
    }
    if !outcomes.is_empty() {
        Err(Error::TestFailures(path, TestOutcomes(outcomes)))
    } else {
        Ok(())
    }
}

pub fn test_all(path: &str) {
    match try_test_all(path) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}

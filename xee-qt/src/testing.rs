use std::path::Path;
use xot::Xot;

use crate::error::{Error, Result};
use crate::{path::paths, qt, run::RunContextBuilder};

// fn run_all(path: &Path) -> Result<()> {
//     let path = if path.extension().is_none() {
//         // add the xml postfix
//         path.with_extension("xml")
//     } else {
//         path.to_owned()
//     };
//     let (catalog_path, relative_path) = paths(&path)?;
//     let mut xot = Xot::new();
//     let catalog = qt::Catalog::load_from_file(&mut xot, &catalog_path)?;
//     let mut run_context = RunContextBuilder::default()
//         .xot(xot)
//         .catalog(catalog)
//         .verbose(false)
//         .build()
//         .unwrap();
//     let full_path = run_context.catalog.base_dir().join(&relative_path);
//     let test_set = qt::TestSet::load_from_file(&mut run_context.xot, &full_path)?;
//     let mut outcomes = Vec::new();
//     for test_case in &test_set.test_cases {
//         if !test_case.is_supported(&run_context.known_dependencies) {
//             continue;
//         }
//         let outcome = test_case.run(&mut run_context, &test_set);
//         if !outcome.is_passed() {
//             outcomes.push(outcome);
//         }
//     }
//     if !outcomes.is_empty() {
//         Err(Error::TestFailures(full_path))
//     } else {
//         Ok(())
//     }
// }

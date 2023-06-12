use std::path::Path;

use xot::Xot;

use crate::error::Error;
use crate::{path::paths, qt, run::RunContextBuilder};

type Result<T> = std::result::Result<T, Error>;

// fn run_all(path: &Path) -> Result<()> {
//     let path = if path.extension().is_none() {
//         // add the xml postfix
//         path.with_extension("xml")
//     } else {
//         path.to_owned()
//     };
//     if path.exists() {
//         if let Some((catalog_path, relative_path)) = paths(&path) {
//             let mut xot = Xot::new();
//             let catalog = qt::Catalog::load_from_file(&mut xot, &catalog_path)?;
//             let run_context = RunContextBuilder::default()
//                 .xot(xot)
//                 .catalog(catalog)
//                 .verbose(cli.verbose)
//                 .build()
//                 .unwrap();
//         } else {
//             Err(Error {})
//         }
//         Ok(())
//     } else {
//         Err(Error {})
//     }
// }

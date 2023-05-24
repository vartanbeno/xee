#![allow(dead_code)]
mod collection;
mod environment;
mod load;
mod qt;
mod run;
mod serialize;

use clap::Parser;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::path::{Path, PathBuf};
use xot::Xot;

use crate::run::CatalogContext;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// A path to a qttests directory or individual test file
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = cli.path;

    if let Some((catalog_path, relative_path)) = paths(&path) {
        let mut xot = Xot::new();
        let catalog = qt::Catalog::load_from_file(&mut xot, &catalog_path)?;
        let catalog_context = CatalogContext::with_base_dir(xot, catalog_path.parent().unwrap());
        if relative_path.components().count() == 0 {
            catalog.run(catalog_context);
        } else {
            catalog.run_path(catalog_context, &relative_path)?;
        }
    } else {
        println!("no qttests catalog.xml found!");
    }
    Ok(())
}

fn paths(path: &Path) -> Option<(PathBuf, PathBuf)> {
    // look for a directory which contains a `catalog.xml`. This
    // is the first path buf. any remaining path components are
    // a relative path
    for ancestor in path.ancestors() {
        let catalog = ancestor.join("catalog.xml");
        if catalog.exists() {
            let relative = path.strip_prefix(ancestor).unwrap();
            return Some((catalog, relative.to_path_buf()));
        }
    }
    None
}

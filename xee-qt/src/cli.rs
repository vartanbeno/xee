use clap::Parser;
use std::path::PathBuf;
use xot::Xot;

use crate::error::Result;
use crate::filter::IncludeAllFilter;
use crate::path::paths;
use crate::qt;
use crate::run::RunContextBuilder;
use crate::ui::{run, run_path};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// A path to a qttests directory or individual test file
    path: PathBuf,
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,
}

pub fn cli() -> Result<()> {
    let cli = Cli::parse();
    let path = cli.path;

    let path_info = paths(&path)?;

    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;
    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(cli.verbose)
        .build()
        .unwrap();
    let test_filter = IncludeAllFilter::new();
    if path_info.relative_path.components().count() == 0 {
        run(&mut run_context, &test_filter)?;
    } else {
        run_path(run_context, &test_filter, &path_info.relative_path)?;
    }
    Ok(())
}

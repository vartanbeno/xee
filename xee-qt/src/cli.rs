use clap::Parser;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::path::PathBuf;
use xot::Xot;

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

    let (catalog_path, relative_path) = paths(&path)?;

    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &catalog_path)
        .into_diagnostic()
        .wrap_err("Could not load catalog")?;
    let run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(cli.verbose)
        .build()
        .unwrap();
    if relative_path.components().count() == 0 {
        run(run_context)?;
    } else {
        run_path(run_context, &relative_path)?;
    }
    Ok(())
}

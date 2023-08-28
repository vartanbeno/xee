use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use xot::Xot;

use crate::error::Result;
use crate::filter::{ExcludedNamesFilter, IncludeAllFilter};
use crate::path::paths;
use crate::qt;
use crate::run::RunContextBuilder;
use crate::ui::{run, run_path};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the filter filter.
    /// This runs all tests, and saves the test outcomes to
    /// a filter file.
    Initialize {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
    Check {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
    Update {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
    All {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
}

pub fn cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Initialize { path } => initialize(&path, cli.verbose),
        Commands::Check { path } => check(&path, cli.verbose),
        Commands::Update { path } => update(&path, cli.verbose),
        Commands::All { path } => all(&path, cli.verbose),
    }
}

fn check(path: &Path, verbose: bool) -> Result<()> {
    let path_info = paths(path)?;
    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(verbose)
        .build()
        .unwrap();

    if !path_info.filter_path.exists() {
        // we cannot check if we don't have a filter file yet
        panic!("Cannot check without filter file");
    }

    let test_filter = ExcludedNamesFilter::load_from_file(&path_info.filter_path)?;
    if path_info.whole_catalog() {
        let _ = run(&mut run_context, &test_filter)?;
    } else {
        let _ = run_path(run_context, &test_filter, &path_info.relative_path)?;
    }
    Ok(())
}

fn all(path: &Path, verbose: bool) -> Result<()> {
    let path_info = paths(path)?;
    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(verbose)
        .build()
        .unwrap();

    let test_filter = IncludeAllFilter::new();

    if path_info.whole_catalog() {
        let _ = run(&mut run_context, &test_filter)?;
    } else {
        let _ = run_path(run_context, &test_filter, &path_info.relative_path)?;
    }
    Ok(())
}

fn update(path: &Path, verbose: bool) -> Result<()> {
    let path_info = paths(path)?;
    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(verbose)
        .build()
        .unwrap();

    if !path_info.filter_path.exists() {
        // we cannot update if we don't have a filter file yet
        panic!("Cannot update without filter file");
    }
    let test_filter = IncludeAllFilter::new();
    let mut update_filter = ExcludedNamesFilter::load_from_file(&path_info.filter_path)?;
    if path_info.whole_catalog() {
        let catalog_outcomes = run(&mut run_context, &test_filter)?;

        update_filter.update_with_catalog_outcomes(&catalog_outcomes);
    } else {
        let test_set_outcomes = run_path(run_context, &test_filter, &path_info.relative_path)?;
        update_filter.update_with_test_set_outcomes(&test_set_outcomes);
    }

    let filter_data = update_filter.to_string();
    fs::write(&path_info.filter_path, filter_data)?;
    Ok(())
}

fn initialize(path: &Path, verbose: bool) -> Result<()> {
    let path_info = paths(path)?;
    if path_info.filter_path.exists() {
        panic!("Cannot reinitialize filters. Use update or delete filters file first");
    }

    let mut xot = Xot::new();
    let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

    let mut run_context = RunContextBuilder::default()
        .xot(xot)
        .catalog(catalog)
        .verbose(verbose)
        .build()
        .unwrap();

    let test_filter = IncludeAllFilter::new();

    let catalog_outcomes = run(&mut run_context, &test_filter)?;

    let test_filter = ExcludedNamesFilter::from_outcomes(&catalog_outcomes);
    let filter_data = test_filter.to_string();
    fs::write(&path_info.filter_path, filter_data)?;
    Ok(())
}

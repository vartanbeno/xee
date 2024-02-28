use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use xee_xpath::xml::Documents;
use xot::Xot;

use crate::catalog::Catalog;
use crate::dependency::xpath_known_dependencies;
use crate::environment::{Environment, XPathEnvironmentSpec};
use crate::error::Result;
use crate::filter::{ExcludedNamesFilter, IncludeAllFilter, NameFilter};
use crate::load::{PathLoadable, XPATH_NS};
use crate::outcomes::Outcomes;
use crate::paths::{paths, PathInfo};
use crate::renderer::{CharacterRenderer, VerboseRenderer};
use crate::runcontext::RunContext;
use crate::testcase::{Runnable, XPathTestCase};
use crate::testset::TestSet;

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
    /// Initialize the filter.
    ///
    /// Runs all tests, and saves the test outcomes to a `filters` file in the
    /// directory of the `catalog.xml`.
    Initialize {
        /// A path to a qttests directory or individual test file
        path: PathBuf,
    },
    /// Check with filters engaged.
    ///
    /// Runs tests that are not excluded by the `filters` file. This can be
    /// used to check for regressions after making changes.
    Check {
        /// A path to a qttests directory or individual test file.
        ///
        /// If individual test file, it runs only the tests in that file,
        /// otherwise it runs the tests in the `catalog.xml`.
        path: PathBuf,
    },
    /// Update the filters.
    ///
    /// This runs all the tests on the path, and then updates the `filters`
    /// file if more tests pass in this run. If not, the filters file is not
    /// updated, so that you can't accidentally introduce a regression.
    Update {
        /// A path to a qttests directory or individual test file.
        ///
        /// If individual test file, it updates only the tests in that file,
        /// otherwise it updates tests in the `catalog.xml`.
        path: PathBuf,
    },
    /// Run all tests.
    ///
    /// Do not use filters, simply run all the tests indicated by path.
    All {
        /// A path to a qttests directory or individual test file.
        path: PathBuf,
        /// Name filter, only test-cases that contain this name are found.
        name_filter: Option<String>,
    },
}

impl Commands {
    fn path(&self) -> &Path {
        match self {
            Commands::Initialize { path } => path,
            Commands::Check { path } => path,
            Commands::Update { path } => path,
            Commands::All { path, .. } => path,
        }
    }
}

pub fn cli() -> Result<()> {
    let cli = Cli::parse();

    let path = cli.command.path();
    let path_info = paths(path)?;

    let xot = Xot::new();
    let run_context = RunContext::new(
        xot,
        Documents::new(),
        xpath_known_dependencies(),
        XPATH_NS.to_string(),
        cli.verbose,
    );

    match cli.command {
        // Commands::Initialize { path } => initialize(&path, run_context),
        Commands::Check { .. } => {
            check::<XPathEnvironmentSpec, XPathTestCase>(run_context, &path_info)
        }
        // Commands::Update { path } => update(&path, verbose),
        // Commands::All { path, name_filter } => all(&path, verbose, name_filter),
        _ => todo!(),
    }
}

fn check<E: Environment, R: Runnable<E>>(
    mut run_context: RunContext,
    path_info: &PathInfo,
) -> Result<()> {
    if !path_info.filter_path.exists() {
        // we cannot check if we don't have a filter file yet
        println!("Cannot check without filter file");
        return Ok(());
    }
    let catalog = Catalog::<E, R>::load_from_file(&mut run_context, &path_info.catalog_path)?;

    let test_filter = ExcludedNamesFilter::load_from_file(&path_info.filter_path)?;
    let mut out = std::io::stdout();

    let renderer = run_context.renderer();
    if path_info.whole_catalog() {
        let outcomes = catalog.run(&mut run_context, &test_filter, &mut out, &renderer)?;
        println!("{}", outcomes.display());
    } else {
        let test_set = TestSet::load_from_file(&mut run_context, &path_info.relative_path)?;
        let outcomes = test_set.run(
            &mut run_context,
            &catalog,
            &test_filter,
            &mut out,
            &renderer,
        )?;
        println!("{}", outcomes.display());
    }
    Ok(())
}

// fn all(path: &Path, verbose: bool, name_filter: Option<String>) -> Result<()> {
//     let path_info = paths(path)?;
//     let mut xot = Xot::new();
//     let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

//     let mut run_context = RunContextBuilder::default()
//         .xot(xot)
//         .catalog(catalog)
//         .verbose(verbose)
//         .build()
//         .unwrap();

//     let test_filter = NameFilter::new(name_filter);

//     if path_info.whole_catalog() {
//         let outcomes = run(&mut run_context, &test_filter)?;
//         println!("{}", outcomes.display());
//     } else {
//         let outcomes = run_path(run_context, &test_filter, &path_info.relative_path)?;
//         println!("{}", outcomes.display());
//     }
//     Ok(())
// }

// fn update(path: &Path, verbose: bool) -> Result<()> {
//     let path_info = paths(path)?;
//     let mut xot = Xot::new();
//     let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

//     let mut run_context = RunContextBuilder::default()
//         .xot(xot)
//         .catalog(catalog)
//         .verbose(verbose)
//         .build()
//         .unwrap();

//     if !path_info.filter_path.exists() {
//         // we cannot update if we don't have a filter file yet
//         println!("Cannot update without filter file");
//         return Ok(());
//     }
//     let test_filter = IncludeAllFilter::new();
//     let mut update_filter = ExcludedNamesFilter::load_from_file(&path_info.filter_path)?;
//     if path_info.whole_catalog() {
//         let catalog_outcomes = run(&mut run_context, &test_filter)?;

//         update_filter.update_with_catalog_outcomes(&catalog_outcomes);
//         println!("{}", catalog_outcomes.display());
//     } else {
//         let test_set_outcomes = run_path(run_context, &test_filter, &path_info.relative_path)?;
//         update_filter.update_with_test_set_outcomes(&test_set_outcomes);
//         println!("{}", test_set_outcomes.display());
//     }

//     let filter_data = update_filter.to_string();
//     fs::write(&path_info.filter_path, filter_data)?;
//     Ok(())
// }

// fn initialize(path: &Path, verbose: bool) -> Result<()> {
//     let path_info = paths(path)?;
//     if path_info.filter_path.exists() {
//         println!("Cannot reinitialize filters. Use update or delete filters file first");
//         return Ok(());
//     }

//     let mut xot = Xot::new();
//     let catalog = qt::Catalog::load_from_file(&mut xot, &path_info.catalog_path)?;

//     let mut run_context = RunContextBuilder::default()
//         .xot(xot)
//         .catalog(catalog)
//         .verbose(verbose)
//         .build()
//         .unwrap();

//     let test_filter = IncludeAllFilter::new();

//     let catalog_outcomes = run(&mut run_context, &test_filter)?;

//     let test_filter = ExcludedNamesFilter::from_outcomes(&catalog_outcomes);
//     let filter_data = test_filter.to_string();
//     fs::write(&path_info.filter_path, filter_data)?;
//     Ok(())
// }

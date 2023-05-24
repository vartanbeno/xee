use crossterm::{
    execute,
    style::{self, Stylize},
};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::io::{stdout, Stdout};
use std::path::Path;

use crate::qt;
use crate::run::{CatalogContext, TestResult, TestSetContext};

pub(crate) fn run(catalog: &qt::Catalog, mut catalog_context: CatalogContext) -> Result<()> {
    let mut stdout = stdout();
    for file_path in &catalog.file_paths {
        run_path_helper(catalog, &mut catalog_context, file_path, &mut stdout)?
    }
    Ok(())
}

pub(crate) fn run_path(
    catalog: &qt::Catalog,
    mut catalog_context: CatalogContext,
    path: &Path,
) -> Result<()> {
    let mut stdout = stdout();
    run_path_helper(catalog, &mut catalog_context, path, &mut stdout)
}

fn run_path_helper(
    catalog: &qt::Catalog,
    catalog_context: &mut CatalogContext,
    path: &Path,
    stdout: &mut Stdout,
) -> Result<()> {
    if !catalog.file_paths.contains(path) {
        miette!("File not found in catalog: {:?}", path);
    }
    let full_path = catalog_context.base_dir.join(path);
    let test_set = qt::TestSet::load_from_file(&mut catalog_context.xot, &full_path)?;
    let test_set_context = TestSetContext::with_file_path(catalog_context, path);
    run_test_set(&test_set, test_set_context, stdout)?;
    Ok(())
}

fn run_test_set(
    test_set: &qt::TestSet,
    mut test_set_context: TestSetContext,
    stdout: &mut Stdout,
) -> Result<()> {
    print!("{} ", test_set_context.file_path.display());
    for test_case in &test_set.test_cases {
        let result = test_case.run(&mut test_set_context, &test_set.shared_environments);
        match result {
            Ok(test_result) => {
                render_test_result_character(stdout, &test_result).into_diagnostic()?;
            }
            Err(_) => {
                render_test_crashed_character(stdout).into_diagnostic()?;
            }
        }
    }
    println!();
    Ok(())
}

fn render_test_result_character(
    stdout: &mut Stdout,
    test_result: &TestResult,
) -> crossterm::Result<()> {
    match test_result {
        TestResult::Passed => {
            execute!(stdout, style::PrintStyledContent(".".green()))?;
        }
        TestResult::PassedWithWrongError(_) => {
            execute!(stdout, style::PrintStyledContent(".".green()))?;
        }
        TestResult::Failed(_) => {
            execute!(stdout, style::PrintStyledContent("F".red()))?;
        }
        TestResult::RuntimeError(_) => {
            execute!(stdout, style::PrintStyledContent("E".red()))?;
        }
        TestResult::CompilationError(_) => {
            execute!(stdout, style::PrintStyledContent("E".red()))?;
        }
        TestResult::UnsupportedExpression(_) => {
            execute!(stdout, style::PrintStyledContent("E".red()))?;
        }
        TestResult::Unsupported => {
            execute!(stdout, style::PrintStyledContent("E".red()))?;
        }
        TestResult::UnsupportedDependency => {
            // do not show any output as this is skipped
        }
    }
    Ok(())
}

fn render_test_crashed_character(stdout: &mut Stdout) -> crossterm::Result<()> {
    execute!(stdout, style::PrintStyledContent("C".red()))
}

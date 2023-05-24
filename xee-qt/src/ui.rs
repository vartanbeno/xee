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
    let verbose = catalog_context.verbose;
    let full_path = catalog_context.base_dir.join(path);
    let test_set = qt::TestSet::load_from_file(
        &mut catalog_context.xot,
        &catalog_context.base_dir,
        &full_path,
    )?;
    let test_set_context = TestSetContext::with_file_path(catalog_context, path);
    if verbose {
        run_test_set(&test_set, test_set_context, stdout, VerboseRenderer::new())?;
    } else {
        run_test_set(
            &test_set,
            test_set_context,
            stdout,
            CharacterRenderer::new(),
        )?;
    }
    Ok(())
}

trait Renderer {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
        test_set_context: &TestSetContext,
    ) -> crossterm::Result<()>;
    fn render_test_case(
        &self,
        stdout: &mut Stdout,
        test_case: &qt::TestCase,
    ) -> crossterm::Result<()>;
    fn render_test_result(
        &self,
        stdout: &mut Stdout,
        test_result: &TestResult,
    ) -> crossterm::Result<()>;
    fn render_test_set_summary(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
    ) -> crossterm::Result<()>;
}

fn run_test_set<R: Renderer>(
    test_set: &qt::TestSet,
    mut test_set_context: TestSetContext,
    stdout: &mut Stdout,
    renderer: R,
) -> Result<()> {
    renderer
        .render_test_set(stdout, test_set, &test_set_context)
        .into_diagnostic()?;
    for test_case in &test_set.test_cases {
        // skip any test case we don't support
        if !test_case.is_supported(&test_set_context.catalog_context.known_dependencies) {
            continue;
        }
        renderer
            .render_test_case(stdout, test_case)
            .into_diagnostic()?;
        let test_result = test_case.run(&mut test_set_context, &test_set.shared_environments);
        renderer
            .render_test_result(stdout, &test_result)
            .into_diagnostic()?;
    }
    renderer
        .render_test_set_summary(stdout, test_set)
        .into_diagnostic()?;
    Ok(())
}

struct CharacterRenderer {}

impl CharacterRenderer {
    fn new() -> Self {
        Self {}
    }
}

impl Renderer for CharacterRenderer {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
        test_set_context: &TestSetContext,
    ) -> crossterm::Result<()> {
        print!("{} ", test_set_context.file_path.display());
        Ok(())
    }

    fn render_test_case(
        &self,
        stdout: &mut Stdout,
        test_case: &qt::TestCase,
    ) -> crossterm::Result<()> {
        Ok(())
    }

    fn render_test_result(
        &self,
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
            TestResult::ContextItemError(_) => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
            TestResult::UnsupportedDependency => {
                // do not show any output as this is skipped
            }
        }
        Ok(())
    }

    fn render_test_set_summary(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
    ) -> crossterm::Result<()> {
        println!();
        Ok(())
    }
}

struct VerboseRenderer {}

impl VerboseRenderer {
    fn new() -> Self {
        Self {}
    }
}

impl Renderer for VerboseRenderer {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
        test_set_context: &TestSetContext,
    ) -> crossterm::Result<()> {
        println!("{}", test_set_context.file_path.display());
        println!("{}", test_set.name);
        for description in &test_set.descriptions {
            println!("{} ", description);
        }
        Ok(())
    }

    fn render_test_case(
        &self,
        stdout: &mut Stdout,
        test_case: &qt::TestCase,
    ) -> crossterm::Result<()> {
        print!("{} ... ", test_case.name);
        Ok(())
    }

    fn render_test_result(
        &self,
        stdout: &mut Stdout,
        test_result: &TestResult,
    ) -> crossterm::Result<()> {
        match test_result {
            TestResult::Passed => {
                println!("{}", "PASS".green());
            }
            TestResult::PassedWithWrongError(_) => {
                println!("{}", "PASS".green());
            }
            TestResult::Failed(_) => {
                println!("{}", "FAIL".red());
            }
            TestResult::RuntimeError(error) => {
                println!("{} {}", "RUNTIME ERROR".red(), error);
            }
            TestResult::CompilationError(error) => {
                println!("{} {}", "COMPILATION ERROR".red(), error);
            }
            TestResult::UnsupportedExpression(error) => {
                println!("{} {}", "UNSUPPORTED EXPRESSION ERROR".red(), error);
            }
            TestResult::Unsupported => {
                println!("{}", "UNSUPPORTED".red());
            }
            TestResult::ContextItemError(error) => {
                println!("{} {}", "CONTEXT ITEM ERROR".red(), error);
            }
            TestResult::UnsupportedDependency => {
                unreachable!();
            }
        }
        Ok(())
    }

    fn render_test_set_summary(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
    ) -> crossterm::Result<()> {
        println!();
        Ok(())
    }
}

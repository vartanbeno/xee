use crossterm::{
    execute,
    style::{self, Stylize},
};
use miette::{miette, Diagnostic, IntoDiagnostic, Result};
use std::io::{stdout, Stdout};
use std::path::Path;

use crate::assert::{TestOutcome, UnexpectedError};
use crate::qt;
use crate::run::RunContext;

pub(crate) fn run(mut run_context: RunContext) -> Result<()> {
    let mut stdout = stdout();
    // XXX annoying clone
    for file_path in &run_context.catalog.file_paths.clone() {
        run_path_helper(&mut run_context, file_path, &mut stdout)?
    }
    Ok(())
}

pub(crate) fn run_path(mut run_context: RunContext, path: &Path) -> Result<()> {
    let mut stdout = stdout();
    run_path_helper(&mut run_context, path, &mut stdout)
}

fn run_path_helper(run_context: &mut RunContext, path: &Path, stdout: &mut Stdout) -> Result<()> {
    if !run_context.catalog.file_paths.contains(path) {
        miette!("File not found in catalog: {:?}", path);
    }
    let verbose = run_context.verbose;
    let full_path = run_context.catalog.base_dir().join(path);
    let test_set = qt::TestSet::load_from_file(&mut run_context.xot, &full_path)?;
    if verbose {
        run_test_set(run_context, &test_set, stdout, VerboseRenderer::new())?;
    } else {
        run_test_set(run_context, &test_set, stdout, CharacterRenderer::new())?;
    }
    Ok(())
}

trait Renderer {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
        catalog: &qt::Catalog,
    ) -> crossterm::Result<()>;
    fn render_test_case(
        &self,
        stdout: &mut Stdout,
        test_case: &qt::TestCase,
    ) -> crossterm::Result<()>;
    fn render_test_outcome(
        &self,
        stdout: &mut Stdout,
        test_result: &TestOutcome,
    ) -> crossterm::Result<()>;
    fn render_test_set_summary(
        &self,
        stdout: &mut Stdout,
        test_set: &qt::TestSet,
    ) -> crossterm::Result<()>;
}

fn run_test_set<R: Renderer>(
    run_context: &mut RunContext,
    test_set: &qt::TestSet,
    stdout: &mut Stdout,
    renderer: R,
) -> Result<()> {
    renderer
        .render_test_set(stdout, test_set, &run_context.catalog)
        .into_diagnostic()?;
    for test_case in &test_set.test_cases {
        // skip any test case we don't support
        if !test_case.is_supported(&run_context.known_dependencies) {
            continue;
        }
        renderer
            .render_test_case(stdout, test_case)
            .into_diagnostic()?;
        let outcome = test_case.run(run_context, test_set);
        renderer
            .render_test_outcome(stdout, &outcome)
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
        _stdout: &mut Stdout,
        test_set: &qt::TestSet,
        catalog: &qt::Catalog,
    ) -> crossterm::Result<()> {
        print!("{} ", test_set.file_path(catalog).display());
        Ok(())
    }

    fn render_test_case(
        &self,
        _stdout: &mut Stdout,
        _test_case: &qt::TestCase,
    ) -> crossterm::Result<()> {
        Ok(())
    }

    fn render_test_outcome(
        &self,
        stdout: &mut Stdout,
        outcome: &TestOutcome,
    ) -> crossterm::Result<()> {
        match outcome {
            TestOutcome::Passed => {
                execute!(stdout, style::PrintStyledContent(".".green()))?;
            }
            TestOutcome::PassedWithUnexpectedError(_) => {
                execute!(stdout, style::PrintStyledContent(".".green()))?;
            }
            TestOutcome::Failed(_) => {
                execute!(stdout, style::PrintStyledContent("F".red()))?;
            }
            TestOutcome::RuntimeError(_) => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
            TestOutcome::CompilationError(_) => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
            TestOutcome::UnsupportedExpression(_) => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
            TestOutcome::Unsupported => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
            TestOutcome::EnvironmentError(_) => {
                execute!(stdout, style::PrintStyledContent("E".red()))?;
            }
        }
        Ok(())
    }

    fn render_test_set_summary(
        &self,
        _stdout: &mut Stdout,
        _test_set: &qt::TestSet,
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
        _stdout: &mut Stdout,
        test_set: &qt::TestSet,
        catalog: &qt::Catalog,
    ) -> crossterm::Result<()> {
        println!("{}", test_set.file_path(catalog).display());
        println!("{}", test_set.name);
        for description in &test_set.descriptions {
            println!("{} ", description);
        }
        Ok(())
    }

    fn render_test_case(
        &self,
        _stdout: &mut Stdout,
        test_case: &qt::TestCase,
    ) -> crossterm::Result<()> {
        print!("{} ... ", test_case.name);
        Ok(())
    }

    fn render_test_outcome(
        &self,
        _stdout: &mut Stdout,
        test_result: &TestOutcome,
    ) -> crossterm::Result<()> {
        match test_result {
            TestOutcome::Passed => {
                println!("{}", "PASS".green());
            }
            TestOutcome::PassedWithUnexpectedError(error) => match error {
                UnexpectedError::Code(s) => println!("{} code: {}", "PASS".green(), s),
                UnexpectedError::Error(e) => println!("{} error: {}", "PASS".green(), e),
            },
            TestOutcome::Failed(failure) => {
                println!("{} {}", "FAIL".red(), failure);
            }
            TestOutcome::RuntimeError(error) => match error.code() {
                Some(code) => {
                    println!("{} {} {}", "RUNTIME ERROR".red(), code, error);
                }
                None => {
                    println!("{} {}", "RUNTIME ERROR".red(), error);
                }
            },
            TestOutcome::CompilationError(error) => match error.code() {
                Some(code) => {
                    println!("{} {} {}", "COMPILATION ERROR".red(), code, error);
                }
                None => {
                    println!("{} {}", "COMPILATION ERROR".red(), error);
                }
            },
            TestOutcome::UnsupportedExpression(error) => {
                println!("{} {}", "UNSUPPORTED EXPRESSION ERROR".red(), error);
            }
            TestOutcome::Unsupported => {
                println!("{}", "UNSUPPORTED".red());
            }
            TestOutcome::EnvironmentError(error) => {
                println!("{} {}", "CONTEXT ITEM ERROR".red(), error);
            }
        }
        Ok(())
    }

    fn render_test_set_summary(
        &self,
        _stdout: &mut Stdout,
        _test_set: &qt::TestSet,
    ) -> crossterm::Result<()> {
        println!();
        Ok(())
    }
}

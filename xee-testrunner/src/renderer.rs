use crossterm::{
    execute,
    style::{self, Stylize},
};
use std::io::Stdout;

use crate::{
    catalog::Catalog,
    environment::Environment,
    testcase::{Runnable, TestCase, TestOutcome},
    testset::TestSet,
};

pub(crate) trait Renderer<E: Environment, R: Runnable<E>> {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> crossterm::Result<()>;
    fn render_test_case(
        &self,
        stdout: &mut Stdout,
        test_case: &TestCase<E>,
    ) -> crossterm::Result<()>;
    fn render_test_outcome(
        &self,
        stdout: &mut Stdout,
        test_result: &TestOutcome,
    ) -> crossterm::Result<()>;
    fn render_test_set_summary(
        &self,
        stdout: &mut Stdout,
        test_set: &TestSet<E, R>,
    ) -> crossterm::Result<()>;
}

struct VerboseRenderer {}

impl VerboseRenderer {
    fn new() -> Self {
        Self {}
    }
}

impl<E: Environment, R: Runnable<E>> Renderer<E, R> for VerboseRenderer {
    fn render_test_set(
        &self,
        _stdout: &mut Stdout,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
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
        test_case: &TestCase<E>,
    ) -> crossterm::Result<()> {
        print!("{} ... ", test_case.name);
        Ok(())
    }

    fn render_test_outcome(
        &self,
        _stdout: &mut Stdout,
        test_result: &TestOutcome,
    ) -> crossterm::Result<()> {
        println!("{}", test_result);
        Ok(())
    }

    fn render_test_set_summary(
        &self,
        _stdout: &mut Stdout,
        _test_set: &TestSet<E, R>,
    ) -> crossterm::Result<()> {
        println!();
        Ok(())
    }
}

struct CharacterRenderer {}

impl CharacterRenderer {
    fn new() -> Self {
        Self {}
    }
}

impl<E: Environment, R: Runnable<E>> Renderer<E, R> for CharacterRenderer {
    fn render_test_set(
        &self,
        _stdout: &mut Stdout,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> crossterm::Result<()> {
        print!("{} ", test_set.file_path(catalog).display());
        Ok(())
    }

    fn render_test_case(
        &self,
        _stdout: &mut Stdout,
        _test_case: &TestCase<E>,
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
            TestOutcome::UnexpectedError(_) => {
                execute!(stdout, style::PrintStyledContent("F".red()))?;
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
        _test_set: &TestSet<E, R>,
    ) -> crossterm::Result<()> {
        println!();
        Ok(())
    }
}

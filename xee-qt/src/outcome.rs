use crossterm::style::Stylize;
use miette::Diagnostic;
use xee_xpath::Error;

use crate::assert::Failure;

#[derive(Debug, PartialEq)]
pub enum UnexpectedError {
    Code(String),
    Error(Error),
}

#[derive(Debug, PartialEq)]
pub enum TestOutcome {
    Passed,
    PassedWithUnexpectedError(UnexpectedError),
    Failed(Failure),
    RuntimeError(Error),
    CompilationError(Error),
    UnsupportedExpression(Error),
    Unsupported,
    EnvironmentError(String),
}

#[derive(Debug)]
pub struct TestOutcomes(pub Vec<(String, TestOutcome)>);

impl std::fmt::Display for TestOutcomes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (name, test_outcome) in self.0.iter() {
            writeln!(f, "{} ... {}", name, test_outcome)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOutcome::Passed => write!(f, "{}", "PASS".green()),
            TestOutcome::PassedWithUnexpectedError(error) => match error {
                UnexpectedError::Code(s) => write!(f, "{} code: {}", "PASS".green(), s),
                UnexpectedError::Error(e) => write!(f, "{} error: {}", "PASS".green(), e),
            },
            TestOutcome::Failed(failure) => {
                write!(f, "{} {}", "FAIL".red(), failure)
            }
            TestOutcome::RuntimeError(error) => match error.code() {
                Some(code) => {
                    write!(f, "{} {} {}", "RUNTIME ERROR".red(), code, error)
                }
                None => {
                    write!(f, "{} {}", "RUNTIME ERROR".red(), error)
                }
            },
            TestOutcome::CompilationError(error) => match error.code() {
                Some(code) => {
                    write!(f, "{} {} {}", "COMPILATION ERROR".red(), code, error)
                }
                None => {
                    write!(f, "{} {}", "COMPILATION ERROR".red(), error)
                }
            },
            TestOutcome::UnsupportedExpression(error) => {
                write!(f, "{} {}", "UNSUPPORTED EXPRESSION ERROR".red(), error)
            }
            TestOutcome::Unsupported => {
                write!(f, "{}", "UNSUPPORTED".red())
            }
            TestOutcome::EnvironmentError(error) => {
                write!(f, "{} {}", "CONTEXT ITEM ERROR".red(), error)
            }
        }
    }
}

impl TestOutcome {
    pub(crate) fn is_passed(&self) -> bool {
        matches!(self, Self::Passed | Self::PassedWithUnexpectedError(..))
    }
    pub(crate) fn is_exactly_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

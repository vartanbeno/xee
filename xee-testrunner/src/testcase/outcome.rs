use crossterm::style::Stylize;

use xee_xpath_compiler::error::Error;

use super::assert::Failure;

#[derive(Debug, PartialEq)]
pub struct UnexpectedError(pub String);

#[derive(Debug, PartialEq)]
pub enum TestOutcome {
    Passed,
    UnexpectedError(UnexpectedError),
    Failed(Failure),
    RuntimeError(Error),
    CompilationError(Error),
    UnsupportedExpression(Error),
    Unsupported,
    EnvironmentError(String),
}

impl TestOutcome {
    pub(crate) fn is_exactly_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

impl std::fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOutcome::Passed => write!(f, "{}", "PASS".green()),
            TestOutcome::UnexpectedError(error) => match error {
                UnexpectedError(s) => write!(f, "{} code: {}", "WRONG ERROR".yellow(), s),
            },
            TestOutcome::Failed(failure) => {
                write!(f, "{} {}", "FAIL".red(), failure)
            }
            TestOutcome::RuntimeError(error) => {
                write!(f, "{} {} {:?}", "RUNTIME ERROR".red(), error, error)
            }
            TestOutcome::CompilationError(error) => {
                write!(f, "{} {} {:?}", "COMPILATION ERROR".red(), error, error)
            }
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

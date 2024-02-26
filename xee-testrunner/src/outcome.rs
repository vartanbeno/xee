use crossterm::style::Stylize;

use xee_xpath::error::Error;

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

impl TestOutcome {
    pub(crate) fn is_passed(&self) -> bool {
        matches!(self, Self::Passed | Self::PassedWithUnexpectedError(..))
    }
    pub(crate) fn is_exactly_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

impl std::fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOutcome::Passed => write!(f, "{}", "PASS".green()),
            TestOutcome::PassedWithUnexpectedError(error) => match error {
                UnexpectedError::Code(s) => write!(f, "{} code: {}", "WRONG ERROR".yellow(), s),
                UnexpectedError::Error(e) => {
                    write!(f, "{} error: {}", "WRONG ERROR".yellow(), e)
                }
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

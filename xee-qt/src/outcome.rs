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

impl TestOutcome {
    pub(crate) fn is_passed(&self) -> bool {
        matches!(self, Self::Passed | Self::PassedWithUnexpectedError(..))
    }
    pub(crate) fn is_exactly_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

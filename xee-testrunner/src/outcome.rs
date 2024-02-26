// this is in fact the general interpreter error
use xee_xpath::error::Error;

#[derive(Debug, PartialEq)]
pub(crate) enum UnexpectedError {
    Code(String),
    Error(Error),
}

#[derive(Debug, PartialEq)]
pub(crate) struct TestOutcome {
    pub(crate) status: OutcomeStatus,
}

#[derive(Debug, PartialEq)]
pub(crate) enum OutcomeStatus {
    Passed,
    PassedWithUnexpectedError(UnexpectedError),
    Failed,
    RuntimeError(Error),
    CompilationError(Error),
    UnsupportedExpression(Error),
    Unsupported,
    EnvironmentError(String),
}

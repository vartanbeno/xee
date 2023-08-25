use xee_xpath::Error;

use crate::{assert::Failure, qt};

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

trait Outcomes {
    fn outcomes(&self) -> Vec<&TestCaseOutcome>;

    fn count<F>(&self, f: F) -> usize
    where
        F: Fn(&TestCaseOutcome) -> bool,
    {
        self.outcomes().iter().filter(|outcome| f(outcome)).count()
    }

    fn passed(&self) -> usize {
        self.count(|outcome| matches!(outcome.outcome, TestOutcome::Passed))
    }
    fn passed_with_unexpected_error(&self) -> usize {
        self.count(|outcome| matches!(outcome.outcome, TestOutcome::PassedWithUnexpectedError(..)))
    }
    fn failed(&self) -> usize {
        self.count(|outcome| matches!(outcome.outcome, TestOutcome::Failed(..)))
    }
    fn erroring(&self) -> usize {
        self.count(|outcome| {
            matches!(
                outcome.outcome,
                TestOutcome::RuntimeError(..)
                    | TestOutcome::CompilationError(..)
                    | TestOutcome::UnsupportedExpression(..)
                    | TestOutcome::Unsupported
                    | TestOutcome::EnvironmentError(..)
            )
        })
    }
}

#[derive(Debug)]
pub struct TestCaseOutcome {
    pub(crate) test_case_name: String,
    pub(crate) outcome: TestOutcome,
}

impl TestCaseOutcome {
    pub(crate) fn new(test_case: &qt::TestCase, outcome: TestOutcome) -> Self {
        Self {
            test_case_name: test_case.name.clone(),
            outcome,
        }
    }
}

#[derive(Debug)]
pub struct TestSetOutcomes {
    pub(crate) test_set_name: String,
    pub(crate) outcomes: Vec<TestCaseOutcome>,
}

impl TestSetOutcomes {
    pub(crate) fn new(test_set: &qt::TestSet) -> Self {
        Self {
            test_set_name: test_set.name.clone(),
            outcomes: Vec::new(),
        }
    }

    pub(crate) fn has_failures(&self) -> bool {
        self.passed_with_unexpected_error() > 0 || self.failed() > 0 || self.erroring() > 0
    }

    pub(crate) fn add_outcome(&mut self, test_case: &qt::TestCase, outcome: TestOutcome) {
        self.outcomes.push(TestCaseOutcome::new(test_case, outcome));
    }

    pub(crate) fn failing_names(&self) -> Vec<String> {
        self.outcomes
            .iter()
            .filter(|outcome| !outcome.outcome.is_exactly_passed())
            .map(|outcome| outcome.test_case_name.clone())
            .collect()
    }
}

impl Outcomes for TestSetOutcomes {
    fn outcomes(&self) -> Vec<&TestCaseOutcome> {
        self.outcomes.iter().collect()
    }
}

pub struct CatalogOutcomes {
    outcomes: Vec<TestSetOutcomes>,
}

impl CatalogOutcomes {
    pub(crate) fn new() -> Self {
        Self {
            outcomes: Vec::new(),
        }
    }

    pub(crate) fn add_outcomes(&mut self, test_set_outcomes: TestSetOutcomes) {
        self.outcomes.push(test_set_outcomes);
    }
}

impl Outcomes for CatalogOutcomes {
    fn outcomes(&self) -> Vec<&TestCaseOutcome> {
        self.outcomes
            .iter()
            .flat_map(|test_set_outcome| test_set_outcome.outcomes())
            .collect()
    }
}

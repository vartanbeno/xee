use std::io::Stdout;

use crate::{
    catalog::Catalog,
    environment::Environment,
    outcome::TestOutcome,
    testcase::{Runnable, TestCase},
    testset::TestSet,
};

pub(crate) trait Renderer<E: Environment, R: Runnable<E>> {
    fn render_test_set(
        &self,
        stdout: &mut Stdout,
        test_set: &TestSet<E, R>,
        catalog: &Catalog<E>,
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

use std::{io::Stdout, path::PathBuf};

use crate::{
    dependency::Dependencies,
    environment::{Environment, SharedEnvironments},
    renderer::Renderer,
    runcontext::RunContext,
    testcase::{Runnable, TestCase},
};

#[derive(Debug)]
pub(crate) struct TestSet<E: Environment, R: Runnable<E>> {
    pub(crate) full_path: PathBuf,
    pub(crate) name: String,
    pub(crate) descriptions: Vec<String>,
    pub(crate) dependencies: Dependencies,
    pub(crate) shared_environments: SharedEnvironments<E>,
    pub(crate) test_cases: Vec<R>,
}

impl<E: Environment, R: Runnable<E>> TestSet<E, R> {
    // fn run<Ren: Renderer<E, R>>(
    //     run_context: &mut RunContext,
    //     test_filter: &impl TestFilter,
    //     test_set: &TestSet<E, R>,
    //     stdout: &mut Stdout,
    //     renderer: Ren,
    // ) -> Result<TestSetOutcomes> {
    //     renderer.render_test_set(stdout, test_set, &run_context.catalog)?;

    //     let mut test_set_outcomes = TestSetOutcomes::new(&test_set.name);
    //     for test_case in &test_set.test_cases {
    //         if !test_filter.is_included(test_set, test_case) {
    //             test_set_outcomes.add_filtered();
    //             continue;
    //         }
    //         // skip any test case we don't support, either on test set or
    //         // test case level
    //         if !test_set
    //             .dependencies
    //             .is_supported(&run_context.known_dependencies)
    //             || !test_case
    //                 .dependencies
    //                 .is_supported(&run_context.known_dependencies)
    //         {
    //             test_set_outcomes.add_unsupported();
    //             continue;
    //         }
    //         renderer.render_test_case(stdout, test_case)?;
    //         let outcome = test_case.run(run_context, test_set);
    //         renderer.render_test_outcome(stdout, &outcome)?;
    //         test_set_outcomes.add_outcome(&test_case.name, outcome);
    //     }
    //     renderer.render_test_set_summary(stdout, test_set)?;
    //     Ok(test_set_outcomes)
    // }
}

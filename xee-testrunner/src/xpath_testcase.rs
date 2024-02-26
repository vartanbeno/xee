use std::borrow::Cow;

use xee_name::Namespaces;
use xee_xpath::{
    context::{DynamicContext, StaticContext},
    parse,
};

use crate::{
    assert::TestCaseResult,
    catalog::Catalog,
    environment::XPathEnvironmentSpec,
    error::Result,
    outcome::{OutcomeStatus, TestOutcome},
    runcontext::RunContext,
    testcase::TestCase,
    testset::TestSet,
};

#[derive(Debug)]
pub(crate) struct XPathTestCase {
    test_case: TestCase<XPathEnvironmentSpec>,

    pub(crate) test: String,
    pub(crate) result: TestCaseResult,
}

impl XPathTestCase {
    pub(crate) fn run(
        &self,
        run_context: &mut RunContext<XPathEnvironmentSpec>,
        test_set: &TestSet<XPathEnvironmentSpec>,
    ) -> TestOutcome {
        let variables = self.test_case.variables(run_context, test_set);
        let variables = match variables {
            Ok(variables) => variables,
            Err(error) => {
                return TestOutcome {
                    status: OutcomeStatus::EnvironmentError(error.to_string()),
                }
            }
        };

        let context_item = self.test_case.context_item(run_context, test_set);
        let context_item = match context_item {
            Ok(context_item) => context_item,
            Err(error) => {
                return TestOutcome {
                    status: OutcomeStatus::EnvironmentError(error.to_string()),
                }
            }
        };

        let namespaces = self.namespaces(&run_context.catalog, test_set);
        let namespaces = match namespaces {
            Ok(namespaces) => namespaces,
            Err(error) => {
                return TestOutcome {
                    status: OutcomeStatus::EnvironmentError(error.to_string()),
                }
            }
        };

        let variable_names = variables.iter().map(|(name, _)| name.clone()).collect();
        let static_context = StaticContext::new(namespaces, variable_names);
        let program = parse(&static_context, &self.test);
        let program = match program {
            Ok(xpath) => xpath,
            Err(error) => {
                return match &self.result {
                    TestCaseResult::AssertError(assert_error) => TestOutcome {
                        status: assert_error.assert_error(&error.error),
                    },
                    TestCaseResult::AnyOf(any_of) => TestOutcome {
                        status: any_of.assert_error(&error.error),
                    },
                    _ => TestOutcome {
                        status: OutcomeStatus::CompilationError(error.error),
                    },
                }
            }
        };

        let dynamic_context = DynamicContext::new(
            &static_context,
            Cow::Borrowed(&run_context.documents),
            Cow::Borrowed(&variables),
        );
        let runnable = program.runnable(&dynamic_context);
        let result = runnable.many(context_item.as_ref(), &mut run_context.xot);
        TestOutcome {
            status: self.result.assert_result(
                &runnable,
                &mut run_context.xot,
                &result.map_err(|error| error.error),
            ),
        }
    }

    fn namespaces<'a>(
        &'a self,
        catalog: &'a Catalog<XPathEnvironmentSpec>,
        test_set: &'a TestSet<XPathEnvironmentSpec>,
    ) -> Result<Namespaces<'a>> {
        let environments = self
            .test_case
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let mut namespaces = Namespaces::default();
        for environment in environments {
            namespaces.add(&environment.namespace_pairs())
        }
        Ok(namespaces)
    }
}

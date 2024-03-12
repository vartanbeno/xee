use anyhow::Result;
use std::{borrow::Cow, path::Path};
use xee_name::Namespaces;
use xee_xpath::{
    context::{DynamicContext, StaticContext},
    parse,
};
use xee_xpath_load::{convert_string, ContextLoadable, Queries, Query};

use crate::{
    catalog::Catalog, environment::XPathEnvironmentSpec, runcontext::RunContext, testset::TestSet,
};

use super::{
    assert::TestCaseResult,
    core::{Runnable, TestCase},
    outcome::TestOutcome,
};

#[derive(Debug)]
pub(crate) struct XPathTestCase {
    pub(crate) test_case: TestCase<XPathEnvironmentSpec>,
    pub(crate) test: String,
}

impl XPathTestCase {
    fn namespaces<'a>(
        &'a self,
        catalog: &'a Catalog<XPathEnvironmentSpec, Self>,
        test_set: &'a TestSet<XPathEnvironmentSpec, Self>,
    ) -> anyhow::Result<Namespaces<'a>> {
        let environments = self
            .test_case
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>, crate::error::Error>>()?;
        let mut namespaces = Namespaces::default();
        for environment in environments {
            namespaces.add(&environment.namespace_pairs())
        }
        Ok(namespaces)
    }
}

impl Runnable<XPathEnvironmentSpec> for XPathTestCase {
    fn test_case(&self) -> &TestCase<XPathEnvironmentSpec> {
        &self.test_case
    }

    fn run(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<XPathEnvironmentSpec, Self>,
        test_set: &TestSet<XPathEnvironmentSpec, Self>,
    ) -> TestOutcome {
        let variables = self.test_case.variables(run_context, catalog, test_set);
        let variables = match variables {
            Ok(variables) => variables,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        let context_item = self.test_case.context_item(run_context, catalog, test_set);
        let context_item = match context_item {
            Ok(context_item) => context_item,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        let namespaces = self.namespaces(catalog, test_set);
        let namespaces = match namespaces {
            Ok(namespaces) => namespaces,
            Err(error) => return TestOutcome::EnvironmentError(error.to_string()),
        };

        let variable_names = variables.iter().map(|(name, _)| name.clone()).collect();
        let static_context = StaticContext::new(namespaces, variable_names);
        let program = parse(&static_context, &self.test);
        let program = match program {
            Ok(xpath) => xpath,
            Err(error) => {
                return match &self.test_case.result {
                    TestCaseResult::AssertError(assert_error) => {
                        assert_error.assert_error(&error.error)
                    }

                    TestCaseResult::AnyOf(any_of) => any_of.assert_error(&error.error),
                    _ => TestOutcome::CompilationError(error.error),
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
        self.test_case.result.assert_result(
            &runnable,
            &mut run_context.xot,
            &result.map_err(|error| error.error),
        )
    }

    fn query<'a>(
        mut queries: Queries<'a>,
        path: &'a Path,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        XPathEnvironmentSpec: 'a,
    {
        let test_query = queries.one("test/string()", convert_string)?;
        let (queries, test_case_query) = TestCase::load_with_context(queries, path)?;
        let test_case_query = test_case_query.map(move |test_case, session, item| {
            Ok(XPathTestCase {
                test_case,
                test: test_query.execute(session, item)?,
            })
        });
        Ok((queries, test_case_query))
    }
}

impl ContextLoadable<Path> for XPathTestCase {
    fn load_with_context<'a>(
        mut queries: Queries<'a>,
        path: &'a Path,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        XPathEnvironmentSpec: 'a,
    {
        let test_query = queries.one("test/string()", convert_string)?;
        let (queries, test_case_query) = TestCase::load_with_context(queries, path)?;
        let test_case_query = test_case_query.map(move |test_case, session, item| {
            Ok(XPathTestCase {
                test_case,
                test: test_query.execute(session, item)?,
            })
        });
        Ok((queries, test_case_query))
    }
}

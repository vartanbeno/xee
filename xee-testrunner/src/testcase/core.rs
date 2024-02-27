use std::path::Path;

use xee_xpath::{context::Variables, sequence, Queries, Query};
use xot::Xot;

use crate::{
    catalog::Catalog,
    dependency::{Dependencies, Dependency},
    environment::{
        Environment, EnvironmentIterator, EnvironmentRef, TestCaseEnvironment, XPathEnvironmentSpec,
    },
    error::Result,
    load::convert_string,
    metadata::Metadata,
    runcontext::RunContext,
    testset::TestSet,
};

use super::{assert::TestCaseResult, outcome::TestOutcome, XPathTestCase};

pub(crate) trait Runnable<E: Environment>: std::marker::Sized {
    fn test_case(&self) -> &TestCase<E>;

    fn run(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, Self>,
        test_set: &TestSet<E, Self>,
    ) -> TestOutcome;
}

#[derive(Debug)]
pub(crate) struct TestCase<E: Environment> {
    pub(crate) name: String,
    pub(crate) metadata: Metadata,
    // environments can be a reference by name, or a locally defined environment
    pub(crate) environments: Vec<TestCaseEnvironment<E>>,
    pub(crate) dependencies: Dependencies,
    pub(crate) result: TestCaseResult,
    // pub(crate) modules: Vec<Module>,
}

impl<E: Environment> TestCase<E> {
    pub(crate) fn environments<'a, R: Runnable<E>>(
        &'a self,
        catalog: &'a Catalog<E, R>,
        test_set: &'a TestSet<E, R>,
    ) -> EnvironmentIterator<'a, E> {
        EnvironmentIterator::new(
            vec![&catalog.shared_environments, &test_set.shared_environments],
            &self.environments,
        )
    }

    pub(crate) fn context_item<R: Runnable<E>>(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> Result<Option<sequence::Item>> {
        let environments = self
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let xot = &mut run_context.xot;
        let documents = &mut run_context.documents;
        for environment in environments {
            let item = environment
                .environment_spec()
                .context_item(xot, documents)?;
            if let Some(item) = item {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    pub(crate) fn variables<R: Runnable<E>>(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_set: &TestSet<E, R>,
    ) -> Result<Variables> {
        let environments = self
            .environments(catalog, test_set)
            .collect::<Result<Vec<_>>>()?;
        let mut variables = Variables::new();
        let xot = &mut run_context.xot;
        let source_cache = &mut run_context.documents;
        for environment in environments {
            variables.extend(
                environment
                    .environment_spec()
                    .variables(xot, source_cache)?,
            );
        }
        Ok(variables)
    }

    pub(crate) fn test_cases_query<'a>(
        xot: &Xot,
        path: &'a Path,
        mut queries: Queries<'a>,
    ) -> Result<(Queries<'a>, impl Query<Vec<XPathTestCase>> + 'a)> {
        let name_query = queries.one("@name/string()", convert_string)?;
        let (mut queries, metadata_query) = Metadata::metadata_query(xot, queries)?;

        let ref_query = queries.option("@ref/string()", convert_string)?;
        let (mut queries, environment_query) =
            XPathEnvironmentSpec::environment_spec_query(xot, path, queries)?;
        let local_environment_query = queries.many("environment", move |session, item| {
            let ref_ = ref_query.execute(session, item)?;
            if let Some(ref_) = ref_ {
                Ok(TestCaseEnvironment::Ref(EnvironmentRef { ref_ }))
            } else {
                Ok(TestCaseEnvironment::Local(Box::new(
                    environment_query.execute(session, item)?,
                )))
            }
        })?;

        let (queries, result_query) = TestCaseResult::testcase_result_query(xot, queries)?;
        let (mut queries, dependency_query) = Dependency::dependency_query(xot, queries)?;
        let test_query = queries.one("test/string()", convert_string)?;
        let test_case_query = queries.many("test-case", move |session, item| {
            let test_case = TestCase {
                name: name_query.execute(session, item)?,
                metadata: metadata_query.execute(session, item)?,
                environments: local_environment_query.execute(session, item)?,
                dependencies: Dependencies::new(
                    dependency_query
                        .execute(session, item)?
                        .into_iter()
                        .flatten()
                        .collect(),
                ),
                result: result_query.execute(session, item)?,
            };
            let xpath_test_case = XPathTestCase {
                test_case,
                test: test_query.execute(session, item)?,
            };
            Ok(xpath_test_case)
        })?;

        Ok((queries, test_case_query))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use xee_xpath::xml::Documents;
    use xot::Xot;

    use crate::{dependency::KnownDependencies, environment::EnvironmentSpec};

    use super::*;

    // #[test]
    // fn test_simple_runnable() {
    //     struct FakeEnvironment {
    //         environment_spec: EnvironmentSpec,
    //     }

    //     impl Environment for FakeEnvironment {
    //         fn empty() -> Self {
    //             Self {
    //                 environment_spec: EnvironmentSpec::empty(),
    //             }
    //         }

    //         fn environment_spec(&self) -> &EnvironmentSpec {
    //             &self.environment_spec
    //         }
    //     }
    //     // make a simple fake runnable
    //     struct FakeRunnable {
    //         test_case: TestCase<FakeEnvironment>,
    //     }

    //     impl Runnable<FakeEnvironment> for FakeRunnable {
    //         fn test_case(&self) -> &TestCase<FakeEnvironment> {
    //             &self.test_case
    //         }

    //         fn run(
    //             &self,
    //             _run_context: &mut RunContext,
    //             _catalog: &Catalog<FakeEnvironment, Self>,
    //             _test_set: &TestSet<FakeEnvironment, Self>,
    //         ) -> TestOutcome {
    //             TestOutcome::Passed
    //         }
    //     }

    //     let runnable = FakeRunnable {
    //         test_case: TestCase {
    //             name: "test".to_string(),
    //             metadata: Metadata {
    //                 description: None,
    //                 created: None,
    //                 modified: vec![],
    //             },
    //             environments: vec![],
    //             dependencies: Dependencies::empty(),

    //         },
    //     };

    //     let xot = Xot::new();
    //     let documents = Documents::new();
    //     let known_dependencies = KnownDependencies::empty();
    //     let run_context = RunContext::new(xot, documents, known_dependencies);
    // }
}

use std::{
    io::Stdout,
    path::{Path, PathBuf},
};

use xee_xpath::{Queries, Query};

use crate::{
    catalog::Catalog,
    dependency::{Dependencies, Dependency},
    environment::{Environment, SharedEnvironments},
    error::Result,
    filter::TestFilter,
    load::{convert_string, ContextLoadable},
    outcomes::TestSetOutcomes,
    renderer::Renderer,
    runcontext::RunContext,
    testcase::Runnable,
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
    fn base_dir(&self) -> &Path {
        self.full_path.parent().unwrap()
    }

    pub(crate) fn file_path(&self, catalog: &Catalog<E, R>) -> &Path {
        self.full_path.strip_prefix(catalog.base_dir()).unwrap()
    }

    pub(crate) fn run<Ren: Renderer<E, R>>(
        &self,
        run_context: &mut RunContext,
        catalog: &Catalog<E, R>,
        test_filter: &impl TestFilter<E, R>,
        stdout: &mut Stdout,
        renderer: &Ren,
    ) -> Result<TestSetOutcomes> {
        renderer.render_test_set(stdout, catalog, self)?;

        let mut test_set_outcomes = TestSetOutcomes::new(&self.name);
        for runner in &self.test_cases {
            let test_case = runner.test_case();
            if !test_filter.is_included(self, test_case) {
                test_set_outcomes.add_filtered();
                continue;
            }
            // skip any test case we don't support, either on test set or
            // test case level
            if !self
                .dependencies
                .is_supported(&run_context.known_dependencies)
                || !test_case
                    .dependencies
                    .is_supported(&run_context.known_dependencies)
            {
                test_set_outcomes.add_unsupported();
                continue;
            }
            renderer.render_test_case(stdout, test_case)?;
            let outcome = runner.run(run_context, catalog, self);
            renderer.render_test_outcome(stdout, &outcome)?;
            test_set_outcomes.add_outcome(&test_case.name, outcome);
        }
        renderer.render_test_set_summary(stdout, self)?;
        Ok(test_set_outcomes)
    }
}

impl<E: Environment, R: Runnable<E>> ContextLoadable<Path> for TestSet<E, R> {
    fn query_with_context<'a>(
        mut queries: Queries<'a>,
        path: &'a Path,
    ) -> Result<(Queries<'a>, impl Query<TestSet<E, R>> + 'a)>
    where
        E: 'a,
        R: 'a,
    {
        let name_query = queries.one("@name/string()", convert_string)?;
        let descriptions_query = queries.many("description/string()", convert_string)?;

        let (queries, shared_environments_query) =
            SharedEnvironments::query_with_context(queries, path)?;
        let (queries, dependency_query) = Dependency::query(queries)?;
        let (mut queries, test_case_query) = R::query(queries, path)?;
        let test_cases_query = queries.many("test-case", move |session, item| {
            test_case_query.execute(session, item)
        })?;
        let test_set_query = queries.one("/test-set", move |session, item| {
            let name = name_query.execute(session, item)?;
            let descriptions = descriptions_query.execute(session, item)?;
            let dependencies = dependency_query.execute(session, item)?;
            let shared_environments = shared_environments_query.execute(session, item)?;
            let test_cases = test_cases_query.execute(session, item)?;
            Ok(TestSet {
                full_path: path.to_path_buf(),
                name,
                descriptions,
                dependencies: Dependencies::new(dependencies.into_iter().flatten().collect()),
                shared_environments,
                test_cases,
            })
        })?;
        Ok((queries, test_set_query))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        environment::{EnvironmentRef, XPathEnvironmentSpec},
        load::XPATH_NS,
        testcase::XPathTestCase,
    };

    use super::*;

    use xot::Xot;

    #[test]
    fn test_load_set_set() {
        let xml = r#"
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="testset-name">
   <description>Test set</description>

   <environment name="x">
      <param name="a"
         select="''"
         declared="true"/>
      <param name="b"
         select="()"
         declared="true"/>
      <param name="c"
         select="0"
         declared="true"/>
   </environment>

   <test-case name="test-1">
      <description>Test 1</description>
      <created by="Bar Quxson" on="2024-01-01"/>
      <test>1</test>
      <result>
         <assert-true/>
      </result>
   </test-case>

   <test-case name="test-2">
      <description>Test 2</description>
      <created by="Flurb Flurba" on="2024-02-01"/>
      <test>2</test>
      <result>
         <assert-true/>
      </result>
   </test-case>
</test-set>"#;

        let mut xot = Xot::new();

        let path = PathBuf::from("bar/foo");
        let test_set = TestSet::<XPathEnvironmentSpec, XPathTestCase>::load_from_xml_with_context(
            &mut xot, xml, XPATH_NS, &path,
        )
        .unwrap();
        assert_eq!(test_set.name, "testset-name");
        assert_eq!(test_set.test_cases.len(), 2);
        assert!(test_set
            .shared_environments
            .get(&EnvironmentRef::new("x".to_string()))
            .is_some());
    }
}

use derive_builder::Builder;
use miette::{Diagnostic, IntoDiagnostic, Result, WrapErr};
use std::path::{Path, PathBuf};
use xee_xpath::{
    Atomic, DynamicContext, Error, Item, Namespaces, Node, StackValue, StaticContext, XPath,
};
use xot::Xot;

use crate::collection::FxIndexSet;
use crate::environment::SourceCache;
use crate::qt;
use crate::serialize::serialize;

#[derive(Debug)]
pub(crate) struct KnownDependencies {
    specs: FxIndexSet<qt::DependencySpec>,
}

impl KnownDependencies {
    fn new(specs: FxIndexSet<qt::DependencySpec>) -> Self {
        Self { specs }
    }

    fn is_supported(&self, dependency: &qt::Dependency) -> bool {
        let contains = self.specs.contains(&dependency.spec);
        if dependency.satisfied {
            contains
        } else {
            !contains
        }
    }
}

impl Default for KnownDependencies {
    fn default() -> Self {
        let mut specs = FxIndexSet::default();
        specs.insert(qt::DependencySpec {
            type_: "spec".to_string(),
            value: "XP30+".to_string(),
        });
        Self::new(specs)
    }
}

// dependency indicator: hashset with type + value keys
// environment: hashmap with environment name as key, empty key should
// always be present. an environment contains a bunch of elements

// if an environment with a schema is referenced, then schema-awareness
// is an implicit dependency

struct TestSetResult {
    results: Vec<TestResult>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TestResult {
    // The test passed
    Passed,
    // The test passed because it errored, but the error
    // code was unexpected
    PassedWithWrongError(Error),
    // We failed with an unexpected stack value
    Failed(StackValue),
    // We failed with an unexpected error during runtime
    RuntimeError(Error),
    // We failed with a compilation error
    CompilationError(Error),
    // We could not compile some xpath expression in the test
    // and therefore the test could not be executed
    UnsupportedExpression(Error),
    // We failed because our implementation does not yet
    // implement something it should
    Unsupported,
    // We couldn't load context item for some reason
    ContextItemError(String),
    // We skipped this test as we don't support the stated
    // dependency
    UnsupportedDependency,
}

#[derive(Default, Builder)]
#[builder(pattern = "owned")]
pub(crate) struct CatalogContext {
    pub(crate) xot: Xot,
    #[builder(default)]
    pub(crate) base_dir: PathBuf,
    #[builder(default)]
    pub(crate) source_cache: SourceCache,
    #[builder(default)]
    pub(crate) known_dependencies: KnownDependencies,
    #[builder(default)]
    pub(crate) verbose: bool,
    #[builder(default)]
    pub(crate) shared_environments: qt::SharedEnvironments,
}

impl CatalogContext {
    pub(crate) fn new(xot: Xot) -> Self {
        Self {
            xot,
            base_dir: PathBuf::new(),
            source_cache: SourceCache::new(),
            known_dependencies: KnownDependencies::default(),
            verbose: false,
            shared_environments: qt::SharedEnvironments::default(),
        }
    }

    pub(crate) fn with_base_dir(xot: Xot, base_dir: &Path) -> Self {
        Self {
            xot,
            base_dir: base_dir.to_path_buf(),
            source_cache: SourceCache::new(),
            known_dependencies: KnownDependencies::default(),
            verbose: false,
            shared_environments: qt::SharedEnvironments::default(),
        }
    }
}

impl Drop for CatalogContext {
    fn drop(&mut self) {
        self.source_cache.cleanup(&mut self.xot);
    }
}

pub(crate) struct TestSetContext<'a> {
    pub(crate) catalog_context: &'a mut CatalogContext,
    pub(crate) file_path: PathBuf,
}

impl<'a> TestSetContext<'a> {
    pub(crate) fn new(catalog_context: &'a mut CatalogContext) -> Self {
        Self {
            catalog_context,
            file_path: PathBuf::from("dummy.xml"),
        }
    }

    pub(crate) fn with_file_path(
        catalog_context: &'a mut CatalogContext,
        file_path: &Path,
    ) -> Self {
        Self {
            catalog_context,
            file_path: file_path.to_path_buf(),
        }
    }
}

impl<'a> qt::TestSet {
    // XXX Make this result an iterator of results?
    fn run(&'a self, mut test_set_context: TestSetContext) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test_case in &self.test_cases {
            let result = test_case.run(&mut test_set_context, &self.shared_environments);
            results.push(result);
        }
        Ok(results)
    }
}

impl qt::TestCase {
    pub(crate) fn is_supported(&self, known_dependencies: &KnownDependencies) -> bool {
        // if we have no dependencies, we're always supported
        if self.dependencies.is_empty() {
            return true;
        }
        // if any of the listed dependencies is supported, we're supported
        for dependency in &self.dependencies {
            if known_dependencies.is_supported(dependency) {
                return true;
            }
        }
        false
    }

    pub(crate) fn run<'a>(
        &'a self,
        test_set_context: &'a mut TestSetContext,
        shared_environments: &qt::SharedEnvironments,
    ) -> TestResult {
        if !self.is_supported(&test_set_context.catalog_context.known_dependencies) {
            return TestResult::UnsupportedDependency;
        }
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let xpath = XPath::new(&static_context, &self.test);
        let xpath = match xpath {
            Ok(xpath) => xpath,
            Err(error) => {
                return if let qt::TestCaseResult::Error(expected_error) = &self.result {
                    Self::assert_expected_error(expected_error, &error)
                } else {
                    TestResult::CompilationError(error)
                }
            }
        };

        let context_item = self.context_item(
            &mut test_set_context.catalog_context.xot,
            &test_set_context.catalog_context.base_dir,
            &mut test_set_context.catalog_context.source_cache,
            &test_set_context.catalog_context.shared_environments,
            shared_environments,
        );
        let context_item = match context_item {
            Ok(context_item) => context_item,
            Err(error) => return TestResult::ContextItemError(error.to_string()),
        };

        let dynamic_context =
            DynamicContext::new(&test_set_context.catalog_context.xot, &static_context);
        let value = xpath.run(&dynamic_context, context_item.as_ref());
        Self::check_value(
            &mut test_set_context.catalog_context.xot,
            &self.result,
            &value,
        )
    }

    fn context_item(
        &self,
        xot: &mut Xot,
        base_dir: &Path,
        source_cache: &mut SourceCache,
        catalog_shared_environments: &qt::SharedEnvironments,
        test_set_shared_environments: &qt::SharedEnvironments,
    ) -> Result<Option<Item>> {
        for environment in &self.environments {
            match environment {
                qt::TestCaseEnvironment::Local(local_environment) => {
                    // the base dir is the same as in the test set shared environments
                    let base_dir = base_dir.join(&test_set_shared_environments.base_dir);
                    let item = local_environment.context_item(xot, &base_dir, source_cache)?;
                    if let Some(item) = item {
                        return Ok(Some(item));
                    }
                }
                qt::TestCaseEnvironment::Ref(environment_ref) => {
                    for shared_environments in
                        [test_set_shared_environments, catalog_shared_environments]
                    {
                        let environment = shared_environments.get(environment_ref);
                        if let Some(environment) = environment {
                            let base_dir = base_dir.join(&shared_environments.base_dir);
                            let item = environment.context_item(xot, &base_dir, source_cache)?;
                            if let Some(item) = item {
                                return Ok(Some(item));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn check_value(
        xot: &mut Xot,
        result: &qt::TestCaseResult,
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        // we handle any of and all of first, because we don't
        // yet want to distinguish between value and error
        match result {
            qt::TestCaseResult::AllOf(test_case_results) => {
                return Self::assert_all_of(xot, test_case_results, run_result)
            }
            qt::TestCaseResult::AnyOf(test_case_results) => {
                return Self::assert_any_of(xot, test_case_results, run_result)
            }
            _ => {}
        }
        match run_result {
            Ok(value) => {
                let value = value.clone();
                match result {
                    // qt::TestCaseResult::Assert(xpath_expr) => self.assert_(xpath_expr, run_result),
                    qt::TestCaseResult::AssertEq(xpath_expr) => Self::assert_eq(xpath_expr, value),
                    qt::TestCaseResult::AssertTrue => Self::assert_true(value),
                    qt::TestCaseResult::AssertFalse => Self::assert_false(value),
                    qt::TestCaseResult::AssertCount(number) => Self::assert_count(*number, value),
                    qt::TestCaseResult::AssertStringValue(s) => {
                        Self::assert_string_value(xot, s, value)
                    }
                    qt::TestCaseResult::AssertXml(xml) => Self::assert_xml(xot, xml, value),
                    qt::TestCaseResult::Error(error) => {
                        Self::assert_unexpected_no_error(error, value)
                    }
                    qt::TestCaseResult::Unsupported => TestResult::Unsupported,
                    _ => {
                        panic!("unimplemented test case result {:?}", result);
                    }
                }
            }
            Err(error) => match result {
                qt::TestCaseResult::Error(expected_error) => {
                    Self::assert_expected_error(expected_error, error)
                }
                _ => TestResult::RuntimeError(error.clone()),
            },
        }
    }

    fn assert_any_of(
        xot: &mut Xot,
        test_case_results: &[qt::TestCaseResult],
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        for test_case_result in test_case_results {
            let result = Self::check_value(xot, test_case_result, run_result);
            match result {
                TestResult::Passed | TestResult::PassedWithWrongError(_) => return result,
                _ => {}
            }
        }
        match run_result {
            Ok(value) => TestResult::Failed(value.clone()),
            Err(error) => TestResult::RuntimeError(error.clone()),
        }
    }

    fn assert_all_of(
        xot: &mut Xot,
        test_case_results: &[qt::TestCaseResult],
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        for test_case_result in test_case_results {
            let result = Self::check_value(xot, test_case_result, run_result);
            if let TestResult::Failed(_) = result {
                return result;
            }
        }
        TestResult::Passed
    }

    fn assert_(xpath_expr: &qt::XPathExpr, stack_value: Result<StackValue, Error>) -> TestResult {
        unimplemented!()
    }

    fn assert_eq(xpath_expr: &qt::XPathExpr, stack_value: StackValue) -> TestResult {
        let expected_value = Self::run_xpath(xpath_expr);
        match expected_value {
            Ok(expected_value) => {
                if expected_value == stack_value {
                    TestResult::Passed
                } else {
                    TestResult::Failed(stack_value)
                }
            }
            // This only happens if we can't run the xpath expression
            // with the expected value. These errors should stop
            // appearing once we lift ourselves by our bootstraps
            Err(error) => TestResult::UnsupportedExpression(error),
        }
    }

    fn assert_true(stack_value: StackValue) -> TestResult {
        if matches!(stack_value, StackValue::Atomic(Atomic::Boolean(true))) {
            TestResult::Passed
        } else {
            TestResult::Failed(stack_value)
        }
    }

    fn assert_false(stack_value: StackValue) -> TestResult {
        if matches!(stack_value, StackValue::Atomic(Atomic::Boolean(false))) {
            TestResult::Passed
        } else {
            TestResult::Failed(stack_value)
        }
    }

    fn assert_count(count: usize, stack_value: StackValue) -> TestResult {
        let sequence = stack_value.to_sequence();
        if let Ok(sequence) = sequence {
            if sequence.borrow().len() == count {
                TestResult::Passed
            } else {
                TestResult::Failed(stack_value)
            }
        } else {
            TestResult::Failed(stack_value)
        }
    }

    fn assert_string_value(xot: &Xot, s: &str, stack_value: StackValue) -> TestResult {
        let seq = stack_value.to_sequence();
        match seq {
            Ok(seq) => {
                let strings = seq
                    .borrow()
                    .as_slice()
                    .iter()
                    .map(|item| item.string_value(xot))
                    .collect::<Result<Vec<_>, _>>();
                match strings {
                    Ok(strings) => {
                        let joined = strings.join(" ");
                        if joined == s {
                            TestResult::Passed
                        } else {
                            // the string value is not what we expected
                            TestResult::Failed(stack_value)
                        }
                    }
                    // we weren't able to produce a string value
                    Err(_) => TestResult::Failed(stack_value),
                }
            }
            // we weren't able to produce a sequence
            Err(_) => TestResult::Failed(stack_value),
        }
    }

    fn assert_xml(xot: &mut Xot, expected_xml: &str, value: StackValue) -> TestResult {
        let xml = serialize(xot, &value);

        let xml = if let Ok(xml) = xml {
            xml
        } else {
            return TestResult::Failed(value);
        };
        // also wrap expected XML in a sequence element
        let expected_xml = format!("<sequence>{}</sequence>", expected_xml);

        // now parse both with Xot
        let found = xot.parse(&xml).unwrap();
        let expected = xot.parse(&expected_xml).unwrap();

        // and compare
        let c = xot.compare(expected, found);

        // clean up
        xot.remove(found).unwrap();
        xot.remove(expected).unwrap();

        if c {
            TestResult::Passed
        } else {
            TestResult::Failed(value)
        }
    }

    fn assert_expected_error(expected_error: &str, error: &Error) -> TestResult {
        // all errors are officially a pass, but we check whether the error
        // code matches too
        let code = error.code();
        if let Some(code) = code {
            if code.to_string() == expected_error {
                TestResult::Passed
            } else {
                TestResult::PassedWithWrongError(error.clone())
            }
        } else {
            TestResult::PassedWithWrongError(error.clone())
        }
    }

    fn assert_unexpected_no_error(_expected_error: &str, stack_value: StackValue) -> TestResult {
        TestResult::Failed(stack_value)
    }

    fn run_xpath(expr: &qt::XPathExpr) -> Result<StackValue, Error> {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let xpath = XPath::new(&static_context, &expr.0)?;
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        xpath.run(&dynamic_context, None)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::{fs::File, rc::Rc};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_assert_true() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 eq 1</test>
    <result>
      <assert-true />
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_true_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 ne 1</test>
    <result>
      <assert-true />
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::Failed(StackValue::Atomic(Atomic::Boolean(false)))
        );
    }

    #[test]
    fn test_assert_count() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1, 2, 3</test>
    <result>
      <assert-count>3</assert-count>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_expected_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 div 0</test>
    <result>
      <error code="FOAR0001"/>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_error_code_unexpected() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 div 0</test>
    <result>
      <error code="FOAR0002"/>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::PassedWithWrongError(Error::FOAR0001)
        );
    }

    #[test]
    fn test_unexpected_runtime_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 div 0</test>
    <result>
      <assert-true/>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::RuntimeError(Error::FOAR0001));
    }

    #[test]
    fn test_unexpected_compilation_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 @#!</test>
    <result>
      <assert-true/>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::CompilationError(Error::XPST0003 {
                src: "1 @#!".to_string(),
                span: (1, 0).into()
            })
        );
    }

    #[test]
    fn test_expected_compilation_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>1 @#!</test>
    <result>
      <error code="XPST0003" />
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>5</test>
    <result>
      <assert-eq>5</assert-eq>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_passes2() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>4 div 2</test>
    <result>
      <assert-eq>2</assert-eq>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#" 
<test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
  <test-case name="true">
    <description>Description</description>
    <created by="Martijn Faassen" on="2023-05-22"/>
    <environment ref="empty"/>
    <test>5</test>
    <result>
      <assert-eq>6</assert-eq>
    </result>
  </test-case>
  </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::Failed(StackValue::Atomic(Atomic::Integer(5)))
        );
    }

    #[test]
    fn test_assert_any_of_error_case() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>1 div 0</test>
        <result>
          <any-of>
            <assert-eq>2</assert-eq>
            <error code="FOAR0001"/>
          </any-of>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_any_of_eq_case() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>4 div 2</test>
        <result>
          <any-of>
            <assert-eq>2</assert-eq>
            <error code="FOAR0001"/>
          </any-of>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_all_of_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>4 div 2</test>
        <result>
          <all-of>
            <assert-eq>2</assert-eq>
            <assert-count>1</assert-count>
          </all-of>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_all_of_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>4 div 2</test>
        <result>
          <all-of>
            <assert-eq>2</assert-eq>
            <assert-count>8</assert-count>
          </all-of>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::Failed(StackValue::Atomic(Atomic::Integer(2)))
        );
    }

    #[test]
    fn test_assert_string_value_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>"foo"</test>
        <result>
          <assert-string-value>foo</assert-string-value>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_string_value_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>"foo"</test>
        <result>
          <assert-string-value>foo2</assert-string-value>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(
            test_result[0],
            TestResult::Failed(StackValue::Atomic(Atomic::String(Rc::new(
                "foo".to_string()
            ))))
        );
    }

    #[test]
    fn test_assert_string_value_sequence_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
            PathBuf::from("my/test"),
            r#"
    <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
      <test-case name="true">
        <description>Description</description>
        <created by="Martijn Faassen" on="2023-05-22"/>
        <environment ref="empty"/>
        <test>"foo", 3</test>
        <result>
          <assert-string-value>foo 3</assert-string-value>
        </result>
      </test-case>
      </test-set>"#,
        )
        .unwrap();
        let mut catalog_context = CatalogContext::new(xot);
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_external_environment_source() {
        let tmp_dir = tempdir().unwrap();
        let test_cases_path = tmp_dir.path().join("test_cases.xml");
        let mut test_cases_file = File::create(&test_cases_path).unwrap();
        write!(
            test_cases_file,
            r#"
        <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
          <environment name="data">
            <source role="." file="data.xml">
            </source>
          </environment>
          <test-case name="true">
            <description>Description</description>
            <created by="Martijn Faassen" on="2023-05-22"/>
            <environment ref="data"/>
            <test>/doc/p/string()</test>
            <result>
              <assert-string-value>Hello world!</assert-string-value>
            </result>
          </test-case>
          </test-set>"#,
        )
        .unwrap();

        let mut data_file = File::create(tmp_dir.path().join("data.xml")).unwrap();
        write!(data_file, r#"<doc><p>Hello world!</p></doc>"#).unwrap();

        let mut xot = Xot::new();
        let test_set =
            qt::TestSet::load_from_file(&mut xot, tmp_dir.path(), &test_cases_path).unwrap();

        let mut catalog_context = CatalogContext::with_base_dir(xot, tmp_dir.path());
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_local_environment_source() {
        let tmp_dir = tempdir().unwrap();
        let test_cases_path = tmp_dir.path().join("test_cases.xml");
        let mut test_cases_file = File::create(&test_cases_path).unwrap();
        write!(
            test_cases_file,
            r#"
        <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
          <test-case name="true">
            <description>Description</description>
            <created by="Martijn Faassen" on="2023-05-22"/>
            <environment name="data">
              <source role="." file="data.xml">
              </source>
            </environment>
            <test>/doc/p/string()</test>
            <result>
              <assert-string-value>Hello world!</assert-string-value>
            </result>
          </test-case>
          </test-set>"#,
        )
        .unwrap();

        let mut data_file = File::create(tmp_dir.path().join("data.xml")).unwrap();
        write!(data_file, r#"<doc><p>Hello world!</p></doc>"#).unwrap();

        let mut xot = Xot::new();
        let test_set =
            qt::TestSet::load_from_file(&mut xot, tmp_dir.path(), &test_cases_path).unwrap();
        let mut catalog_context = CatalogContext::with_base_dir(xot, tmp_dir.path());
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_xml_passed() {
        let tmp_dir = tempdir().unwrap();
        let test_cases_path = tmp_dir.path().join("test_cases.xml");
        let mut test_cases_file = File::create(&test_cases_path).unwrap();
        write!(
            test_cases_file,
            r#"
        <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
          <environment name="data">
            <source role="." file="data.xml">
            </source>
          </environment>
          <test-case name="true">
            <description>Description</description>
            <created by="Martijn Faassen" on="2023-05-22"/>
            <environment ref="data"/>
            <test>/doc/p</test>
            <result>
              <assert-xml><![CDATA[<p>Hello world!</p>]]></assert-xml>
            </result>
          </test-case>
          </test-set>"#,
        )
        .unwrap();

        let mut data_file = File::create(tmp_dir.path().join("data.xml")).unwrap();
        write!(data_file, r#"<doc><p>Hello world!</p></doc>"#).unwrap();

        let mut xot = Xot::new();
        let test_set =
            qt::TestSet::load_from_file(&mut xot, tmp_dir.path(), &test_cases_path).unwrap();

        let mut catalog_context = CatalogContext::with_base_dir(xot, tmp_dir.path());
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }

    #[test]
    fn test_assert_xml_failed() {
        let tmp_dir = tempdir().unwrap();
        let test_cases_path = tmp_dir.path().join("test_cases.xml");
        let mut test_cases_file = File::create(&test_cases_path).unwrap();
        write!(
            test_cases_file,
            r#"
        <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
          <environment name="data">
            <source role="." file="data.xml">
            </source>
          </environment>
          <test-case name="true">
            <description>Description</description>
            <created by="Martijn Faassen" on="2023-05-22"/>
            <environment ref="data"/>
            <test>/doc/p</test>
            <result>
              <assert-xml><![CDATA[<p>Something else!</p>]]></assert-xml>
            </result>
          </test-case>
          </test-set>"#,
        )
        .unwrap();

        let mut data_file = File::create(tmp_dir.path().join("data.xml")).unwrap();
        write!(data_file, r#"<doc><p>Hello world!</p></doc>"#).unwrap();

        let mut xot = Xot::new();
        let test_set =
            qt::TestSet::load_from_file(&mut xot, tmp_dir.path(), &test_cases_path).unwrap();

        let mut catalog_context = CatalogContext::with_base_dir(xot, tmp_dir.path());
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert!(matches!(test_result[0], TestResult::Failed { .. }));
    }

    #[test]
    fn test_assert_xml_sequence() {
        let tmp_dir = tempdir().unwrap();
        let test_cases_path = tmp_dir.path().join("test_cases.xml");
        let mut test_cases_file = File::create(&test_cases_path).unwrap();
        write!(
            test_cases_file,
            r#"
        <test-set xmlns="http://www.w3.org/2010/09/qt-fots-catalog" name="test">
          <environment name="data">
            <source role="." file="data.xml">
            </source>
          </environment>
          <test-case name="true">
            <description>Description</description>
            <created by="Martijn Faassen" on="2023-05-22"/>
            <environment ref="data"/>
            <test>/doc/p</test>
            <result>
              <assert-xml><![CDATA[<p>Hello world!</p><p>Grabthar's Hammer</p>]]></assert-xml>
            </result>
          </test-case>
          </test-set>"#,
        )
        .unwrap();

        let mut data_file = File::create(tmp_dir.path().join("data.xml")).unwrap();
        write!(
            data_file,
            r#"<doc><p>Hello world!</p><p>Grabthar's Hammer</p></doc>"#
        )
        .unwrap();

        let mut xot = Xot::new();
        let test_set =
            qt::TestSet::load_from_file(&mut xot, tmp_dir.path(), &test_cases_path).unwrap();

        let mut catalog_context = CatalogContext::with_base_dir(xot, tmp_dir.path());
        let test_set_context = TestSetContext::new(&mut catalog_context);
        let test_result = test_set.run(test_set_context).unwrap();
        assert_eq!(test_result.len(), 1);
        assert_eq!(test_result[0], TestResult::Passed);
    }
}

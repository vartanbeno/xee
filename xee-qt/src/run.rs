use miette::Diagnostic;
use xee_xpath::{Atomic, DynamicContext, Error, Namespaces, StackValue, StaticContext, XPath};
use xot::Xot;

use crate::collection::FxIndexSet;
use crate::qt;

#[derive(Debug, Default)]
struct KnownDependencies {
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

// dependency indicator: hashset with type + value keys
// environment: hashmap with environment name as key, empty key should
// always be present. an environment contains a bunch of elements

// if an environment with a schema is referenced, then schema-awareness
// is an implicit dependency

struct TestSetResult<'a> {
    results: Vec<(&'a qt::TestCase, TestResult)>,
}

#[derive(Debug, PartialEq)]
enum TestResult {
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
    Todo,
    // We skipped this test as we don't support the stated
    // dependency
    UnsupportedDependency,
}

impl<'a> qt::TestSet {
    // XXX Make this result an iterator of results?
    fn run(&'a self, known_dependencies: &KnownDependencies) -> TestSetResult<'a> {
        let mut results = Vec::new();
        for test_case in &self.test_cases {
            let result = test_case.run(known_dependencies, &self.shared_environments);
            results.push((test_case, result));
        }
        TestSetResult { results }
    }
}

impl qt::TestCase {
    fn run(
        &self,
        known_dependencies: &KnownDependencies,
        shared_environments: &qt::SharedEnvironments,
    ) -> TestResult {
        for dependency in &self.dependencies {
            if !known_dependencies.is_supported(dependency) {
                return TestResult::UnsupportedDependency;
            }
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
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);

        let value = xpath.run(&dynamic_context, None);
        Self::check_value(&self.result, &value)
    }

    fn check_value(
        result: &qt::TestCaseResult,
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        // we handle any of and all of first, because we don't
        // yet want to distinguish between value and error
        match result {
            qt::TestCaseResult::AllOf(test_case_results) => {
                return Self::assert_all_of(test_case_results, run_result)
            }
            qt::TestCaseResult::AnyOf(test_case_results) => {
                return Self::assert_any_of(test_case_results, run_result)
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
                    qt::TestCaseResult::Error(error) => {
                        Self::assert_unexpected_no_error(error, value)
                    }
                    _ => {
                        panic!("unimplemented test case result")
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
        test_case_results: &[qt::TestCaseResult],
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        for test_case_result in test_case_results {
            let result = Self::check_value(test_case_result, run_result);
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
        test_case_results: &[qt::TestCaseResult],
        run_result: &Result<StackValue, Error>,
    ) -> TestResult {
        for test_case_result in test_case_results {
            let result = Self::check_value(test_case_result, run_result);
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

    use super::*;

    #[test]
    fn test_assert_true() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_true_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
            TestResult::Failed(StackValue::Atomic(Atomic::Boolean(false)))
        );
    }

    #[test]
    fn test_assert_count() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_expected_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_error_code_unexpected() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
            TestResult::PassedWithWrongError(Error::FOAR0001)
        );
    }

    #[test]
    fn test_unexpected_runtime_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
            TestResult::RuntimeError(Error::FOAR0001)
        );
    }

    #[test]
    fn test_unexpected_compilation_error() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_passes2() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_eq_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
            TestResult::Failed(StackValue::Atomic(Atomic::Integer(5)))
        );
    }

    #[test]
    fn test_assert_any_of_error_case() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_any_of_eq_case() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_all_of_passes() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(test_result.results[0].1, TestResult::Passed);
    }

    #[test]
    fn test_assert_all_of_fails() {
        let mut xot = Xot::new();
        let test_set = qt::TestSet::load_from_xml(
            &mut xot,
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
        let known_dependencies = KnownDependencies::default();
        let test_result = test_set.run(&known_dependencies);
        assert_eq!(test_result.results.len(), 1);
        assert_eq!(
            test_result.results[0].1,
            TestResult::Failed(StackValue::Atomic(Atomic::Integer(2)))
        );
    }
}

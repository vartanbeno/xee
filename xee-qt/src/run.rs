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
        // XXX compilation errors can be expected too
        let xpath = match xpath {
            Ok(xpath) => xpath,
            Err(error) => return TestResult::CompilationError(error),
        };
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);

        let run_result = xpath.run(&dynamic_context, None);
        self.check_result(run_result)
    }

    fn check_result(&self, run_result: Result<StackValue, Error>) -> TestResult {
        match run_result {
            Ok(value) => {
                match &self.result {
                    // qt::TestCaseResult::Assert(xpath_expr) => self.assert_(xpath_expr, run_result),
                    qt::TestCaseResult::AssertEq(xpath_expr) => self.assert_eq(xpath_expr, value),
                    qt::TestCaseResult::AssertTrue => self.assert_true(value),
                    qt::TestCaseResult::AssertFalse => self.assert_false(value),
                    qt::TestCaseResult::AssertCount(number) => self.assert_count(*number, value),
                    qt::TestCaseResult::Error(error) => {
                        self.assert_unexpected_no_error(error, value)
                    }
                    _ => {
                        panic!("unimplemented test case result")
                    }
                }
            }
            Err(error) => {
                if let qt::TestCaseResult::Error(expected_error) = &self.result {
                    self.assert_expected_error(expected_error, error)
                } else {
                    TestResult::RuntimeError(error)
                }
            }
        }
    }

    fn assert_(
        &self,
        xpath_expr: &qt::XPathExpr,
        stack_value: Result<StackValue, Error>,
    ) -> TestResult {
        unimplemented!()
    }

    fn assert_eq(&self, xpath_expr: &qt::XPathExpr, stack_value: StackValue) -> TestResult {
        unimplemented!()
    }

    fn assert_true(&self, stack_value: StackValue) -> TestResult {
        if matches!(stack_value, StackValue::Atomic(Atomic::Boolean(true))) {
            TestResult::Passed
        } else {
            TestResult::Failed(stack_value)
        }
    }

    fn assert_false(&self, stack_value: StackValue) -> TestResult {
        if matches!(stack_value, StackValue::Atomic(Atomic::Boolean(false))) {
            TestResult::Passed
        } else {
            TestResult::Failed(stack_value)
        }
    }

    fn assert_count(&self, count: usize, stack_value: StackValue) -> TestResult {
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

    fn assert_expected_error(&self, expected_error: &str, error: Error) -> TestResult {
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

    fn assert_unexpected_no_error(
        &self,
        _expected_error: &str,
        stack_value: StackValue,
    ) -> TestResult {
        TestResult::Failed(stack_value)
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
    <description>True</description>
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
    <description>True</description>
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
}

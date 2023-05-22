use miette::Diagnostic;
use xee_xpath::{Atomic, DynamicContext, Error, Namespaces, StackValue, StaticContext, XPath};
use xot::Xot;

use crate::collection::FxIndexSet;
use crate::qt;

#[derive(Debug)]
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

struct TestSetResult {
    results: Vec<TestResult>,
}

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

impl qt::TestSet {
    fn run(&self, known_dependencies: &KnownDependencies) -> TestSetResult {
        let mut results = Vec::new();
        for test_case in &self.test_cases {
            let result = test_case.run(known_dependencies, &self.shared_environments);
            results.push(result);
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
            Err(error) => return TestResult::CompilationError(error),
        };
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);

        let run_result = xpath.run(&dynamic_context, None);
        self.check_result(run_result)
        // let value = match run_result {
        //     Ok(stack_value) => stack_value,
        //     // XXX need to handle expected errors
        //     Err(_err) => return TestCaseResult::RuntimeError,
        // };
        // execute test
        // compare with result
        // TestCaseResult::Failed
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
                    _ => {
                        panic!("unimplemented test case result")
                    }
                }
            }
            Err(error) => {
                if let qt::TestCaseResult::Error(expected_error) = &self.result {
                    self.assert_error(expected_error, error)
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

    fn assert_error(&self, expected_error: &str, error: Error) -> TestResult {
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
}

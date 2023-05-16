use xee_xpath::{
    Atomic, DynamicContext, Error, Item, Namespaces, StackValue, StaticContext, XPath,
};
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
        let xpath_result = XPath::new(&static_context, &self.test);
        let xpath = match xpath_result {
            Ok(xpath) => xpath,
            Err(error) => return TestResult::CompilationError(error),
        };
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let item = Item::Atomic(Atomic::Integer(0));
        let run_result = xpath.run(&dynamic_context, Some(&item));
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
        match &self.result {
            qt::TestCaseResult::Assert(xpath_expr) => self.assert_(xpath_expr, run_result),
            qt::TestCaseResult::AssertEq(xpath_expr) => self.assert_eq(xpath_expr, run_result),
            _ => {
                panic!("unimplemented test case result")
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

    fn assert_eq(
        &self,
        xpath_expr: &qt::XPathExpr,
        stack_value: Result<StackValue, Error>,
    ) -> TestResult {
        unimplemented!()
    }
}

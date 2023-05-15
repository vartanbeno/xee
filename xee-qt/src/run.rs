use ahash::HashSet;

use crate::qt;

#[derive(Debug)]
struct KnownDependencies {
    specs: HashSet<qt::DependencySpec>,
}

impl KnownDependencies {
    fn new(specs: HashSet<qt::DependencySpec>) -> Self {
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
    results: Vec<TestCaseResult>,
}

enum TestCaseResult {
    Passed,
    Failed,
    Errored,
    Unsupported,
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
    ) -> TestCaseResult {
        for dependency in &self.dependencies {
            if !known_dependencies.is_supported(dependency) {
                return TestCaseResult::UnsupportedDependency;
            }
        }

        // execute test
        // compare with result
        TestCaseResult::Failed
    }
}

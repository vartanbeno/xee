use ahash::HashSet;

use crate::qt;

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

enum TestCaseResult {
    Passed,
    Failed,
    Unsupported,
    UnsupportedDependency,
}

impl qt::TestCase {
    // run should take a bunch of environments and dependencies
    // under which it is run
    fn run(&self, known_dependencies: &KnownDependencies) -> TestCaseResult {
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

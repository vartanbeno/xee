use crate::qt;

// dependency indicator: hashset with type + value keys
// environment: hashmap with environment name as key, empty key should
// always be present. an environment contains a bunch of elements

// if an environment with a schema is referenced, then schema-awareness
// is an implicit dependency

impl qt::TestCase {
    // run should take a bunch of environments and dependencies
    // under which it is run
    fn run(&self) -> bool {
        // execute test
        // compare with result
        false
    }
}

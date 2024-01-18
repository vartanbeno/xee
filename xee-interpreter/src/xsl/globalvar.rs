use std::rc::Rc;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

struct Error {}
type Resolver = dyn Fn(Rc<dyn Fn(&str) -> Option<u64>>) -> Option<u64>;

struct GlobalVariables {
    declarations: HashSet<String>,
    resolvers: HashMap<String, Rc<Resolver>>,
}

impl GlobalVariables {
    fn new() -> Self {
        Self {
            declarations: HashSet::new(),
            resolvers: HashMap::new(),
        }
    }

    fn add_declaration(&mut self, name: &str) {
        self.declarations.insert(name.to_string());
    }

    fn add_resolver(&mut self, name: &str, resolver: Rc<Resolver>) {
        self.resolvers.insert(name.to_string(), resolver);
    }

    fn get(self: Rc<Self>, name: &str) -> Option<u64> {
        self.get_internal(name, HashSet::new())
    }

    fn get_internal(self: Rc<Self>, name: &str, seen: HashSet<String>) -> Option<u64> {
        let s = self.clone();
        let resolve = s.resolvers.get(name)?;
        if seen.contains(name) {
            return None;
        }
        let name_seen = name.to_string();

        resolve(Rc::new(move |name: &str| {
            let mut new_seen = seen.clone();
            new_seen.insert(name_seen.clone());
            self.clone().get_internal(name, new_seen)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_global_variable() {
        // first declare a few global variables
        let mut global_variables = GlobalVariables::new();
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        // now something that uses the global variables
        global_variables.add_resolver("bar", Rc::new(|_| Some(2)));
        global_variables.add_resolver("foo", Rc::new(|resolve| Some(resolve("bar")? + 1)));

        // now we can resolve foo and bar
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.clone().get("foo"), Some(3));
        assert_eq!(global_variables.get("bar"), Some(2));
    }

    #[test]
    fn test_circular() {
        // first declare a few global variables
        let mut global_variables = GlobalVariables::new();
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        // now something that uses the global variables
        global_variables.add_resolver("bar", Rc::new(|resolve| resolve("foo")));
        global_variables.add_resolver("foo", Rc::new(|resolve| Some(resolve("bar")? + 1)));

        // now we can resolve foo but resolution fails as there is a circular dependency
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.clone().get("foo"), None);
    }
}

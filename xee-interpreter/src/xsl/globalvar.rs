use std::rc::Rc;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

trait Resolver {
    fn resolve(self: Rc<Self>, name: &str) -> Option<u64>;
}

// type Resolver = dyn Fn(Rc<dyn Fn(&str) -> Option<u64>>) -> Option<u64>;

struct GlobalVariables {
    declarations: HashSet<String>,
    resolvers: HashMap<String, Rc<dyn Fn(Rc<dyn Resolver>) -> Option<u64>>>,
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

    fn add_resolver(&mut self, name: &str, resolver: Rc<dyn Fn(Rc<dyn Resolver>) -> Option<u64>>) {
        self.resolvers.insert(name.to_string(), resolver);
    }

    fn get(self: Rc<Self>, name: &str) -> Option<u64> {
        let resolve = self.resolvers.get(name)?;
        resolve(self.clone())
    }
}

impl Resolver for GlobalVariables {
    fn resolve(self: Rc<Self>, name: &str) -> Option<u64> {
        self.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_global_variable() {
        // first declare a few global variables
        let mut global_variables = Box::new(GlobalVariables::new());
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        // now something that uses the global variables
        global_variables.add_resolver("bar", Rc::new(|_| Some(2)));
        global_variables.add_resolver(
            "foo",
            Rc::new(|resolver| Some(resolver.resolve("bar")? + 1)),
        );

        // now we can resolve foo and bar
        let global_variables = Rc::new(*global_variables);
        assert_eq!(global_variables.clone().get("foo"), Some(3));
        assert_eq!(global_variables.get("bar"), Some(2));
    }
}

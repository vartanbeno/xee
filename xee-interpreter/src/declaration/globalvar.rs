use std::rc::Rc;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::error::{Error, Result};

type Resolver<V> = dyn Fn(Rc<dyn Fn(&str) -> Result<V>>) -> Result<V>;

struct GlobalVariables<V: 'static> {
    declarations: HashSet<String>,
    resolvers: HashMap<String, Rc<Resolver<V>>>,
}

impl<V: 'static> GlobalVariables<V> {
    fn new() -> Self {
        Self {
            declarations: HashSet::new(),
            resolvers: HashMap::new(),
        }
    }

    fn add_declaration(&mut self, name: &str) {
        self.declarations.insert(name.to_string());
    }

    fn add_resolver(&mut self, name: &str, resolver: Rc<Resolver<V>>) {
        self.resolvers.insert(name.to_string(), resolver);
    }

    fn get(self: &Rc<Self>, name: &str) -> Result<V> {
        self.get_internal(name, HashSet::new())
    }

    fn get_internal(self: &Rc<Self>, name: &str, seen: HashSet<String>) -> Result<V> {
        let resolve = self.resolvers.get(name).unwrap();
        if seen.contains(name) {
            return Err(Error::XTDE0640);
        }

        let s = self.clone();
        let name_seen = name.to_string();
        resolve(Rc::new(move |name: &str| {
            let mut new_seen = seen.clone();
            new_seen.insert(name_seen.clone());
            s.get_internal(name, new_seen)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_global_variable() {
        // first declare a few global variables
        let mut global_variables = GlobalVariables::<u64>::new();
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        // now something that uses the global variables
        global_variables.add_resolver("bar", Rc::new(|_| Ok(2)));
        global_variables.add_resolver("foo", Rc::new(|resolve| Ok(resolve("bar")? + 1)));

        // now we can resolve foo and bar
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.get("foo"), Ok(3));
        assert_eq!(global_variables.get("bar"), Ok(2));
    }

    #[test]
    fn test_circular() {
        // first declare a few global variables
        let mut global_variables = GlobalVariables::<u64>::new();
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        // now something that uses the global variables
        global_variables.add_resolver("bar", Rc::new(|resolve| resolve("foo")));
        global_variables.add_resolver("foo", Rc::new(|resolve| Ok(resolve("bar")? + 1)));

        // now we can resolve foo but resolution fails as there is a circular dependency
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.get("foo"), Err(Error::XTDE0640));
    }
}

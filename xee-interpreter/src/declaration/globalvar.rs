use std::{cell::RefCell, rc::Rc};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::error::{Error, Result};

type Resolver<V> = dyn Fn(Box<dyn Fn(&str) -> Result<V>>) -> Result<V>;

struct GlobalVariables<V: Clone + 'static> {
    declarations: HashSet<String>,
    resolvers: HashMap<String, Box<Resolver<V>>>,
    resolved: RefCell<HashMap<String, V>>,
}

impl<V: Clone + 'static> GlobalVariables<V> {
    fn new() -> Self {
        Self {
            declarations: HashSet::new(),
            resolvers: HashMap::new(),
            resolved: RefCell::new(HashMap::new()),
        }
    }

    fn add_declaration(&mut self, name: &str) {
        self.declarations.insert(name.to_string());
    }

    fn add_resolver(&mut self, name: &str, resolver: Box<Resolver<V>>) {
        self.resolvers.insert(name.to_string(), resolver);
    }

    fn get(self: &Rc<Self>, name: &str) -> Result<V> {
        self.get_internal(name, HashSet::new())
    }

    fn get_internal(self: &Rc<Self>, name: &str, seen: HashSet<String>) -> Result<V> {
        if let Some(value) = self.resolved.borrow().get(name) {
            return Ok(value.clone());
        }
        let resolve = self.resolvers.get(name).unwrap();
        if seen.contains(name) {
            return Err(Error::XTDE0640);
        }

        let s = self.clone();
        let name_seen = name.to_string();
        let value = resolve(Box::new(move |name: &str| {
            let mut new_seen = seen.clone();
            new_seen.insert(name_seen.clone());
            s.get_internal(name, new_seen)
        }))?;
        let mut resolved = self.resolved.borrow_mut();
        resolved.insert(name.to_string(), value.clone());
        Ok(value)
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
        global_variables.add_resolver("bar", Box::new(|_| Ok(2)));
        global_variables.add_resolver("foo", Box::new(|resolve| Ok(resolve("bar")? + 1)));

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
        global_variables.add_resolver("bar", Box::new(|resolve| resolve("foo")));
        global_variables.add_resolver("foo", Box::new(|resolve| Ok(resolve("bar")? + 1)));

        // now we can resolve foo but resolution fails as there is a circular dependency
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.get("foo"), Err(Error::XTDE0640));
    }

    #[test]
    fn test_cache() {
        // first declare a few global variables
        let mut global_variables = GlobalVariables::<u64>::new();
        global_variables.add_declaration("foo");
        global_variables.add_declaration("bar");

        struct Counter {
            count: RefCell<usize>,
        }
        impl Counter {
            fn plus(&self) {
                let mut c = self.count.borrow_mut();
                (*c) += 1;
            }
            fn get(&self) -> usize {
                *self.count.borrow()
            }
        }
        let counter = Rc::new(Counter {
            count: RefCell::new(0),
        });
        let current_counter = counter.clone();
        global_variables.add_resolver(
            "foo",
            Box::new(move |_resolve| {
                current_counter.plus();
                Ok(1)
            }),
        );

        // now we can resolve foo and if we resolve it twice, the resolver is only called once
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.get("foo"), Ok(1));
        assert_eq!(global_variables.get("foo"), Ok(1));
        assert_eq!(counter.get(), 1);
    }
}

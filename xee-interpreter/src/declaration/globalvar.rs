use std::{cell::RefCell, rc::Rc};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::error::{Error, Result};

type Resolver<'a, V> = dyn Fn(Box<dyn Fn(&'a str) -> Result<V> + 'a>) -> Result<V> + 'a;

struct GlobalVariables<'a, V: Clone + 'a> {
    declarations: HashSet<String>,
    resolvers: HashMap<String, Box<Resolver<'a, V>>>,
    resolved: RefCell<HashMap<String, V>>,
}

impl<'a, V: Clone + 'a> GlobalVariables<'a, V> {
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

    fn add_resolver<F>(&mut self, name: &str, resolver: F)
    where
        F: Fn(Box<dyn Fn(&'a str) -> Result<V> + 'a>) -> Result<V> + 'a,
    {
        self.resolvers.insert(name.to_string(), Box::new(resolver));
    }

    fn get(self: &Rc<Self>, name: &'a str) -> Result<V> {
        self.get_internal(name, HashSet::new())
    }

    fn get_resolve(
        self: &Rc<Self>,
        name_seen: &'a str,
        seen: HashSet<String>,
    ) -> Box<dyn Fn(&'a str) -> Result<V> + 'a> {
        let s = self.clone();
        Box::new(move |name: &'a str| {
            let mut new_seen = seen.clone();
            new_seen.insert(name_seen.to_string());
            s.get_internal(name, new_seen)
        })
    }

    fn get_internal(self: &Rc<Self>, name: &'a str, seen: HashSet<String>) -> Result<V> {
        if let Some(value) = self.resolved.borrow().get(name) {
            return Ok(value.clone());
        }
        let resolve = self.resolvers.get(name).unwrap();
        if seen.contains(name) {
            return Err(Error::XTDE0640);
        }

        let value = resolve(self.get_resolve(name, seen))?;

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
        global_variables.add_resolver("bar", |_| Ok(2));
        global_variables.add_resolver("foo", |resolve| Ok(resolve("bar")? + 1));

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
        global_variables.add_resolver("bar", |resolve| resolve("foo"));
        global_variables.add_resolver("foo", |resolve| Ok(resolve("bar")? + 1));

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
        global_variables.add_resolver("foo", move |_resolve| {
            current_counter.plus();
            Ok(1)
        });

        // now we can resolve foo and if we resolve it twice, the resolver is only called once
        let global_variables = Rc::new(global_variables);
        assert_eq!(global_variables.get("foo"), Ok(1));
        assert_eq!(global_variables.get("foo"), Ok(1));
        assert_eq!(counter.get(), 1);
    }
}

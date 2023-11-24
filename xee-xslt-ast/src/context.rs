use ahash::{HashMap, HashMapExt};
use xee_xpath_ast::{Namespaces, FN_NAMESPACE};

use crate::state::State;

/// Parser context is passed around. You can create new contexts as
/// for particular sub-trees.
pub(crate) struct Context<'a> {
    prefixes: xot::Prefixes,

    next: Option<&'a Context<'a>>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(element: &xot::Element) -> Self {
        Self {
            prefixes: element.prefixes().clone(),
            next: None,
        }
    }

    pub(crate) fn element(&'a self, element: &'a xot::Element) -> Self {
        Self {
            prefixes: element.prefixes().clone(),
            next: Some(self),
        }
    }

    fn prefixes(&self) -> xot::Prefixes {
        if let Some(next) = &self.next {
            let mut combined_prefixes = xot::Prefixes::new();
            let prefixes = next.prefixes();
            for (prefix, uri) in prefixes.iter() {
                combined_prefixes.insert(*prefix, *uri);
            }
            combined_prefixes
        } else {
            self.prefixes.clone()
        }
    }

    pub(crate) fn namespaces(&self, state: &'a State) -> Namespaces {
        let mut namespaces = HashMap::new();
        for (prefix, ns) in self.prefixes() {
            let prefix = state.xot.prefix_str(prefix);
            let uri = state.xot.namespace_str(ns);
            namespaces.insert(prefix, uri);
        }
        Namespaces::new(namespaces, None, Some(FN_NAMESPACE))
    }
}

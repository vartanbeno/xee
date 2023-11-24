use ahash::{HashMap, HashMapExt};
use xee_xpath_ast::{Namespaces, FN_NAMESPACE};

use crate::state::State;

struct StackedContext<'a> {
    element: &'a xot::Element,

    next: Option<&'a StackedContext<'a>>,
}

impl<'a> StackedContext<'a> {
    pub(crate) fn new(element: &'a xot::Element) -> Self {
        Self {
            element,
            next: None,
        }
    }

    pub(crate) fn push(&'a self, element: &'a xot::Element) -> Self {
        Self {
            element,
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
            self.element.prefixes().clone()
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

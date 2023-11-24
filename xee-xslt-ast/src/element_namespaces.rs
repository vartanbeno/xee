use ahash::{HashMap, HashMapExt};
use xee_xpath_ast::{Namespaces, FN_NAMESPACE};
use xot::Xot;

// TODO: I think Xot has a way to get the namespaces for an element
// already, so we should use just that.
pub(crate) struct ElementNamespaces<'a> {
    xot: &'a Xot,
    element: &'a xot::Element,
    next: Option<&'a ElementNamespaces<'a>>,
}

impl<'a> ElementNamespaces<'a> {
    pub(crate) fn new(xot: &'a Xot, element: &'a xot::Element) -> Self {
        Self {
            xot,
            element,
            next: None,
        }
    }

    pub(crate) fn push(&'a self, element: &'a xot::Element) -> Self {
        Self {
            xot: self.xot,
            element,
            next: Some(self),
        }
    }

    fn pop(self) -> Option<&'a Self> {
        self.next
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

    pub(crate) fn namespaces(&self) -> Namespaces {
        let mut namespaces = HashMap::new();
        for (prefix, ns) in self.prefixes() {
            let prefix = self.xot.prefix_str(prefix);
            let uri = self.xot.namespace_str(ns);
            namespaces.insert(prefix, uri);
        }
        Namespaces::new(namespaces, None, Some(FN_NAMESPACE))
    }
}

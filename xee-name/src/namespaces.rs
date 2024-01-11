use ahash::{HashMap, HashMapExt};

pub const FN_NAMESPACE: &str = "http://www.w3.org/2005/xpath-functions";
pub const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

const STATIC_NAMESPACES: [(&str, &str); 7] = [
    ("xs", XS_NAMESPACE),
    ("fn", FN_NAMESPACE),
    ("math", "http://www.w3.org/2005/xpath-functions/math"),
    ("map", "http://www.w3.org/2005/xpath-functions/map"),
    ("array", "http://www.w3.org/2005/xpath-functions/array"),
    ("err", "http://www.w3.org/2005/xqt-errors"),
    ("output", "http://www.w3.org/2010/xslt-xquery-serialization"),
];

#[derive(Debug, Clone)]
pub struct Namespaces<'a> {
    namespaces: HashMap<&'a str, &'a str>,
    pub default_element_namespace: Option<&'a str>,
    pub default_function_namespace: Option<&'a str>,
}

impl<'a> Namespaces<'a> {
    pub const FN_NAMESPACE: &'static str = FN_NAMESPACE;

    pub fn new(
        namespaces: HashMap<&'a str, &'a str>,
        default_element_namespace: Option<&'a str>,
        default_function_namespace: Option<&'a str>,
    ) -> Self {
        Self {
            namespaces,
            default_element_namespace,
            default_function_namespace,
        }
    }

    pub fn default_namespaces() -> HashMap<&'a str, &'a str> {
        let mut namespaces = HashMap::new();
        namespaces.insert("xml", XML_NAMESPACE);
        for (prefix, uri) in STATIC_NAMESPACES.into_iter() {
            namespaces.insert(prefix, uri);
        }
        namespaces
    }

    pub fn add(&mut self, namespace_pairs: &[(&'a str, &'a str)]) {
        for (prefix, uri) in namespace_pairs {
            if prefix.is_empty() {
                self.default_element_namespace = Some(uri);
            } else {
                self.namespaces.insert(*prefix, *uri);
            }
        }
    }

    pub fn by_prefix(&self, prefix: &str) -> Option<&str> {
        self.namespaces.get(prefix).copied()
    }

    pub fn default_element_namespace(&self) -> Option<&str> {
        self.default_element_namespace
    }
}

impl Default for Namespaces<'_> {
    fn default() -> Self {
        Self::new(Self::default_namespaces(), None, Some(FN_NAMESPACE))
    }
}

pub trait NamespaceLookup {
    fn by_prefix(&self, prefix: &str) -> Option<&str>;
}

impl NamespaceLookup for Namespaces<'_> {
    fn by_prefix(&self, prefix: &str) -> Option<&str> {
        self.namespaces.get(prefix).copied()
    }
}

impl<T: NamespaceLookup> NamespaceLookup for &T {
    fn by_prefix(&self, prefix: &str) -> Option<&str> {
        (**self).by_prefix(prefix)
    }
}

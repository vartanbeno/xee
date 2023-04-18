use ahash::{HashMap, HashMapExt};

pub(crate) const FN_NAMESPACE: &str = "http://www.w3.org/2005/xpath-functions";

const STATIC_NAMESPACES: [(&str, &str); 7] = [
    ("xs", "http://www.w3.org/2001/XMLSchema"),
    ("fn", FN_NAMESPACE),
    ("math", "http://www.w3.org/2005/xpath-functions/math"),
    ("map", "http://www.w3.org/2005/xpath-functions/map"),
    ("array", "http://www.w3.org/2005/xpath-functions/array"),
    ("err", "http://www.w3.org/2005/xqt-errors"),
    ("output", "http://www.w3.org/2010/xslt-xquery-serialization"),
];

pub(crate) struct Namespaces<'a> {
    namespaces: HashMap<&'a str, &'a str>,
}

impl<'a> Namespaces<'a> {
    pub(crate) fn new() -> Self {
        let mut namespaces = HashMap::new();
        for (prefix, uri) in STATIC_NAMESPACES.into_iter() {
            namespaces.insert(prefix, uri);
        }
        Self { namespaces }
    }

    pub(crate) fn by_prefix(&self, prefix: &str) -> Option<&str> {
        self.namespaces.get(prefix).copied()
    }
}

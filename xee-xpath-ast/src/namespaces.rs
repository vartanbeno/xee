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
    pub(crate) default_element_namespace: Option<&'a str>,
    pub(crate) default_function_namespace: Option<&'a str>,
}

impl<'a> Namespaces<'a> {
    pub fn new(
        default_element_namespace: Option<&'a str>,
        default_function_namespace: Option<&'a str>,
    ) -> Self {
        let mut namespaces = HashMap::new();
        namespaces.insert("xml", XML_NAMESPACE);
        for (prefix, uri) in STATIC_NAMESPACES.into_iter() {
            namespaces.insert(prefix, uri);
        }
        Self {
            namespaces,
            default_element_namespace,
            default_function_namespace,
        }
    }

    pub fn with_default_element_namespace(uri: &'a str) -> Self {
        Self::new(Some(uri), Some(FN_NAMESPACE))
    }

    pub(crate) fn by_prefix(&self, prefix: &str) -> Option<&str> {
        self.namespaces.get(prefix).copied()
    }
}

impl Default for Namespaces<'_> {
    fn default() -> Self {
        Self::new(None, Some(FN_NAMESPACE))
    }
}

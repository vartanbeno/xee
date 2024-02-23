use std::path::PathBuf;

use super::{
    core::{Environment, EnvironmentSpec},
    decimal_format::DecimalFormat,
};

#[derive(Debug, Clone)]
pub(crate) struct XPathEnvironmentSpec {
    environment_spec: EnvironmentSpec,

    pub(crate) decimal_formats: Vec<DecimalFormat>,
    pub(crate) namespaces: Vec<Namespace>,
    pub(crate) context_items: Vec<ContextItem>,
    pub(crate) static_base_uris: Vec<StaticBaseUri>,
}

// Only is used by some XPath tests, not by XSLT
#[derive(Debug, Clone)]
pub(crate) struct ContextItem {
    pub(crate) select: String,
}

// only in XPath, not in use by XSLT
#[derive(Debug, Clone)]
pub(crate) struct Namespace {
    pub(crate) prefix: String,
    pub(crate) uri: String,
}

// // Does not appear to be in use by either XPath or XSLT test suites
// #[derive(Debug, Clone)]
// pub(crate) struct FunctionLibrary {}

// Only in use by the XPath test suite
#[derive(Debug, Clone)]
pub(crate) struct StaticBaseUri {
    uri: Option<String>,
}

impl XPathEnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            environment_spec: EnvironmentSpec::empty(),
            decimal_formats: vec![],
            namespaces: vec![],
            context_items: vec![],
            static_base_uris: vec![],
        }
    }

    pub(crate) fn namespace_pairs(&self) -> Vec<(&str, &str)> {
        self.namespaces
            .iter()
            .map(|ns| (ns.prefix.as_ref(), ns.uri.as_ref()))
            .collect()
    }
}

impl Environment for XPathEnvironmentSpec {
    fn empty() -> Self {
        Self::empty()
    }

    fn environment_spec(&self) -> &EnvironmentSpec {
        &self.environment_spec
    }
}

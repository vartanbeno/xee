use std::path::PathBuf;

use crate::{decimal_format::DecimalFormat, environment::EnvironmentSpec};

#[derive(Debug, Clone)]
pub(crate) struct XPathEnvironmentSpec {
    environment_spec: EnvironmentSpec,

    pub(crate) base_dir: PathBuf,
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

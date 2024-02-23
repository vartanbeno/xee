use super::{resource::Resource, source::Source};

#[derive(Debug, Clone)]
pub(crate) struct Collection {
    uri: Option<String>,
    query: Vec<Query>,
    source: Vec<Source>,
    // this doesn't appear to be in use by XPath or XSLT test suites
    resource: Vec<Resource>,
}

#[derive(Debug, Clone)]
pub(crate) struct Query {
    expression: String,
}

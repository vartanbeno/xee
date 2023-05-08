use xot::Xot;

use crate::context::Context;
use crate::document::{Documents, Uri};
use crate::error::Result;
use crate::name::{Namespaces, FN_NAMESPACE};
use crate::static_context::StaticContext;
use crate::value::StackValue;
use crate::xpath::CompiledXPath;

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(
    xml: &str,
    xpath: &str,
    default_element_namespace: Option<&str>,
) -> Result<StackValue> {
    let mut xot = Xot::new();
    let uri = Uri("http://example.com".to_string());
    let mut documents = Documents::new();
    documents.add(&mut xot, &uri, xml).unwrap();
    let namespaces = Namespaces::new(default_element_namespace, Some(FN_NAMESPACE));
    let static_context = StaticContext::new(&namespaces);
    let context = Context::with_documents(&xot, xpath, static_context, &documents);
    let document = documents.get(&uri).unwrap();

    let xpath = CompiledXPath::new(&context, xpath)?;
    xpath.run_xot_node(document.root)
}

pub fn run_without_context(s: &str) -> Result<StackValue> {
    let xot = Xot::new();
    let namespaces = Namespaces::new(None, None);
    let static_context = StaticContext::new(&namespaces);
    let context = Context::new(&xot, s, static_context);
    let xpath = CompiledXPath::new(&context, s)?;
    xpath.run_without_context()
}

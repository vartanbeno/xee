use xot::Xot;

use xee_xpath_ast::{Namespaces, FN_NAMESPACE};

use crate::context::{DynamicContext, StaticContext};
use crate::error::Result;
use crate::output2 as output;
use crate::xml;
use crate::xpath::XPath;

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(
    xml: &str,
    xpath: &str,
    default_element_namespace: Option<&str>,
) -> Result<output::Sequence> {
    let mut xot = Xot::new();
    let root = xot.parse(xml).unwrap();
    evaluate_root(&xot, root, xpath, default_element_namespace)
}

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate_root(
    xot: &Xot,
    root: xot::Node,
    xpath: &str,
    default_element_namespace: Option<&str>,
) -> Result<output::Sequence> {
    let uri = xml::Uri("http://example.com".to_string());
    let mut documents = xml::Documents::new();
    documents.add_root(xot, &uri, root);
    let namespaces = Namespaces::new(default_element_namespace, Some(FN_NAMESPACE));
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::with_documents(xot, &static_context, &documents);
    let document = documents.get(&uri).unwrap();

    let xpath = XPath::new(context.static_context, xpath)?;
    xpath.many_xot_node(&context, document.root)
}

pub fn evaluate_without_focus(s: &str) -> Result<output::Sequence> {
    let xot = Xot::new();
    let namespaces = Namespaces::new(None, None);
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::new(&xot, &static_context);
    let xpath = XPath::new(context.static_context, s)?;
    xpath.many(&context, None)
}

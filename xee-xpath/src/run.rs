use xot::Xot;

use xee_xpath_ast::{ast, Namespaces, FN_NAMESPACE};

use crate::context::{DynamicContext, StaticContext};
use crate::error::Result;
use crate::sequence;
use crate::xml;
use crate::xpath::XPath;

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(
    xml: &str,
    xpath: &str,
    default_element_namespace: Option<&str>,
) -> Result<sequence::Sequence> {
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
) -> Result<sequence::Sequence> {
    let uri = xml::Uri("http://example.com".to_string());
    let mut documents = xml::Documents::new();
    documents.add_root(xot, &uri, root);
    let namespaces = Namespaces::new(default_element_namespace, Some(FN_NAMESPACE));
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::with_documents(xot, &static_context, &documents);
    let document = documents.get(&uri).unwrap();

    let xpath = XPath::new(context.static_context, xpath)?;
    xpath.runnable(&context).many_xot_node(document.root)
}

pub fn evaluate_without_focus(s: &str) -> Result<sequence::Sequence> {
    let xot = Xot::new();
    let namespaces = Namespaces::default();
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::new(&xot, &static_context);
    let xpath = XPath::new(context.static_context, s)?;
    xpath.runnable(&context).many(None)
}

pub fn evaluate_without_focus_with_variables(
    s: &str,
    variables: &[(ast::Name, Vec<sequence::Item>)],
) -> Result<sequence::Sequence> {
    let xot = Xot::new();
    let namespaces = Namespaces::default();
    let variable_names = variables
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let static_context = StaticContext::with_variable_names(&namespaces, &variable_names);
    let context = DynamicContext::with_variables(&xot, &static_context, variables);
    let xpath = XPath::new(context.static_context, s)?;
    xpath.runnable(&context).many(None)
}

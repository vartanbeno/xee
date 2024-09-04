use xot::Xot;

use xee_interpreter::{
    context::DynamicContext, context::StaticContext, context::Variables, error::SpannedResult,
    sequence::Sequence, xml::Documents, xml::Uri,
};
use xee_xpath_ast::{Namespaces, FN_NAMESPACE};

use crate::compile::parse;

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(
    xml: &str,
    xpath: &str,
    default_element_namespace: &str,
) -> SpannedResult<Sequence> {
    let mut xot = Xot::new();
    let root = xot.parse(xml).unwrap();
    evaluate_root(&mut xot, root, xpath, default_element_namespace)
}

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate_root(
    xot: &mut Xot,
    root: xot::Node,
    xpath: &str,
    default_element_namespace: &str,
) -> SpannedResult<Sequence> {
    let namespaces = Namespaces::new(
        Namespaces::default_namespaces(),
        default_element_namespace,
        FN_NAMESPACE,
    );
    let static_context = StaticContext::from_namespaces(namespaces);
    // TODO: isn't the right URI
    let uri = Uri::new("http://example.com");
    let mut documents = Documents::new();
    documents.add_root(xot, &uri, root);
    let root = documents.get(&uri).unwrap().root;
    let context = DynamicContext::from_documents(static_context, documents);

    let program = parse(&context.static_context, xpath)?;
    let runnable = program.runnable(&context);
    runnable.many_xot_node(root, xot)
}

pub fn evaluate_without_focus(s: &str) -> SpannedResult<Sequence> {
    let mut xot = Xot::new();
    let static_context = StaticContext::default();
    let context = DynamicContext::empty(static_context);

    let program = parse(&context.static_context, s)?;
    let runnable = program.runnable(&context);
    runnable.many(None, &mut xot)
}

pub fn evaluate_without_focus_with_variables(
    s: &str,
    variables: Variables,
) -> SpannedResult<Sequence> {
    let mut xot = Xot::new();
    let namespaces = Namespaces::default();
    let variable_names = variables.keys().cloned().collect();
    let static_context = StaticContext::new(namespaces, variable_names);
    let context = DynamicContext::from_variables(static_context, &variables);
    let program = parse(&context.static_context, s)?;
    let runnable = program.runnable(&context);
    runnable.many(None, &mut xot)
}

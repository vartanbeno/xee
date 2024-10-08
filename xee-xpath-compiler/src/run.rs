use xot::Xot;

use xee_interpreter::{
    context::{DynamicContextBuilder, StaticContext, StaticContextBuilder, Variables},
    error::SpannedResult,
    sequence::Sequence,
    xml::{Documents, Uri},
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
        default_element_namespace.to_string(),
        FN_NAMESPACE.to_string(),
    );
    let static_context = StaticContext::from_namespaces(namespaces);
    // TODO: isn't the right URI
    let uri = Uri::new("http://example.com");
    let mut documents = Documents::new();
    // TODO: The unwrap here is bad, but DocumentsError isn't integrated int
    // the general error system yet
    documents.add_root(xot, &uri, root).unwrap();

    let program = parse(static_context, xpath)?;

    let mut dynamic_context_builder = DynamicContextBuilder::new(program.static_context());
    dynamic_context_builder.context_node(root);
    dynamic_context_builder.documents(documents);
    let context = dynamic_context_builder.build();

    let runnable = program.runnable(&context);
    runnable.many(xot)
}

pub fn evaluate_without_focus(s: &str) -> SpannedResult<Sequence> {
    let mut xot = Xot::new();
    let static_context = StaticContext::default();

    let program = parse(static_context, s)?;

    let dynamic_context_buidler = DynamicContextBuilder::new(program.static_context());
    let context = dynamic_context_buidler.build();
    let runnable = program.runnable(&context);
    runnable.many(&mut xot)
}

pub fn evaluate_without_focus_with_variables(
    s: &str,
    variables: Variables,
) -> SpannedResult<Sequence> {
    let mut xot = Xot::new();
    let mut builder = StaticContextBuilder::default();
    let variable_names = variables.keys().cloned();
    builder.variable_names(variable_names);
    let static_context = builder.build();

    let program = parse(static_context, s)?;

    let mut dynamic_context_builder = DynamicContextBuilder::new(program.static_context());
    dynamic_context_builder.variables(variables);
    let context = dynamic_context_builder.build();
    let runnable = program.runnable(&context);
    runnable.many(&mut xot)
}

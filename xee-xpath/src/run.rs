use xot::Xot;

use xee_xpath_ast::{ast, Namespaces, FN_NAMESPACE};

use crate::context::{DynamicContext, StaticContext};
use crate::error;
use crate::xml;
use crate::{interpreter, sequence};

/// A high level function that evaluates an xpath expression on an xml document.
pub fn evaluate(
    xml: &str,
    xpath: &str,
    default_element_namespace: Option<&str>,
) -> error::SpannedResult<sequence::Sequence> {
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
) -> error::SpannedResult<sequence::Sequence> {
    let uri = xml::Uri("http://example.com".to_string());
    let mut documents = xml::Documents::new();
    documents.add_root(xot, &uri, root);
    let namespaces = Namespaces::new(
        Namespaces::default_namespaces(),
        default_element_namespace,
        Some(FN_NAMESPACE),
    );
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::with_documents(xot, &static_context, &documents);
    let document = documents.get(&uri).unwrap();

    let program = interpreter::Program::new(context.static_context, xpath)?;
    let runnable = program.runnable(&context);
    runnable.many_xot_node(document.root)
}

pub fn evaluate_without_focus(s: &str) -> error::SpannedResult<sequence::Sequence> {
    let xot = Xot::new();
    let namespaces = Namespaces::default();
    let static_context = StaticContext::new(&namespaces);
    let context = DynamicContext::new(&xot, &static_context);

    let program = interpreter::Program::new(context.static_context, s)?;
    let runnable = program.runnable(&context);
    runnable.many(None)
}

pub fn evaluate_without_focus_with_variables(
    s: &str,
    variables: &[(ast::Name, Vec<sequence::Item>)],
) -> error::SpannedResult<sequence::Sequence> {
    let xot = Xot::new();
    let namespaces = Namespaces::default();
    let variable_names = variables
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let static_context = StaticContext::with_variable_names(&namespaces, &variable_names);
    let context = DynamicContext::with_variables(&xot, &static_context, variables);
    let program = interpreter::Program::new(context.static_context, s)?;
    let runnable = program.runnable(&context);
    runnable.many(None)
}

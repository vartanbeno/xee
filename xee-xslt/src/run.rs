use xee_interpreter::context::DynamicContext;
use xee_interpreter::context::StaticContext;
use xee_interpreter::error;
use xee_interpreter::interpreter::Program;
use xee_interpreter::sequence;
use xee_interpreter::xml;
use xee_name::{Namespaces, FN_NAMESPACE};
use xot::{Node, Xot};

use crate::ast_ir::parse;

pub fn evaluate_program(
    xot: &mut Xot,
    program: &Program,
    root: Node,
    static_context: &StaticContext,
) -> error::SpannedResult<sequence::Sequence> {
    let uri = xee_interpreter::xml::Uri::new("http://example.com");
    let mut documents = xee_interpreter::xml::Documents::new();
    documents.add_root(xot, &uri, root);
    let context = DynamicContext::from_documents(static_context, &documents);
    let document = documents.get(&uri).unwrap();
    let runnable = program.runnable(&context);
    let item: sequence::Item = document.root.into();
    runnable.many(Some(&item), xot)
}

pub fn evaluate(xot: &mut Xot, xml: &str, xslt: &str) -> error::SpannedResult<sequence::Sequence> {
    let namespaces = Namespaces::new(Namespaces::default_namespaces(), None, Some(FN_NAMESPACE));
    let static_context = StaticContext::from_namespaces(namespaces);
    let root = xot.parse(xml).unwrap();
    let program = parse(&static_context, xslt).unwrap();
    evaluate_program(xot, &program, root, &static_context)
}

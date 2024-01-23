use xee_interpreter::context::DynamicContext;
use xee_interpreter::context::StaticContext;
use xee_interpreter::error;
use xee_interpreter::interpreter::{Program, SequenceOutput};
use xee_name::{Namespaces, FN_NAMESPACE};
use xot::{Node, Xot};

use crate::ast_ir::parse;

fn evaluate_program(
    xot: &Xot,
    program: &Program,
    root: Node,
    static_context: &StaticContext,
) -> error::SpannedResult<SequenceOutput> {
    let uri = xee_interpreter::xml::Uri::new("http://example.com");
    let mut documents = xee_interpreter::xml::Documents::new();
    documents.add_root(xot, &uri, root);
    let context = DynamicContext::from_documents(xot, static_context, &documents);
    let document = documents.get(&uri).unwrap();
    let runnable = program.runnable(&context);
    runnable.apply_templates_xot_node(document.root)
}

fn evaluate(xml: &str, xslt: &str) -> error::SpannedResult<SequenceOutput> {
    let namespaces = Namespaces::new(Namespaces::default_namespaces(), None, Some(FN_NAMESPACE));
    let static_context = StaticContext::from_namespaces(namespaces);
    let mut xot = Xot::new();
    let root = xot.parse(xml).unwrap();
    let program = parse(&static_context, xslt).unwrap();
    evaluate_program(&xot, &program, root, &static_context)
}

#[cfg(test)]
mod tests {
    use xee_interpreter::occurrence::Occurrence;

    use super::*;

    #[test]
    fn test_transform() {
        let o = evaluate(
            "<doc/>",
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><a/></xsl:template></xsl:transform>"#,
        ).unwrap();
        let sequence = o.sequence;
        let output = o.output;
        assert_eq!(
            output
                .to_string(
                    sequence
                        .items()
                        .one()
                        .unwrap()
                        .to_node()
                        .unwrap()
                        .xot_node()
                )
                .unwrap(),
            "<a/>"
        );
    }
}

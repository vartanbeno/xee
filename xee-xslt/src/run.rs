use xee_interpreter::context::DynamicContext;
use xee_interpreter::context::StaticContext;
use xee_interpreter::error;
use xee_interpreter::interpreter::{Program, SequenceOutput};
use xee_interpreter::sequence;
use xee_interpreter::xml;
use xee_name::{Namespaces, FN_NAMESPACE};
use xot::{Node, Xot};

use crate::ast_ir::parse;

pub fn evaluate_program(
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
    let item: sequence::Item = xml::Node::Xot(document.root).into();
    runnable.many_output(Some(&item))
}

pub fn evaluate(xml: &str, xslt: &str) -> error::SpannedResult<SequenceOutput> {
    let namespaces = Namespaces::new(Namespaces::default_namespaces(), None, Some(FN_NAMESPACE));
    let static_context = StaticContext::from_namespaces(namespaces);
    let mut xot = Xot::new();
    let root = xot.parse(xml).unwrap();
    let program = parse(&static_context, xslt).unwrap();
    // dbg!(&program.functions[0].decoded());
    evaluate_program(&xot, &program, root, &static_context)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_transform() {
        let output = evaluate(
            "<doc/>",
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><a/></xsl:template></xsl:transform>"#,
        ).unwrap();
        assert_eq!(output.to_string(), "<a/>");
    }

    #[test]
    fn test_transform_nested() {
        let output = evaluate(
            "<doc/>",
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><a><b/><b/></a></xsl:template></xsl:transform>"#,
        ).unwrap();
        assert_eq!(output.to_string(), "<a><b/><b/></a>");
    }

    // #[test]
    // fn test_transform_nested_apply_templates() {
    //     let output = evaluate(
    //         "<doc><foo/><bar/></doc>",
    //         r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
    //              <xsl:template match="/">
    //                <o><xsl:apply-templates select="doc/*" /></o>
    //              </xsl:template>
    //              <xsl:template match="foo">
    //                <f/>
    //              </xsl:template>
    //              <xsl:template match="bar">
    //                 <b/>
    //              </xsl:template>
    //           </xsl:transform>"#,
    //     )
    //     .unwrap();
    //     assert_eq!(output.to_string(), "<o><f/><b/></o>");
    // }
}

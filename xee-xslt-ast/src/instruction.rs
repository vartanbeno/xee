use xot::{NameId, Xot};

use crate::ast_core as ast;
use crate::error::Error;
use crate::names::SequenceConstructorName;
use crate::parse::Element;

pub(crate) trait InstructionParser: Sized + Into<ast::SequenceConstructorItem> {
    fn parse(element: &Element) -> Result<ast::SequenceConstructorItem, Error> {
        let ast = Self::parse_ast(element)?;
        ast.validate(element)?;
        Ok(ast.into())
    }

    fn validate(&self, _element: &Element) -> Result<(), Error> {
        Ok(())
    }

    fn parse_ast(element: &Element) -> Result<Self, Error>;
}

impl InstructionParser for ast::SequenceConstructorItem {
    fn parse_ast(element: &Element) -> Result<ast::SequenceConstructorItem, Error> {
        let sname = element
            .names
            .sequence_constructor_name(element.element.name());

        if let Some(sname) = sname {
            // parse a known sequence constructor instruction
            match sname {
                SequenceConstructorName::Copy => ast::Copy::parse(element),
                SequenceConstructorName::If => ast::If::parse(element),
                SequenceConstructorName::Variable => ast::Variable::parse(element),
            }
        } else {
            let ns = element.xot.namespace_for_name(element.element.name());
            if ns == element.names.xsl_ns {
                // we have an unknown xsl instruction, fail with error
                Err(Error::InvalidInstruction { span: element.span })
            } else {
                // we parse the literal element
                ast::ElementNode::parse(element)
            }
        }
    }
}

impl InstructionParser for ast::ElementNode {
    fn parse_ast(element: &Element) -> Result<ast::ElementNode, Error> {
        Ok(ast::ElementNode {
            name: to_name(element.xot, element.element.name()),

            standard: element.xsl_standard()?,
            span: element.span,
        })
    }
}

fn to_name(xot: &Xot, name: NameId) -> ast::Name {
    let (local, namespace) = xot.name_ns_str(name);
    ast::Name {
        namespace: namespace.to_string(),
        local: local.to_string(),
    }
}

// impl InstructionParser for ast::Assert {
//     fn parse_ast(element: &Element) -> Result<Self, Error> {
//         let names = element.names;
//         Ok(ast::Assert {
//             test: element.required(names.test, |s, span| element.xpath(s, span))?,
//             select: element.optional(names.select, |s, span| element.xpath(s, span))?,
//             error_code: element
//                 .optional(names.error_code, |s, span| element.attribute_value(s, span))?,
//             content: element.sequence_constructor()?,

//             standard: element.standard()?,
//             span: element.span,
//         })
//     }
// }

impl InstructionParser for ast::Copy {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let names = element.names;
        Ok(ast::Copy {
            select: element.optional(names.select, |s, span| element.xpath(s, span))?,
            copy_namespaces: element.boolean(names.copy_namespaces, true)?,
            inherit_namespaces: element.boolean(names.inherit_namespaces, true)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, Element::eqnames)?,
            type_: element.optional(names.as_, Element::eqname)?,
            validation: element
                .optional(names.validation, Element::validation)?
                // TODO: should depend on global validation attribute
                .unwrap_or(ast::Validation::Strip),
            content: element.sequence_constructor()?,
            standard: element.standard()?,
            span: element.span,
        })
    }
}

// impl InstructionParser for ast::Fallback {
//     fn parse_ast(element: &Element) -> Result<Self, Error> {
//         let parser = element.xslt_parser;
//         let content =
//         Ok(ast::Fallback {
//             content: parser.parse_sequence_constructor(element.node)?;
//             span: element.span,
//         })
//     }
// }

impl InstructionParser for ast::If {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let names = element.names;
        Ok(ast::If {
            test: element.required(names.test, |s, span| element.xpath(s, span))?,
            content: element.sequence_constructor()?,
            standard: element.standard()?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse_ast(element: &Element) -> Result<Self, Error> {
        let names = element.names;

        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });

        Ok(ast::Variable {
            name: element.required(names.name, Element::eqname)?,
            select: element.optional(names.select, |s, span| element.xpath(s, span))?,
            as_: element.optional(names.as_, |s, span| element.sequence_type(s, span))?,
            static_: element.boolean(names.static_, false)?,
            visibility: element.optional(names.visibility, Element::visibility_with_abstract)?,
            content: element.sequence_constructor()?,
            standard: element.standard()?,
            span: element.span,
        })
    }

    fn validate(&self, element: &Element) -> Result<(), Error> {
        if self.visibility == Some(ast::VisibilityWithAbstract::Abstract) && self.select.is_some() {
            return Err(element.attribute_unexpected(
                element.names.select,
                "select attribute is not allowed when visibility is abstract",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{names::Names, parse::XsltParser};

    use super::*;
    use insta::assert_ron_snapshot;
    use xee_xpath_ast::Namespaces;

    fn parse(s: &str) -> Result<ast::SequenceConstructorItem, Error> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let namespaces = Namespaces::default();

        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let node = xot.document_element(node).unwrap();
        let parser = XsltParser::new(&xot, &names, &span_info, namespaces);
        parser.parse(node)
    }

    #[test]
    fn test_if() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_variable() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_missing_required() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_broken_xpath() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="let $x := 1">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_sequence_type() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_boolean_default_no_with_explicit_yes() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" static="yes" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="public">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility_abstract_with_select_is_error() {
        assert_ron_snapshot!(parse(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="abstract" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_copy() {
        assert_ron_snapshot!(parse(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" copy-namespaces="no" inherit-namespaces="no" validation="strict">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames() {
        assert_ron_snapshot!(parse(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo bar baz">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_nested_if() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()"><xsl:if test="true()">Hello</xsl:if></xsl:if>"#
        ));
    }

    #[test]
    fn test_if_with_standard_attribute() {
        assert_ron_snapshot!(parse(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_literal_result_element() {
        assert_ron_snapshot!(parse(r#"<foo/>"#));
    }

    #[test]
    fn test_literal_result_element_with_standard_attribute() {
        assert_ron_snapshot!(parse(
            r#"<foo xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:expand-text="yes"/>"#
        ));
    }
}

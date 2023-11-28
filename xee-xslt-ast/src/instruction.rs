use xot::{NameId, Xot};

use crate::ast_core as ast;
use crate::combinator::{many, optional, ElementError as Error, NodeParser};
use crate::element::{by_element, content_parse, instruction, Element};

type Result<V> = std::result::Result<V, Error>;

pub(crate) trait InstructionParser: Sized {
    fn validate(&self, _element: &Element) -> Result<()> {
        Ok(())
    }

    fn should_be_empty() -> bool {
        false
    }

    fn parse(element: &Element) -> Result<Self>;

    fn parse_and_validate(element: &Element) -> Result<Self> {
        let item = Self::parse(element)?;
        if Self::should_be_empty() {
            if let Some(child) = element.state.xot.first_child(element.node) {
                return Err(Error::Unexpected {
                    span: element.state.span(child).ok_or(Error::Internal)?,
                });
            }
        }
        item.validate(element)?;
        Ok(item)
    }
}

pub(crate) trait SequenceConstructorParser:
    InstructionParser + Into<ast::SequenceConstructorItem>
{
    fn parse_sequence_constructor_item(element: &Element) -> Result<ast::SequenceConstructorItem> {
        let item = Self::parse_and_validate(element)?;
        Ok(item.into())
    }
}

impl<T> SequenceConstructorParser for T where
    T: InstructionParser + Into<ast::SequenceConstructorItem>
{
}

pub(crate) trait DeclarationParser: InstructionParser + Into<ast::Declaration> {
    fn parse_declaration(element: &Element) -> Result<ast::Declaration> {
        let item = Self::parse_and_validate(element)?;
        Ok(item.into())
    }
}

impl<T> DeclarationParser for T where T: InstructionParser + Into<ast::Declaration> {}

impl InstructionParser for ast::SequenceConstructorItem {
    fn parse(element: &Element) -> Result<ast::SequenceConstructorItem> {
        let context = element.state;
        let name = context
            .names
            .sequence_constructor_name(element.element.name());

        if let Some(name) = name {
            name.parse(element)
        } else {
            let ns = context.xot.namespace_for_name(element.element.name());
            if ns == context.names.xsl_ns {
                // we have an unknown xsl instruction, fail with error
                Err(Error::Unexpected { span: element.span })
            } else {
                // we parse the literal element
                ast::ElementNode::parse_sequence_constructor_item(element)
            }
        }
    }
}

impl InstructionParser for ast::Declaration {
    fn parse(element: &Element) -> Result<ast::Declaration> {
        let name = element.state.names.declaration_name(element.element.name());

        if let Some(name) = name {
            name.parse(element)
        } else {
            Err(Error::Unexpected { span: element.span })
        }
    }
}

impl InstructionParser for ast::ElementNode {
    fn parse(element: &Element) -> Result<ast::ElementNode> {
        Ok(ast::ElementNode {
            name: to_name(&element.state.xot, element.element.name()),

            standard: element.xsl_standard()?,
            span: element.span,
        })
    }
}

// impl InstructionParser for ast::ApplyTemplatesContent {
//     fn parse(element: &Element) -> Result<Self> {

//     }
// }

fn to_name(xot: &Xot, name: NameId) -> ast::Name {
    let (local, namespace) = xot.name_ns_str(name);
    ast::Name {
        namespace: namespace.to_string(),
        local: local.to_string(),
    }
}

impl InstructionParser for ast::Accept {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Accept {
            component: element.required(names.component, element.component())?,
            names: element.required(names.names, element.tokens())?,
            visibility: element.required(names.visibility, element.visibility_with_hidden())?,

            standard: element.standard()?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::Accumulator {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse_rules = content_parse(many(instruction(names.xsl_accumulator_rule)));

        Ok(ast::Accumulator {
            name: element.required(names.name, element.eqname())?,
            initial_value: element.required(names.initial_value, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            streamable: element.boolean_with_default(names.streamable, false)?,

            standard: element.standard()?,
            span: element.span,

            rules: parse_rules(element)?,
        })
    }
}

impl InstructionParser for ast::AccumulatorRule {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::AccumulatorRule {
            match_: element.required(names.match_, element.pattern())?,
            phase: element.optional(names.phase, element.phase())?,
            select: element.optional(names.select, element.xpath())?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AnalyzeString {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let select = element.required(names.select, element.xpath())?;
        let regex = element.required(names.regex, element.value_template(element.string()))?;
        let flags = element.optional(names.flags, element.value_template(element.string()))?;

        let standard = element.standard()?;

        let parse = content_parse(
            optional(instruction(names.xsl_matching_substring))
                .then(optional(instruction(names.xsl_non_matching_substring)))
                .then(many(instruction(names.xsl_fallback))),
        );

        let ((matching_substring, non_matching_substring), fallbacks) = parse(element)?;

        Ok(ast::AnalyzeString {
            select,
            regex,
            flags,
            standard,

            span: element.span,

            matching_substring,
            non_matching_substring,
            fallbacks,
        })
    }
}

impl InstructionParser for ast::ApplyImports {
    fn parse(element: &Element) -> Result<Self> {
        let parse = content_parse(many(instruction(element.state.names.xsl_with_param)));
        Ok(ast::ApplyImports {
            standard: element.standard()?,
            span: element.span,

            with_params: parse(element)?,
        })
    }
}

impl InstructionParser for ast::ApplyTemplates {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        let parse = content_parse(many(by_element(|element| {
            let name = element.element.name();
            if name == names.xsl_with_param {
                Ok(ast::ApplyTemplatesContent::WithParam(
                    ast::WithParam::parse_and_validate(&element)?,
                ))
            } else if name == names.xsl_sort {
                Ok(ast::ApplyTemplatesContent::Sort(
                    ast::Sort::parse_and_validate(&element)?,
                ))
            } else {
                Err(Error::Unexpected { span: element.span })
            }
        })));

        Ok(ast::ApplyTemplates {
            select: element.optional(names.select, element.xpath())?,
            mode: element.optional(names.mode, element.token())?,

            standard: element.standard()?,
            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Assert {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Assert {
            test: element.required(names.test, element.xpath())?,
            select: element.optional(names.select, element.xpath())?,
            error_code: element
                .optional(names.error_code, element.value_template(element.eqname()))?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Attribute {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Attribute {
            name: element.required(names.name, element.value_template(element.qname()))?,
            namespace: element.optional(names.namespace, element.value_template(element.uri()))?,
            select: element.optional(names.select, element.xpath())?,
            separator: element
                .optional(names.separator, element.value_template(element.string()))?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element.optional(names.validation, element.validation())?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::AttributeSet {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        let parse = content_parse(many(instruction(names.xsl_attribute)));

        Ok(ast::AttributeSet {
            name: element.required(names.name, element.eqname())?,
            use_attribute_sets: element.optional(names.use_attribute_sets, element.eqnames())?,
            visibility: element.optional(names.visibility, element.visibility_with_abstract())?,
            streamable: element.boolean_with_default(names.streamable, false)?,

            standard: element.standard()?,
            span: element.span,

            content: parse(element)?,
        })
    }
}

impl InstructionParser for ast::Copy {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Copy {
            select: element.optional(names.select, element.xpath())?,
            copy_namespaces: element.boolean_with_default(names.copy_namespaces, true)?,
            inherit_namespaces: element.boolean_with_default(names.inherit_namespaces, true)?,
            use_attribute_sets: element.optional(names.use_attribute_sets, element.eqnames())?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element
                .optional(names.validation, element.validation())?
                // TODO: should depend on global validation attribute
                .unwrap_or(ast::Validation::Strip),

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::CopyOf {
    fn should_be_empty() -> bool {
        true
    }

    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        Ok(ast::CopyOf {
            select: element.required(names.select, element.xpath())?,
            copy_accumulators: element.boolean_with_default(names.copy_accumulators, false)?,
            copy_namespaces: element.boolean_with_default(names.copy_namespaces, true)?,
            type_: element.optional(names.type_, element.eqname())?,
            validation: element.optional(names.validation, element.validation())?,

            standard: element.standard()?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::Fallback {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::Fallback {
            content: element.sequence_constructor()?,

            standard: element.standard()?,
            span: element.span,
        })
    }
}

impl InstructionParser for ast::If {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::If {
            test: element.required(names.test, element.xpath())?,
            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::MatchingSubstring {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::MatchingSubstring {
            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::NonMatchingSubstring {
    fn parse(element: &Element) -> Result<Self> {
        Ok(ast::NonMatchingSubstring {
            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Sort {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Sort {
            select: element.optional(names.select, element.xpath())?,
            lang: element.optional(names.lang, element.value_template(element.language()))?,
            order: element.optional(names.order, element.value_template(element.order()))?,
            collation: element.optional(names.collation, element.value_template(element.uri()))?,
            stable: element.optional(names.stable, element.value_template(element.boolean()))?,
            case_order: element.optional(
                names.case_order,
                element.value_template(element.case_order()),
            )?,
            data_type: element
                .optional(names.data_type, element.value_template(element.data_type()))?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

impl InstructionParser for ast::Transform {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::Transform {
            id: element.optional(names.id, element.id())?,
            input_type_annotations: element.optional(
                names.input_type_annotations,
                element.input_type_annotations(),
            )?,
            extension_element_prefixes: element
                .optional(names.extension_element_prefixes, element.prefixes())?,

            standard: element.standard()?,
            span: element.span,

            declarations: element.declarations()?,
        })
    }
}

impl InstructionParser for ast::Variable {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;

        // This is a rule somewhere, but not sure whether it's correct;
        // can visibility be absent or is there a default visibility?
        // let visibility = visibility.unwrap_or(if static_ {
        //     ast::VisibilityWithAbstract::Private
        // } else {
        //     ast::VisibilityWithAbstract::Public
        // });

        Ok(ast::Variable {
            name: element.required(names.name, element.eqname())?,
            select: element.optional(names.select, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            static_: element.boolean_with_default(names.static_, false)?,
            visibility: element.optional(names.visibility, element.visibility_with_abstract())?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }

    fn validate(&self, element: &Element) -> Result<()> {
        if self.visibility == Some(ast::VisibilityWithAbstract::Abstract) && self.select.is_some() {
            return Err(element
                .attribute_unexpected(
                    element.state.names.select,
                    "select attribute is not allowed when visibility is abstract",
                )
                .into());
        }
        Ok(())
    }
}

impl InstructionParser for ast::WithParam {
    fn parse(element: &Element) -> Result<Self> {
        let names = &element.state.names;
        Ok(ast::WithParam {
            name: element.required(names.name, element.eqname())?,
            select: element.optional(names.select, element.xpath())?,
            as_: element.optional(names.as_, element.sequence_type())?,
            tunnel: element.boolean_with_default(names.tunnel, false)?,

            standard: element.standard()?,
            span: element.span,

            content: element.sequence_constructor()?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::context::Context;
    use crate::element::Element;
    use crate::{element::XsltParser, names::Names, state::State};

    use super::*;
    use insta::assert_ron_snapshot;

    fn parse_sequence_constructor_item(s: &str) -> Result<ast::SequenceConstructorItem> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let state = State::new(xot, span_info, names);
        let node = state.xot.document_element(node).unwrap();

        if let Some(element) = state.xot.element(node) {
            let context = Context::new(element);
            let element = Element::new(node, element, context, &state)?;
            ast::SequenceConstructorItem::parse_sequence_constructor_item(&element)
        } else {
            Err(Error::Internal)
        }
    }

    fn parse_transform(s: &str) -> Result<ast::Transform> {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);

        let (node, span_info) = xot.parse_with_span_info(s).unwrap();
        let node = xot.document_element(node).unwrap();
        let context = State::new(xot, span_info, names);
        let parser = XsltParser::new(&context);
        parser.parse_transform(node)
    }

    #[test]
    fn test_if() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_variable() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_missing_required() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_broken_xpath() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" select="let $x := 1">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_sequence_type() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="http://www.w3.org/2001/XMLSchema" name="foo" as="xs:string" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_boolean_default_no_with_explicit_yes() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" static="yes" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="public">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_variable_visibility_abstract_with_select_is_error() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:variable xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="foo" visibility="abstract" select="true()">Hello</xsl:variable>"#
        ));
    }

    #[test]
    fn test_copy() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" copy-namespaces="no" inherit-namespaces="no" validation="strict">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo bar baz">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_eqnames_error() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy xmlns:xsl="http://www.w3.org/1999/XSL/Transform" use-attribute-sets="foo br!ken bar">Hello</xsl:copy>"#
        ));
    }

    #[test]
    fn test_nested_if() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()"><xsl:if test="true()">Hello</xsl:if></xsl:if>"#
        ));
    }

    #[test]
    fn test_if_with_standard_attribute() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" expand-text="yes">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_literal_result_element() {
        assert_ron_snapshot!(parse_sequence_constructor_item(r#"<foo/>"#));
    }

    #[test]
    fn test_literal_result_element_with_standard_attribute() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<foo xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:expand-text="yes"/>"#
        ));
    }

    #[test]
    fn test_no_fn_namespace_by_default() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:if xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="fn:true()">Hello</xsl:if>"#
        ));
    }

    #[test]
    fn test_attribute_value_template_just_string() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:assert xmlns:xsl="http://www.w3.org/1999/XSL/Transform" test="true()" error-code="foo">Hello</xsl:assert>"#
        ));
    }

    #[test]
    fn test_analyze_string() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_matching_substring() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_non_matching_substring() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:fallback>Fallback 1</xsl:fallback><xsl:fallback>Fallback 2</xsl:fallback></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_fallbacks() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:matching-substring>Matching</xsl:matching-substring><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_absent_all() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_analyze_string_matching_non_matching_wrong_order() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:analyze-string xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()" regex="foo"><xsl:non-matching-substring>Nonmatching</xsl:non-matching-substring><xsl:matching-substring>Matching</xsl:matching-substring></xsl:analyze-string>"#
        ));
    }

    #[test]
    fn test_accumulator() {
        assert_ron_snapshot!(parse_transform(
            r#"<xsl:transform version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:accumulator name="foo" initial-value="1"><xsl:accumulator-rule match="foo"/></xsl:accumulator></xsl:transform>"#
        ));
    }

    #[test]
    fn test_should_be_empty_not_empty() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:copy-of xmlns:xsl="http://www.w3.org/1999/XSL/Transform" select="true()">Illegal content</xsl:copy-of>"#
        ))
    }

    #[test]
    fn test_apply_templates_with_mixed_content() {
        assert_ron_snapshot!(parse_sequence_constructor_item(
            r#"<xsl:apply-templates xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:sort>Sort</xsl:sort><xsl:with-param name="a">With param</xsl:with-param></xsl:apply-templates>"#
        ))
    }
}

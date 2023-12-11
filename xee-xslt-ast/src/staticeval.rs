// Static evaluation of an XSLT stylesheet
// This handles:
// - Whitespace cleanup
// - Static parameters and variables
// - use-when
// - shadow attributes

// The end result is the static global variables, and modified XML tree
// that has any element with use-when that evaluates to false removed,
// as well as any shadow attributes resolved to normal attributes.
// Any attribute on an XSLT element prefixed by _ is taken as a shadow
// attribute - if the attribute later on turns on not to exist, then
// we get a parse error then.

// The procedure is quite tricky: in order to parse xpath expressions
// statically we need to pass in the names of any known global variables that
// we've encountered before.

use xot::Node;

use xee_xpath::{DynamicContext, Program, Sequence, Variables};
use xee_xpath_ast::ast as xpath_ast;

use crate::attributes::Attributes;
use crate::content::Content;
use crate::context::Context;
use crate::error::ElementError;
use crate::state::State;
use crate::whitespace::strip_whitespace;

struct StaticEvaluator {
    static_global_variables: Variables,
    static_parameters: Variables,
    to_remove: Vec<Node>,
}

struct StaticNode {
    node: Node,
    instruction: StaticInstruction,
}

enum StaticInstruction {
    Variable,
    Param,
    Other,
}

impl StaticEvaluator {
    fn new(static_parameters: Variables) -> Self {
        Self {
            static_global_variables: Variables::new(),
            static_parameters,
            to_remove: Vec::new(),
        }
    }

    fn evaluate_top_level(&mut self, top_attributes: Attributes) -> Result<(), ElementError> {
        let xot = &top_attributes.content.state.xot;
        let names = &top_attributes.content.state.names;
        let mut node = xot.first_child(top_attributes.content.node);
        let mut context = top_attributes.content.context.clone();
        while let Some(current) = node {
            let element = xot.element(current);
            if let Some(element) = element {
                let current_content = Content::new(current, top_attributes.content.state, context);
                let attributes = current_content.attributes(element);
                if !self.evaluate_use_when(&top_attributes)?
                    || !self.evaluate_use_when(&attributes)?
                {
                    self.to_remove.push(current);
                    context = attributes.content.context;
                } else if element.name() == names.xsl_variable {
                    context = self.evaluate_variable(attributes)?;
                } else if element.name() == names.xsl_param {
                    context = self.evaluate_param(attributes)?;
                } else {
                    context = self.evaluate_other(attributes)?;
                }
            }
            node = xot.next_sibling(current);
        }
        Ok(())
    }

    fn update_tree(&self, state: &mut State) -> Result<(), ElementError> {
        for node in &self.to_remove {
            state
                .xot
                .remove(*node)
                .map_err(|_| ElementError::Internal)?;
        }
        Ok(())
    }

    fn evaluate_variable(&mut self, attributes: Attributes) -> Result<Context, ElementError> {
        let names = &attributes.content.state.names;
        if attributes.boolean_with_default(names.static_, false)? {
            let name = attributes.required(names.name, attributes.eqname())?;
            let select = attributes.required(names.select, attributes.xpath())?;
            let value = self.evaluate_static_xpath(select.xpath, &attributes.content)?;
            let context = attributes.content.context.with_variable_name(&name);
            self.static_global_variables.insert(name, value);
            Ok(context)
        } else {
            Ok(attributes.content.context)
        }
    }

    fn evaluate_param(&mut self, attributes: Attributes) -> Result<Context, ElementError> {
        let names = &attributes.content.state.names;
        if attributes.boolean_with_default(names.static_, false)? {
            let name = attributes.required(names.name, attributes.eqname())?;
            let required = attributes.boolean_with_default(names.required, false)?;
            let context = attributes.content.context.with_variable_name(&name);
            let value = self.static_parameters.get(&name);
            let insert_value = if let Some(value) = value {
                value.clone()
            } else if required {
                // a required value is mandatory, should return proper error
                todo!()
            } else {
                let select = attributes.optional(names.select, attributes.xpath())?;
                if let Some(select) = select {
                    self.evaluate_static_xpath(select.xpath, &attributes.content)?
                } else {
                    // we interpret 'as' as a string here, as we really only want to
                    // check for its existence
                    let as_ = attributes.optional(names.as_, attributes.string())?;
                    if as_.is_some() {
                        Sequence::empty()
                    } else {
                        Sequence::from("")
                    }
                }
            };
            self.static_global_variables.insert(name, insert_value);
            Ok(context)
        } else {
            Ok(attributes.content.context)
        }
    }

    fn evaluate_other(&self, attributes: Attributes) -> Result<Context, ElementError> {
        Ok(attributes.content.context)
    }

    fn evaluate_use_when(&mut self, attributes: &Attributes) -> Result<bool, ElementError> {
        let names = &attributes.content.state.names;
        let use_when = if attributes.in_xsl_namespace() {
            attributes.optional(names.standard.use_when, attributes.xpath())?
        } else {
            attributes.optional(names.xsl_standard.use_when, attributes.xpath())?
        };

        if let Some(use_when) = use_when {
            let value = self.evaluate_static_xpath(use_when.xpath, &attributes.content)?;
            if !value
                .effective_boolean_value()
                // TODO: the way the span is added is ugly, but it ought
                // to at least describe the span of the use-when attribute
                .map_err(|e| e.with_span((use_when.span.start..use_when.span.end).into()))?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_static_xpath(
        &self,
        xpath: xpath_ast::XPath,
        content: &Content,
    ) -> Result<Sequence, xee_xpath::SpannedError> {
        let parser_context = content.parser_context();
        let static_context = parser_context.into();
        let program = Program::new(&static_context, xpath)?;
        let dynamic_context = DynamicContext::from_variables(
            &content.state.xot,
            &static_context,
            &self.static_global_variables,
        );
        let runnable = program.runnable(&dynamic_context);
        runnable.many(None)
    }
}

fn static_evaluate(
    state: &mut State,
    node: Node,
    static_parameters: Variables,
) -> Result<Variables, ElementError> {
    strip_whitespace(&mut state.xot, &state.names, node);
    let mut evaluator = StaticEvaluator::new(static_parameters);

    if let Some(element) = state.xot.element(node) {
        // construct a context with the prefixes of the top-level element
        let context = Context::new(element.prefixes().clone());
        // use this to extract the xpath default namespace standard
        // attribute, which is required to execute static xpath
        let content = Content::new(node, state, context.clone());
        let attributes = content.attributes(element);
        let xpath_default_namespace = attributes.optional(
            state.names.standard.xpath_default_namespace,
            attributes.uri(),
        )?;
        // construct a new context based on that, and a new content
        let context = context.with_xpath_default_namespace(xpath_default_namespace);
        let content = Content::new(node, state, context);
        let attributes = content.attributes(element);
        // now go through the top level elements for static evaluation
        evaluator.evaluate_top_level(attributes)?;
        // and update the tree accordingly
        evaluator.update_tree(state)?;
    }
    Ok(evaluator.static_global_variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::Names;

    #[test]
    fn test_one_static_variable() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:variable name="x" static="yes" select="'foo'"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(variables.len(), 1);
        let name = xpath_ast::Name::new("x".to_string(), None, None);
        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("foo").into())
        );
    }

    #[test]
    fn test_static_variable_depends_on_another() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:variable name="x" static="yes" select="'foo'"/>
            <xsl:variable name="y" static="yes" select="concat($x, '!')"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(variables.len(), 2);
        let name = xpath_ast::Name::new("y".to_string(), None, None);
        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("foo!").into())
        );
    }

    #[test]
    fn test_one_parameter_present() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:param name="x" static="yes" select="'foo'"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let name = xpath_ast::Name::new("x".to_string(), None, None);
        let static_parameters =
            Variables::from([(name.clone(), xee_xpath::Item::from("bar").into())]);

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, static_parameters).unwrap();
        assert_eq!(variables.len(), 1);

        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("bar").into())
        );
    }

    #[test]
    fn test_one_parameter_absent_not_required_with_select() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:param name="x" static="yes" select="'foo'"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let name = xpath_ast::Name::new("x".to_string(), None, None);
        let static_parameters = Variables::new();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, static_parameters).unwrap();
        assert_eq!(variables.len(), 1);

        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("foo").into())
        );
    }

    #[test]
    fn test_one_parameter_absent_no_select_without_as() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:param name="x" static="yes" />
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let name = xpath_ast::Name::new("x".to_string(), None, None);
        let static_parameters = Variables::new();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, static_parameters).unwrap();
        assert_eq!(variables.len(), 1);

        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("").into())
        );
    }

    #[test]
    fn test_one_parameter_absent_no_select_with_as() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:param name="x" static="yes" as="xs:integer" />
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let name = xpath_ast::Name::new("x".to_string(), None, None);
        let static_parameters = Variables::new();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, static_parameters).unwrap();
        assert_eq!(variables.len(), 1);

        assert_eq!(variables.get(&name), Some(&Sequence::empty()));
    }

    #[test]
    fn test_use_when_false_on_top_level() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:if use-when="false()"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(
            state.xot.to_string(document_element).unwrap(),
            "<xsl:stylesheet xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\" version=\"3.0\"/>"
        );
    }

    #[test]
    fn test_use_when_true_on_top_level() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:if use-when="true()"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(
            state.xot.to_string(document_element).unwrap(),
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:if use-when="true()"/></xsl:stylesheet>"#
        );
    }

    #[test]
    fn test_use_when_depends_on_variable() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:variable name="x" static="yes" select="false()"/>
            <foo xsl:use-when="$x"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(
            state.xot.to_string(document_element).unwrap(),
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:variable name="x" static="yes" select="false()"/></xsl:stylesheet>"#
        );
    }

    #[test]
    fn test_xsl_use_when_false_on_top_level() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <foo xsl:use-when="false()"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(
            state.xot.to_string(document_element).unwrap(),
            "<xsl:stylesheet xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\" version=\"3.0\"/>"
        );
    }

    #[test]
    fn test_use_when_false_for_xsl_param() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:param name="x" static="yes" select="'foo'" use-when="false()"/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        let variables = static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(variables.len(), 0);
    }

    #[test]
    fn test_xpath_default_namespace() {
        let xslt = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
            xmlns="http://www.w3.org/1999/xhtml"
            xpath-default-namespace="http://www.w3.org/1999/xhtml"
            version="3.0">
            <xsl:param name="x" static="yes"/>
            <xsl:variable name="y" static="yes" select="$x/html/body/p/string()"/>
        </xsl:stylesheet>"#;

        let xhtml = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
          <body>
            <p>foo</p>
          </body>
        </html>"#;

        let mut xot = xot::Xot::new();
        let xhtml = xot.parse(xhtml).unwrap();
        let (xslt, span_info) = xot.parse_with_span_info(xslt).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(xslt).unwrap();

        let mut state = State::new(xot, span_info, names);
        let parameters = Variables::from([(
            xpath_ast::Name::new("x".to_string(), None, None),
            xee_xpath::Item::Node(xee_xpath::Node::Xot(xhtml)).into(),
        )]);
        let variables = static_evaluate(&mut state, document_element, parameters).unwrap();
        assert_eq!(variables.len(), 2);
        let y = xpath_ast::Name::new("y".to_string(), None, None);
        assert_eq!(
            variables.get(&y),
            Some(&xee_xpath::Item::from("foo").into())
        );
    }

    #[test]
    fn test_use_when_false_on_top_node() {
        let xml = r#"
        <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0" use-when="false()">
           <foo/>
        </xsl:stylesheet>
        "#;
        let mut xot = xot::Xot::new();
        let (root, span_info) = xot.parse_with_span_info(xml).unwrap();
        let names = Names::new(&mut xot);
        let document_element = xot.document_element(root).unwrap();

        let mut state = State::new(xot, span_info, names);
        static_evaluate(&mut state, document_element, Variables::new()).unwrap();
        assert_eq!(
            state.xot.to_string(document_element).unwrap(),
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0" use-when="false()"/>"#
        );
    }
    // TODO:

    // - custom element namespace
    // - shadow attributes support
    // - shadow attributes for use-when in particular
    // - handling non-toplevel elements
}

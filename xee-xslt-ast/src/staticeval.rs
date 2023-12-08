use xot::Node;

use xee_xpath::{DynamicContext, Program, Sequence, Variables};
use xee_xpath_ast::ast as xpath_ast;
use xot::Xot;

use crate::ast_core as ast;
use crate::attributes::Attributes;
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

use crate::combinator::Content;
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::InstructionParser;
use crate::names::Names;
use crate::state::State;
use crate::whitespace::strip_whitespace;

struct StaticEvaluator {
    static_global_variables: Variables,
    static_parameters: Variables,
    to_remove: Vec<Node>,
    structure_stack: Vec<bool>,
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
            structure_stack: Vec::new(),
        }
    }

    fn evaluate_top_level(&mut self, content: Content) -> Result<(), ElementError> {
        let xot = &content.state.xot;
        let names = &content.state.names;
        let mut node = xot.first_child(content.node);
        // TODO: we should initialize context with the right prefixes
        let mut current_context = Context::empty();
        while let Some(current) = node {
            let element = xot.element(current);
            if let Some(element) = element {
                if element.name() == names.xsl_variable {
                    let current_content = Content::new(current, content.state, current_context);
                    let attributes = Attributes::new(current_content, element);
                    if attributes.boolean_with_default(names.static_, false)? {
                        let name = attributes.required(names.name, attributes.eqname())?;
                        let select = attributes.required(names.select, attributes.xpath())?;
                        let value =
                            self.evaluate_static_xpath(select.xpath, &attributes.content)?;
                        current_context = attributes.content.context.with_variable_name(&name);
                        self.static_global_variables.insert(name, value);
                    } else {
                        current_context = attributes.content.context.clone();
                    }
                } else if element.name() == names.xsl_param {
                    let current_content = Content::new(current, content.state, current_context);
                    let attributes = Attributes::new(current_content, element);
                    if attributes.boolean_with_default(names.static_, false)? {
                        let name = attributes.required(names.name, attributes.eqname())?;
                        let required = attributes.boolean_with_default(names.required, false)?;
                        current_context = attributes.content.context.with_variable_name(&name);
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
                    } else {
                        current_context = attributes.content.context.clone();
                    }
                }
            }
            node = xot.next_sibling(current);
        }
        Ok(())
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

    fn static_param_instruction(&self, node: Node, content: Content) -> Option<ast::Param> {
        let element = content.state.xot.element(node)?;
        let attributes = Attributes::new(content, element);
        // TODO: we don't handle standard attributes, so unseen attributes
        // will complain if we use one. We can have another entry point
        // that simply doesn't do this check as it'll happen anyway later
        if let Ok(param) = ast::Param::parse_and_validate(&attributes) {
            if param.static_ {
                return Some(param);
            }
        }
        None
    }

    fn static_param_value(&self, param: ast::Param) -> Result<Sequence, xee_xpath::SpannedError> {
        // if it's available in the static parameters, return it
        if let Some(value) = self.static_parameters.get(&param.name) {
            return Ok(value.clone());
        }
        if param.required {
            // we don't have a value, so error
        }

        // if it's not required, we fall back on select, if it's availab
        // if let Some(select) = param.select {
        //     // TODO: select will have to be created with the right
        //     // variables in the context
        //     return self.evaluate_static_xpath(&select);
        // }

        // without select, we default to the empty sequence if there's
        // an 'as' attribute and otherwise the empty string
        if param.as_.is_some() {
            Ok(Sequence::empty())
        } else {
            Ok(Sequence::from(""))
        }
    }

    fn static_variable_instruction(&self, node: Node, content: Content) -> Option<ast::Variable> {
        let element = content.state.xot.element(node)?;
        let attributes = Attributes::new(content, element);
        if let Ok(variable) = ast::Variable::parse_and_validate(&attributes) {
            if variable.static_ {
                return Some(variable);
            }
        }
        None
    }

    // fn static_variable_value(
    //     &self,
    //     variable: ast::Variable,
    //     content: Content,
    // ) -> Result<Sequence, xee_xpath::SpannedError> {
    //     if let Some(select) = variable.select {
    //         self.evaluate_static_xpath(select.xpath, content)
    //     } else {
    //         // This is an error
    //         todo!()
    //     }
    // }
}

fn static_evaluate(
    mut xot: Xot,
    span_info: xot::SpanInfo,
    names: Names,
    node: Node,
    static_parameters: Variables,
) -> Result<Variables, ElementError> {
    strip_whitespace(&mut xot, &names, node);

    let mut evaluator = StaticEvaluator::new(static_parameters);

    let state = State::new(xot, span_info, names);
    let context = Context::new(xot::Prefixes::new());
    let content = Content::new(node, &state, context);
    evaluator.evaluate_top_level(content)?;
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, Variables::new()).unwrap();
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, Variables::new()).unwrap();
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, static_parameters).unwrap();
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, static_parameters).unwrap();
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, static_parameters).unwrap();
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

        let variables =
            static_evaluate(xot, span_info, names, document_element, static_parameters).unwrap();
        assert_eq!(variables.len(), 1);

        assert_eq!(variables.get(&name), Some(&Sequence::empty()));
    }
}

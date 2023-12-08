use xot::Node;

use xee_xpath::{DynamicContext, Program, Sequence, Variables};
use xee_xpath_ast::ast as xpath_ast;
use xot::Xot;

use crate::ast_core as ast;
use crate::attributes::Attributes;
use crate::combinator::one;
use crate::combinator::Content;
use crate::combinator::NodeParser;
use crate::context::Context;
use crate::error::ElementError;
use crate::instruction::InstructionParser;
use crate::names::Names;
use crate::state::State;
use crate::whitespace::strip_whitespace;

// fn algorithm() {
//     let use_when_result = evaluate_use_when(entry, global_variables);
//     if use_when_result == true {
//         let xsl_param = get_xsl_param(entry);
//         if let Some(xsl_param) = xsl_param {
//             let variable = get_static_param(xsl_param, global_variables);
//             global_variables.insert(xsl_param.name, variable);
//             return;
//         }
//         let xsl_variable = get_xsl_variable(entry);
//         if let Some(xsl_variable) = xsl_variable {
//             let variable = get_static_variable(xsl_variable, global_variables);
//             global_variables.insert(xsl_variable.name, variable);
//         }
//     } else {
//         to_remove.push(entry);
//     }
// }

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
    Variable(ast::Variable),
    Param(ast::Param),
    Other,
}

impl StaticEvaluator {
    fn evaluate_top_level(&mut self, content: Content) -> Result<(), ElementError> {
        // we have a parser that parses variable and param children and
        // other children. It doesn't execute anything, just records them
        // Then we go through the children, update global variables accordingly
        // and evaluate use when using it.
        let variable_instruction = one(|content| {
            let node = content.node;
            let element = content
                .state
                .xot
                .element(node)
                .ok_or(ElementError::Unexpected {
                    span: content.span()?,
                })?;
            let attributes = Attributes::new(content, element);
            // TODO: the problem here is that if use-when is false, we should
            // skip even nodes that contain illegal values, but we can't
            // evaluate use-when yet during the parse.
            // so this implies we should do the parse later, and here
            // just establish what kind of node we have
            let variable = ast::Variable::parse_and_validate(&attributes)?;
            if variable.static_ {
                Ok(StaticNode {
                    node,
                    instruction: StaticInstruction::Variable(variable),
                })
            } else {
                Err(ElementError::Unexpected {
                    span: attributes.content.span()?,
                })
            }
        });
        let other_instruction = one(|content| {
            Ok(StaticNode {
                node: content.node,
                instruction: StaticInstruction::Other,
            })
        });
        let parser = (variable_instruction.or(other_instruction))
            .many()
            .contains();
        let static_content = content.clone();
        let static_nodes = parser.parse_content(content)?;

        // now we can execute static nodes
        for static_node in static_nodes {
            let node = static_node.node;
            let current_content = static_content.with_node(node);
            // if use-when says not to do anything, we skip it

            match static_node.instruction {
                StaticInstruction::Variable(variable) => {
                    // TODO we should actually only try to parse the variable
                    // here, after we evaluate its use-when
                    let name = variable.name.clone();
                    let value = self.static_variable_value(variable, current_content)?;
                    self.static_global_variables.insert(name, value);
                }
                StaticInstruction::Param(param) => {}
                StaticInstruction::Other => {
                    // TODO: here we should evaluate use-when and if
                    // true, shadow-attributes, and then recursively down
                    // into children
                }
            }
        }

        Ok(())
    }

    fn evaluate_static_xpath(
        &self,
        xpath: xpath_ast::XPath,
        content: Content,
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

    fn static_variable_value(
        &self,
        variable: ast::Variable,
        content: Content,
    ) -> Result<Sequence, xee_xpath::SpannedError> {
        if let Some(select) = variable.select {
            self.evaluate_static_xpath(select.xpath, content)
        } else {
            // This is an error
            todo!()
        }
    }
}

fn static_evaluate(
    mut xot: Xot,
    span_info: xot::SpanInfo,
    names: Names,
    node: Node,
) -> Result<Variables, ElementError> {
    strip_whitespace(&mut xot, &names, node);

    let mut evaluator = StaticEvaluator {
        static_global_variables: Variables::new(),
        static_parameters: Variables::new(),
        to_remove: Vec::new(),
        structure_stack: Vec::new(),
    };

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

        let variables = static_evaluate(xot, span_info, names, document_element).unwrap();
        assert_eq!(variables.len(), 1);
        let name = xpath_ast::Name::new("x".to_string(), None, None);
        assert_eq!(
            variables.get(&name),
            Some(&xee_xpath::Item::from("foo").into())
        );
    }
}

use xot::Node;

use xee_xpath::{DynamicContext, Program, Sequence, Variables};
use xee_xpath_ast::ast as xpath_ast;

use crate::ast_core as ast;
use crate::attributes::Attributes;
use crate::combinator::one;
use crate::combinator::Content;
use crate::combinator::NodeParser;
use crate::error::ElementError;
use crate::instruction::InstructionParser;

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
            match static_node.instruction {
                StaticInstruction::Variable(variable) => {
                    let name = variable.name.clone();
                    let value = self.static_variable_value(variable, current_content)?;
                    self.static_global_variables.insert(name, value);
                }
                StaticInstruction::Param(param) => {}
                StaticInstruction::Other => {}
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

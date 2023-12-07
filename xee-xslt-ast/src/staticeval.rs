use ahash::HashMap;
use xot::Node;

use xee_xpath::{evaluate_without_focus_with_variables, Sequence, Variables};
use xee_xpath_ast::ast as xpath_ast;

use crate::ast_core as ast;
use crate::attributes::Attributes;
use crate::combinator::Content;
use crate::context::Context;
use crate::instruction::InstructionParser;
use crate::state::State;

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
    state: State,
    static_global_variables: HashMap<xpath_ast::Name, Sequence>,
    static_parameters: HashMap<xpath_ast::Name, Sequence>,
    to_remove: Vec<Node>,
    structure_stack: Vec<bool>,
}

impl StaticEvaluator {
    // TODO: either evaluate_without_focus_with_variables should
    // accept sequence values (instead of Vec items) or we should devise
    // a try_into in xee_xpath to convert them
    fn static_global_variables(&self) -> Variables {
        let mut variables = Variables::with_capacity(self.static_global_variables.len());
        for (key, value) in self.static_global_variables.iter() {
            variables.insert(key.clone(), value.clone());
        }
        variables
    }

    fn evaluate_static_xpath(&self, xpath: &str) -> Result<Sequence, xee_xpath::SpannedError> {
        // TODO: unwrap here ignores any possible errors when obtaining global variables
        let static_global_variables = self.static_global_variables();
        evaluate_without_focus_with_variables(xpath, static_global_variables)
    }

    fn static_param_instruction(&self, node: Node, context: Context) -> Option<ast::Param> {
        let element = self.state.xot.element(node)?;
        let content = Content::new(node, &self.state, context);
        let attributes = Attributes::new(content, element);
        // TODO: we don't handle standard attributes, so unseen attributes
        // will complain if we use one
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

    fn static_variable_instruction(&self, node: Node, context: Context) -> Option<ast::Variable> {
        let element = self.state.xot.element(node)?;
        let content = Content::new(node, &self.state, context);
        let attributes = Attributes::new(content, element);
        if let Ok(variable) = ast::Variable::parse_and_validate(&attributes) {
            if variable.static_ {
                return Some(variable);
            }
        }
        None
    }
}

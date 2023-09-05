// https://www.w3.org/TR/xpath-functions-31/#accessors
use xee_xpath_ast::ast;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::xml;
use crate::DynamicContext;

#[xpath_fn("fn:name($arg as node()?) as xs:string", context_first)]
fn name(context: &DynamicContext, arg: Option<xml::Node>) -> String {
    if let Some(node) = arg {
        let name = node.node_name(context.xot);
        if let Some(name) = name {
            name.to_full_name()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(name)]
}

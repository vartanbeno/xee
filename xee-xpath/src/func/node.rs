// https://www.w3.org/TR/xpath-functions-31/#accessors

use std::rc::Rc;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context::StaticFunctionDescription;
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

#[xpath_fn("fn:local-name($arg as node()?) as xs:string", context_first)]
fn local_name(context: &DynamicContext, arg: Option<xml::Node>) -> String {
    if let Some(arg) = arg {
        arg.local_name(context.xot)
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:namespace-uri($arg as node()?) as xs:anyURI", context_first)]
fn namespace_uri(context: &DynamicContext, arg: Option<xml::Node>) -> atomic::Atomic {
    if let Some(arg) = arg {
        atomic::Atomic::String(
            atomic::StringType::AnyURI,
            Rc::new(arg.namespace_uri(context.xot)),
        )
    } else {
        atomic::Atomic::String(atomic::StringType::AnyURI, "".to_string().into())
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(name),
        wrap_xpath_fn!(local_name),
        wrap_xpath_fn!(namespace_uri),
    ]
}

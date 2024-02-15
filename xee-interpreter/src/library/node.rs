// https://www.w3.org/TR/xpath-functions-31/#node-functions

use ahash::HashSet;
use ahash::HashSetExt;
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::error;
use crate::function::StaticFunctionDescription;
use crate::interpreter::Interpreter;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:name($arg as node()?) as xs:string", context_first)]
fn name(interpreter: &Interpreter, arg: Option<xot::Node>) -> error::Result<String> {
    Ok(if let Some(node) = arg {
        let name = interpreter.xot().node_name(node);
        if let Some(name) = name {
            interpreter.xot().full_name(node, name)?
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    })
}

#[xpath_fn("fn:local-name($arg as node()?) as xs:string", context_first)]
fn local_name(interpreter: &Interpreter, arg: Option<xot::Node>) -> String {
    if let Some(arg) = arg {
        let name = interpreter.xot().node_name(arg);
        if let Some(name) = name {
            interpreter.xot().local_name_str(name).to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:namespace-uri($arg as node()?) as xs:anyURI", context_first)]
fn namespace_uri(interpreter: &Interpreter, arg: Option<xot::Node>) -> atomic::Atomic {
    let uri = if let Some(arg) = arg {
        let name = interpreter.xot().node_name(arg);
        if let Some(name) = name {
            interpreter.xot().uri_str(name).to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    atomic::Atomic::String(atomic::StringType::AnyURI, uri.into())
}

#[xpath_fn("fn:root($arg as node()?) as node()?", context_first)]
fn root(interpreter: &Interpreter, arg: Option<xot::Node>) -> Option<xot::Node> {
    if let Some(arg) = arg {
        Some(interpreter.xot().root(arg))
    } else {
        None
    }
}

#[xpath_fn("fn:has-children($node as node()?) as xs:boolean", context_first)]
fn has_children(interpreter: &Interpreter, node: Option<xot::Node>) -> bool {
    if let Some(node) = node {
        interpreter.xot().first_child(node).is_some()
    } else {
        false
    }
}

#[xpath_fn("fn:innermost($nodes as node()*) as node()*")]
fn innermost(interpreter: &Interpreter, nodes: &[xot::Node]) -> Vec<xot::Node> {
    // get sequence of ancestors
    let mut ancestors = HashSet::new();
    for node in nodes {
        let mut parent_node = *node;
        // insert all parents into ancestors
        while let Some(parent) = interpreter.xot().parent(parent_node) {
            ancestors.insert(parent);
            parent_node = parent;
        }
    }
    // now find all nodes that are not in ancestors
    let mut innermost = Vec::new();
    for node in nodes {
        if !ancestors.contains(node) {
            innermost.push(*node);
        }
    }
    innermost
}

#[xpath_fn("fn:outermost($nodes as node()*) as node()*")]
fn outermost(interpreter: &Interpreter, nodes: &[xot::Node]) -> Vec<xot::Node> {
    let node_set = nodes.iter().collect::<HashSet<_>>();
    // now find all nodes that don't have an ancestor in the set
    let mut outermost = Vec::new();
    'outer: for node in nodes {
        let mut parent_node = *node;
        // if we find an ancestor in node_set, then we don't add this node
        while let Some(parent) = interpreter.xot().parent(parent_node) {
            if node_set.contains(&parent) {
                continue 'outer;
            }
            parent_node = parent;
        }
        outermost.push(*node);
    }
    outermost
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(name),
        wrap_xpath_fn!(local_name),
        wrap_xpath_fn!(namespace_uri),
        wrap_xpath_fn!(root),
        wrap_xpath_fn!(has_children),
        wrap_xpath_fn!(innermost),
        wrap_xpath_fn!(outermost),
    ]
}

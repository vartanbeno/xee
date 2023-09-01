use ibig::IBig;
use xee_xpath_ast::{ast, FN_NAMESPACE, XS_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context::{DynamicContext, FunctionKind, StaticFunctionDescription};
use crate::error;
use crate::occurrence::Occurrence;
use crate::sequence;
use crate::wrap_xpath_fn;
use crate::xml;

#[xpath_fn("fn:my_function($a as xs:integer, $b as xs:integer) as xs:integer")]
fn my_function(a: IBig, b: IBig) -> IBig {
    a + b
}

fn bound_position(
    _context: &DynamicContext,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    if arguments[0].is_absent() {
        return Err(error::Error::ComponentAbsentInDynamicContext);
    }
    // position should be the context value
    Ok(arguments[0].clone())
}

fn bound_last(
    _context: &DynamicContext,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    if arguments[0].is_absent() {
        return Err(error::Error::ComponentAbsentInDynamicContext);
    }
    // size should be the context value
    Ok(arguments[0].clone())
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
fn namespace_uri(context: &DynamicContext, arg: Option<xml::Node>) -> String {
    if let Some(arg) = arg {
        arg.namespace_uri(context.xot)
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:count($arg as item()*) as xs:integer")]
fn count(arg: &[sequence::Item]) -> IBig {
    arg.len().into()
}

#[xpath_fn("fn:root($arg as node()?) as node()?", context_first)]
fn root(context: &DynamicContext, arg: Option<xml::Node>) -> Option<xml::Node> {
    if let Some(arg) = arg {
        let xot_node = match arg {
            xml::Node::Xot(node) => node,
            xml::Node::Attribute(node, _) => node,
            xml::Node::Namespace(node, _) => node,
        };
        // XXX there should be a xot.root() to obtain this in one step
        let top = context.xot.top_element(xot_node);
        let root = context.xot.parent(top).unwrap();

        Some(xml::Node::Xot(root))
    } else {
        None
    }
}

#[xpath_fn("fn:string($arg as item()?) as xs:string", context_first)]
fn string(context: &DynamicContext, arg: Option<sequence::Item>) -> error::Result<String> {
    if let Some(arg) = arg {
        arg.string_value(context.xot)
    } else {
        Ok("".to_string())
    }
}

#[xpath_fn("fn:exists($arg as item()*) as xs:boolean")]
fn exists(arg: &[sequence::Item]) -> bool {
    !arg.is_empty()
}

#[xpath_fn("fn:exactly-one($arg as item()*) as item()")]
fn exactly_one(arg: &[sequence::Item]) -> error::Result<sequence::Item> {
    if arg.len() == 1 {
        Ok(arg[0].clone())
    } else {
        Err(error::Error::FORG0005)
    }
}

#[xpath_fn("fn:empty($arg as item()*) as xs:boolean")]
fn empty(arg: &[sequence::Item]) -> bool {
    arg.is_empty()
}

#[xpath_fn("fn:generate-id($arg as node()?) as xs:string", context_first)]
fn generate_id(context: &DynamicContext, arg: Option<xml::Node>) -> String {
    if let Some(arg) = arg {
        let annotations = &context.documents.annotations;
        let annotation = annotations.get(arg).unwrap();
        annotation.generate_id()
    } else {
        "".to_string()
    }
}

fn untyped_atomic(
    context: &DynamicContext,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    let a = &arguments[0];
    let value = a.atomized(context.xot).one()?;
    // TODO: this needs more work to implement:
    // https://www.w3.org/TR/xpath-functions-31/#casting-to-string
    let s: String = value.try_into()?;
    Ok(sequence::Sequence::from(vec![sequence::Item::from(
        atomic::Atomic::from(s),
    )]))
}

fn error(
    _context: &DynamicContext,
    _arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    Err(error::Error::FOER0000)
}

// #[xpath_fn("fn:node-name($arg as node()?) as xs:QName?", context_first)]
// fn node_name(context: &DynamicContext, arg: Option<xml::Node>) -> Option<ast::Name> {
//     if let Some(node) = arg {
//         Some(node.node_name(context.xot))
//     } else {
//         None
//     }
// }

#[xpath_fn("fn:remove($target as item()*, $position as xs:integer) as item()*")]
fn remove(target: &[sequence::Item], position: IBig) -> error::Result<sequence::Sequence> {
    let position: usize = position.try_into().map_err(|_| error::Error::Overflow)?;
    if position == 0 || position > target.len() {
        // TODO: unfortunate we can't just copy sequence
        return Ok(target.to_vec().into());
    }
    let mut target = target.to_vec();
    target.remove(position - 1);
    Ok(target.into())
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(my_function),
        StaticFunctionDescription {
            name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_kind: Some(FunctionKind::Position),
            func: bound_position,
        },
        StaticFunctionDescription {
            name: ast::Name::new("last".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_kind: Some(FunctionKind::Size),
            func: bound_last,
        },
        wrap_xpath_fn!(local_name),
        wrap_xpath_fn!(namespace_uri),
        wrap_xpath_fn!(count),
        wrap_xpath_fn!(root),
        wrap_xpath_fn!(string),
        wrap_xpath_fn!(exists),
        wrap_xpath_fn!(exactly_one),
        wrap_xpath_fn!(empty),
        wrap_xpath_fn!(generate_id),
        StaticFunctionDescription {
            name: ast::Name::new("untypedAtomic".to_string(), Some(XS_NAMESPACE.to_string())),
            arity: 1,
            function_kind: None,
            func: untyped_atomic,
        },
        StaticFunctionDescription {
            name: ast::Name::new("error".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_kind: None,
            func: error,
        },
        wrap_xpath_fn!(remove),
    ]
}

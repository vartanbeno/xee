use xee_xpath_ast::{ast, FN_NAMESPACE, XS_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::context::{DynamicContext, FunctionKind, StaticFunctionDescription};
use crate::error;
use crate::occurrence::ResultOccurrence;
use crate::output;
use crate::wrap_xpath_fn;
use crate::xml;

#[xpath_fn("my_function($a as xs:int, $b as xs:int) as xs:int")]
fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_position(
    _context: &DynamicContext,
    arguments: &[output::Sequence],
) -> error::Result<output::Sequence> {
    if arguments[0].is_absent() {
        return Err(error::Error::XPDY0002A);
    }
    // position should be the context value
    Ok(arguments[0].clone())
}

fn bound_last(
    _context: &DynamicContext,
    arguments: &[output::Sequence],
) -> error::Result<output::Sequence> {
    if arguments[0].is_absent() {
        return Err(error::Error::XPDY0002A);
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
fn count(arg: &[output::Item]) -> i64 {
    arg.len() as i64
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
fn string(context: &DynamicContext, arg: Option<output::Item>) -> error::Result<String> {
    if let Some(arg) = arg {
        arg.string_value(context.xot)
    } else {
        Ok("".to_string())
    }
}

#[xpath_fn("fn:exists($arg as item()*) as xs:boolean")]
fn exists(arg: &[output::Item]) -> bool {
    !arg.is_empty()
}

#[xpath_fn("fn:exactly-one($arg as item()*) as item()")]
fn exactly_one(arg: &[output::Item]) -> error::Result<output::Item> {
    if arg.len() == 1 {
        Ok(arg[0].clone())
    } else {
        Err(error::Error::FORG0005)
    }
}

#[xpath_fn("fn:empty($arg as item()*) as xs:boolean")]
fn empty(arg: &[output::Item]) -> bool {
    arg.is_empty()
}

fn not(
    _context: &DynamicContext,
    arguments: &[output::Sequence],
) -> error::Result<output::Sequence> {
    let a = &arguments[0];
    let b = a.effective_boolean_value()?;
    Ok(output::Sequence::from(vec![output::Item::from(
        output::Atomic::from(!b),
    )]))
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
    arguments: &[output::Sequence],
) -> error::Result<output::Sequence> {
    let a = &arguments[0];
    let value = a.atomized(context.xot).one()?;
    // TODO: this needs more work to implement:
    // https://www.w3.org/TR/xpath-functions-31/#casting-to-string
    let s: String = value.try_into()?;
    Ok(output::Sequence::from(vec![output::Item::from(
        output::Atomic::from(s),
    )]))
}

fn error(
    _context: &DynamicContext,
    _arguments: &[output::Sequence],
) -> error::Result<output::Sequence> {
    Err(error::Error::FOER0000)
}

#[xpath_fn("fn:true() as xs:boolean")]
fn true_() -> bool {
    true
}

#[xpath_fn("fn:false() as xs:boolean")]
fn false_() -> bool {
    false
}

// #[xpath_fn("fn:string-join($arg1 as xs:anyAtomicType*, $arg2 as xs:string) as xs:string")]
// fn string_join(arg1: &[Atomic], arg2: &str) -> String {
//     let mut s = String::new();
//     for (i, a) in arg1.iter().enumerate() {
//         if i > 0 {
//             s.push_str(arg2);
//         }
//         s.push_str(&a.to_string());
//     }
//     s
// }

// Experimental exploration of wrapping with converters
// fn wrap_math_exp(context: &DynamicContext, arguments: &[Value]) -> Result<Value, stack::ValueError> {
//     let a = &arguments[0];
//     let a = a.context_try_into(context)?;
//     Ok(real_math_exp(a).into())
// }

// fn real_math_exp(d: Option<f64>) -> Option<f64> {
//     d.map(|d| d.exp())
// }

// fn wrap_local_name(context: &DynamicContext, arguments: &[Value]) -> Result<Value, stack::ValueError> {
//     let a = (&arguments[0]).try_into()?;
//     Ok(real_local_name(context, a).into())
// }

// // #[xpath_fn]
// fn real_local_name(context: &DynamicContext, a: xml::Node) -> String {
//     a.local_name(context.xot)
// }

// fn wrap_extract_one(context: &DynamicContext, arguments: &[Value]) -> Result<Value, stack::ValueError> {
//     let a = &arguments[0];
//     let a: Sequence = a.try_into()?;
//     let a = a.borrow();
//     let s = a.as_slice();
//     Ok(real_exactly_one(s)?.into())
// }

// // #[xpath_fn]
// fn real_exactly_one(a: &[Item]) -> Result<Item, stack::ValueError> {
//     if a.len() == 1 {
//         Ok(a[0].clone())
//     } else {
//         // XXX should really be a FORG0005 error
//         Err(stack::ValueError::Type)
//     }
// }

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
            name: ast::Name::new("not".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_kind: None,
            func: not,
        },
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
        wrap_xpath_fn!(true_),
        wrap_xpath_fn!(false_),
    ]
}

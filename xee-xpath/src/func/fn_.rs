use std::rc::Rc;

use xee_xpath_ast::{ast, FN_NAMESPACE, XS_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::context::{FunctionType, StaticFunctionDescription};
use crate::{
    data::{ContextTryInto, Item, Sequence, ValueError},
    Atomic, DynamicContext, Error, Node, Value,
};

#[xpath_fn("my_function($a as xs:int, $b as xs:int) as xs:int")]
fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_position(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    if arguments[0] == Value::Atomic(Atomic::Absent) {
        return Err(ValueError::Absent);
    }
    // position should be the context value
    Ok(arguments[0].clone())
}

fn bound_last(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    if arguments[0] == Value::Atomic(Atomic::Absent) {
        return Err(ValueError::Absent);
    }
    // size should be the context value
    Ok(arguments[0].clone())
}

#[xpath_fn("fn:local-name($arg as node()?) as xs:string")]
fn local_name(context: &DynamicContext, arg: Option<Node>) -> String {
    if let Some(arg) = arg {
        arg.local_name(context.xot)
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:namespace-uri($arg as node()?) as xs:anyURI")]
fn namespace_uri(context: &DynamicContext, arg: Option<Node>) -> String {
    if let Some(arg) = arg {
        arg.namespace_uri(context.xot)
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:count($arg as item()*) as xs:integer")]
fn count(arg: &[Item]) -> i64 {
    arg.len() as i64
}

#[xpath_fn("fn:root($arg as node()?) as node()?")]
fn root(context: &DynamicContext, arg: Option<Node>) -> Option<Node> {
    if let Some(arg) = arg {
        let xot_node = match arg {
            Node::Xot(node) => node,
            Node::Attribute(node, _) => node,
            Node::Namespace(node, _) => node,
        };
        // XXX there should be a xot.root() to obtain this in one step
        let top = context.xot.top_element(xot_node);
        let root = context.xot.parent(top).unwrap();

        Some(Node::Xot(root))
    } else {
        None
    }
}

// fn root(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
//     let a: Node = (&arguments[0]).try_into()?;
//     let xot_node = match a {
//         Node::Xot(node) => node,
//         Node::Attribute(node, _) => node,
//         Node::Namespace(node, _) => node,
//     };

//     let top = context.xot.top_element(xot_node);
//     let root = context.xot.parent(top).unwrap();
//     Ok(Value::Node(Node::Xot(root)))
// }

fn string(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    Ok(Value::Atomic(Atomic::String(Rc::new(
        arguments[0].string_value(context.xot)?,
    ))))
}

fn exists(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = &arguments[0];
    Ok(Value::Atomic(Atomic::Boolean(!a.is_empty_sequence())))
}

// #[xpath_fn]
// fn exactly_one(context: &DynamicContext, a: &[Item]) -> Result<Item, ValueError> {
//     if a.len() == 1 {
//         Ok(a[0])
//     } else {
//         // XXX should really be a FORG0005 error
//         Err(ValueError::Type)
//     }
// }

fn exactly_one(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a: Sequence = (&arguments[0]).try_into()?;
    let a = a.borrow();
    if a.items.len() == 1 {
        Ok(Value::from_item(a.items[0].clone()))
    } else {
        // XXX should really be a FORG0005 error
        Err(ValueError::Type)
    }
}

fn empty(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = &arguments[0];
    Ok(Value::Atomic(Atomic::Boolean(a.is_empty_sequence())))
}

fn not(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = &arguments[0];
    let b = a.effective_boolean_value()?;
    Ok(Value::Atomic(Atomic::Boolean(!b)))
}

fn generate_id(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = &arguments[0];
    if a.is_empty_sequence() {
        return Ok(Value::Atomic(Atomic::String(Rc::new("".to_string()))));
    }
    let annotations = &context.documents.annotations;
    let node = a.try_into()?;
    let annotation = annotations.get(node).unwrap();
    Ok(Value::Atomic(Atomic::String(Rc::new(
        annotation.generate_id(),
    ))))
}

fn untyped_atomic(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = &arguments[0];
    let a: Atomic = a.context_try_into(context)?;
    let s = a.try_into()?;
    Ok(Value::Atomic(Atomic::Untyped(Rc::new(s))))
}

fn error(_context: &DynamicContext, _arguments: &[Value]) -> Result<Value, ValueError> {
    Err(ValueError::Error(Error::FOER0000))
}

fn true_(_context: &DynamicContext, _arguments: &[Value]) -> Result<Value, ValueError> {
    Ok(Value::Atomic(Atomic::Boolean(true)))
}

fn false_(_context: &DynamicContext, _arguments: &[Value]) -> Result<Value, ValueError> {
    Ok(Value::Atomic(Atomic::Boolean(false)))
}

// Experimental exploration of wrapping with converters
// fn wrap_math_exp(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
//     let a = &arguments[0];
//     let a = a.context_try_into(context)?;
//     Ok(real_math_exp(a).into())
// }

// fn real_math_exp(d: Option<f64>) -> Option<f64> {
//     d.map(|d| d.exp())
// }

// fn wrap_local_name(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
//     let a = (&arguments[0]).try_into()?;
//     Ok(real_local_name(context, a).into())
// }

// // #[xpath_fn]
// fn real_local_name(context: &DynamicContext, a: Node) -> String {
//     a.local_name(context.xot)
// }

// fn wrap_extract_one(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
//     let a = &arguments[0];
//     let a: Sequence = a.try_into()?;
//     let a = a.borrow();
//     let s = a.as_slice();
//     Ok(real_exactly_one(s)?.into())
// }

// // #[xpath_fn]
// fn real_exactly_one(a: &[Item]) -> Result<Item, ValueError> {
//     if a.len() == 1 {
//         Ok(a[0].clone())
//     } else {
//         // XXX should really be a FORG0005 error
//         Err(ValueError::Type)
//     }
// }

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        StaticFunctionDescription {
            name: ast::Name::new("my_function".to_string(), None),
            arity: 2,
            function_type: None,
            func: wrapper_my_function,
        },
        StaticFunctionDescription {
            name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: Some(FunctionType::Position),
            func: bound_position,
        },
        StaticFunctionDescription {
            name: ast::Name::new("last".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: Some(FunctionType::Size),
            func: bound_last,
        },
        StaticFunctionDescription {
            name: ast::Name::new("local-name".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: wrapper_local_name,
        },
        StaticFunctionDescription {
            name: ast::Name::new("namespace-uri".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: wrapper_namespace_uri,
        },
        StaticFunctionDescription {
            name: ast::Name::new("count".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: wrapper_count,
        },
        StaticFunctionDescription {
            name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: wrapper_root,
        },
        StaticFunctionDescription {
            name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: string,
        },
        StaticFunctionDescription {
            name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: string,
        },
        StaticFunctionDescription {
            name: ast::Name::new("exists".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: exists,
        },
        StaticFunctionDescription {
            name: ast::Name::new("exactly-one".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: exactly_one,
        },
        StaticFunctionDescription {
            name: ast::Name::new("empty".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: empty,
        },
        StaticFunctionDescription {
            name: ast::Name::new("generate-id".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: generate_id,
        },
        StaticFunctionDescription {
            name: ast::Name::new("not".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: not,
        },
        StaticFunctionDescription {
            name: ast::Name::new("untypedAtomic".to_string(), Some(XS_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: untyped_atomic,
        },
        StaticFunctionDescription {
            name: ast::Name::new("error".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: None,
            func: error,
        },
        StaticFunctionDescription {
            name: ast::Name::new("true".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: None,
            func: true_,
        },
        StaticFunctionDescription {
            name: ast::Name::new("false".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: None,
            func: false_,
        },
    ]
}

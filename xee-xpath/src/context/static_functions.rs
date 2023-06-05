use std::rc::Rc;

use crate::{
    ast,
    context::namespaces::{FN_NAMESPACE, XS_NAMESPACE},
    context::static_context::{FunctionType, StaticFunctionDescription},
    data::{ContextTryInto, ValueError},
    Atomic, DynamicContext, Error, Node, Value,
};

fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_my_function(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a: Atomic = (&arguments[0]).context_try_into(context)?;
    let b: Atomic = (&arguments[1]).context_try_into(context)?;
    Ok(Value::Atomic(Atomic::Integer(my_function(
        a.try_into()?,
        b.try_into()?,
    ))))
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

// #[xpath_fn]
// fn local_name(context: &DynamicContext, a: Node) -> String {
//     a.local_name(context.xot)
// }

fn local_name(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = arguments[0].to_node()?;
    Ok(Value::Atomic(Atomic::String(Rc::new(
        a.local_name(context.xot),
    ))))
}

fn namespace_uri(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = arguments[0].to_node()?;
    Ok(Value::Atomic(Atomic::String(Rc::new(
        a.namespace_uri(context.xot),
    ))))
}

fn count(_context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = arguments[0].to_sequence()?;
    let a = a.borrow();
    Ok(Value::Atomic(Atomic::Integer(a.items.len() as i64)))
}

fn root(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError> {
    let a = arguments[0].to_node()?;
    let xot_node = match a {
        Node::Xot(node) => node,
        Node::Attribute(node, _) => node,
        Node::Namespace(node, _) => node,
    };
    // XXX there should be a xot.root() to obtain this in one step
    let top = context.xot.top_element(xot_node);
    let root = context.xot.parent(top).unwrap();
    Ok(Value::Node(Node::Xot(root)))
}

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
    let a = arguments[0].to_sequence()?;
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
    let node = a.to_node()?;
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

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        StaticFunctionDescription {
            name: ast::Name::new("my_function".to_string(), None),
            arity: 2,
            function_type: None,
            func: bound_my_function,
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
            func: local_name,
        },
        StaticFunctionDescription {
            name: ast::Name::new("namespace-uri".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: namespace_uri,
        },
        StaticFunctionDescription {
            name: ast::Name::new("count".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: count,
        },
        StaticFunctionDescription {
            name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: root,
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

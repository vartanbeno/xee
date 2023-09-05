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

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(my_function),
        StaticFunctionDescription {
            name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string()), None),
            arity: 0,
            function_kind: Some(FunctionKind::Position),
            func: bound_position,
        },
        StaticFunctionDescription {
            name: ast::Name::new("last".to_string(), Some(FN_NAMESPACE.to_string()), None),
            arity: 0,
            function_kind: Some(FunctionKind::Size),
            func: bound_last,
        },
        wrap_xpath_fn!(generate_id),
        StaticFunctionDescription {
            name: ast::Name::new(
                "untypedAtomic".to_string(),
                Some(XS_NAMESPACE.to_string()),
                None,
            ),
            arity: 1,
            function_kind: None,
            func: untyped_atomic,
        },
        StaticFunctionDescription {
            name: ast::Name::new("error".to_string(), Some(FN_NAMESPACE.to_string()), None),
            arity: 0,
            function_kind: None,
            func: error,
        },
    ]
}

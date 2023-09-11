// https://www.w3.org/TR/2017/REC-xpath-functions-31-20170321/#context

use xee_xpath_ast::ast;
use xee_xpath_ast::FN_NAMESPACE;

use crate::context::FunctionKind;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::DynamicContext;

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

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
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
    ]
}

// https://www.w3.org/TR/2017/REC-xpath-functions-31-20170321/#context

use crate::interpreter;
use crate::wrap_xpath_fn;
use crate::NaiveDateWithOffset;
use crate::NaiveTimeWithOffset;
use xee_xpath_ast::ast;
use xee_xpath_ast::FN_NAMESPACE;
use xee_xpath_macros::xpath_fn;

use crate::context::FunctionKind;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::sequence;
use crate::DynamicContext;

use super::datetime::offset_to_duration;

fn bound_position(
    _context: &DynamicContext,
    _interpreter: &mut interpreter::Interpreter,
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
    _interpreter: &mut interpreter::Interpreter,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    if arguments[0].is_absent() {
        return Err(error::Error::ComponentAbsentInDynamicContext);
    }
    // size should be the context value
    Ok(arguments[0].clone())
}

#[xpath_fn("fn:current-dateTime() as xs:dateTimeStamp")]
fn current_date_time(context: &DynamicContext) -> chrono::DateTime<chrono::offset::FixedOffset> {
    context.current_datetime()
}

#[xpath_fn("fn:current-date() as xs:date")]
fn current_date(context: &DynamicContext) -> NaiveDateWithOffset {
    NaiveDateWithOffset {
        date: context.current_datetime().date_naive(),
        offset: Some(context.implicit_timezone()),
    }
}

#[xpath_fn("fn:current-time() as xs:time")]
fn current_time(context: &DynamicContext) -> NaiveTimeWithOffset {
    NaiveTimeWithOffset {
        time: context.current_datetime().time(),
        offset: Some(context.implicit_timezone()),
    }
}

#[xpath_fn("fn:implicit-timezone() as xs:dayTimeDuration")]
fn implicit_timezone(context: &DynamicContext) -> chrono::Duration {
    offset_to_duration(context.implicit_timezone())
}

#[xpath_fn("fn:default-collation() as xs:string")]
fn default_collation(context: &DynamicContext) -> String {
    context.static_context.default_collation_uri().to_string()
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
        wrap_xpath_fn!(current_date_time),
        wrap_xpath_fn!(current_date),
        wrap_xpath_fn!(current_time),
        wrap_xpath_fn!(implicit_timezone),
        wrap_xpath_fn!(default_collation),
    ]
}

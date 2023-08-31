use std::cmp::Ordering;

use ibig::IBig;
use xee_xpath_ast::{ast, FN_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::context::{DynamicContext, StaticFunctionDescription};
use crate::{atomic, error, sequence, wrap_xpath_fn, Occurrence};

// we don't accept concat() invocations with an arity
// of greater than this
const MAX_CONCAT_ARITY: usize = 32;

// https://www.w3.org/TR/xpath-functions-31/#string-functions
#[xpath_fn(
    "fn:compare($arg1 as xs:string?, $arg2 as xs:string?, $collation as xs:string) as xs:integer?",
    collation
)]
fn compare(
    context: &DynamicContext,
    arg1: Option<&str>,
    arg2: Option<&str>,
    collation: &str,
) -> error::Result<Option<IBig>> {
    if let (Some(arg1), Some(arg2)) = (arg1, arg2) {
        let collator = context.static_context.collation(collation)?;
        Ok(Some(
            match collator.compare(arg1, arg2) {
                Ordering::Equal => 0,
                Ordering::Less => -1,
                Ordering::Greater => 1,
            }
            .into(),
        ))
    } else {
        Ok(None)
    }
}

// concat cannot be written using the macro system, as it
// takes an arbitrary amount of arguments. This is the only
// function that does this. We're going to define a general
// concat function and then register it for a sufficient amount
// of arities
fn concat(
    context: &DynamicContext,
    arguments: &[sequence::Sequence],
) -> error::Result<sequence::Sequence> {
    debug_assert!(arguments.len() >= 2);

    let strings = arguments
        .iter()
        .map(|argument| {
            let atomic = argument.atomized(context.xot).option()?;
            if let Some(atomic) = atomic {
                atomic.string_value()
            } else {
                Ok("".to_string())
            }
        })
        .collect::<error::Result<Vec<String>>>()?;
    Ok(strings.concat().into())
}

#[xpath_fn("fn:string-join($arg1 as xs:anyAtomicType*) as xs:string")]
fn string_join(arg1: &[atomic::Atomic]) -> error::Result<String> {
    let arg1 = arg1
        .iter()
        .map(|a| a.string_value())
        .collect::<error::Result<Vec<String>>>()?;
    Ok(arg1.concat())
}

#[xpath_fn("fn:string-join($arg1 as xs:anyAtomicType*, $arg2 as xs:string) as xs:string")]
fn string_join_sep(arg1: &[atomic::Atomic], arg2: &str) -> error::Result<String> {
    let arg1 = arg1
        .iter()
        .map(|a| a.string_value())
        .collect::<error::Result<Vec<String>>>()?;
    Ok(arg1.join(arg2))
}

#[xpath_fn("fn:string-length($arg as xs:string?) as xs:integer", context_first)]
fn string_length(arg: Option<&str>) -> IBig {
    if let Some(arg) = arg {
        // TODO: what about overflow? not a very realistic situation
        arg.chars().count().into()
    } else {
        0.into()
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    let mut r = vec![
        wrap_xpath_fn!(compare),
        wrap_xpath_fn!(string_join),
        wrap_xpath_fn!(string_join_sep),
        wrap_xpath_fn!(string_length),
    ];
    // register concat for a variety of arities
    // it's stupid that we have to do this, but it's in the
    // spec https://www.w3.org/TR/xpath-functions-31/#func-concat
    for arity in 2..MAX_CONCAT_ARITY {
        r.push(StaticFunctionDescription {
            name: ast::Name::new("concat".to_string(), Some(FN_NAMESPACE.to_string())),
            arity,
            function_kind: None,
            func: concat,
        });
    }
    r
}

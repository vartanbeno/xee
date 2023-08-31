use std::cmp::Ordering;

use ibig::IBig;
use xee_xpath_ast::{ast, FN_NAMESPACE};
use xee_xpath_macros::xpath_fn;

use crate::context::{DynamicContext, StaticFunctionDescription};
use crate::{atomic, error, sequence, wrap_xpath_fn, Occurrence};

// we don't accept concat() invocations with an arity
// of greater than this
const MAX_CONCAT_ARITY: usize = 32;

#[xpath_fn("fn:codepoints-to-string($arg as xs:integer*) as xs:string")]
fn codepoints_to_string(arg: &[IBig]) -> error::Result<String> {
    arg.iter()
        .map(|c| {
            let c: u32 = c.try_into().map_err(|_| error::Error::FOCH0001)?;
            char::from_u32(c).ok_or(error::Error::FOCH0001)
        })
        .collect::<error::Result<String>>()
}

#[xpath_fn("fn:string-to-codepoints($arg as xs:string?) as xs:integer*")]
fn string_to_codepoints(arg: Option<&str>) -> error::Result<Vec<IBig>> {
    if let Some(arg) = arg {
        Ok(arg.chars().map(|c| c as u32).map(IBig::from).collect())
    } else {
        // empty sequence
        Ok(Vec::new())
    }
}

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
        let collation = context.static_context.collation(collation)?;
        Ok(Some(
            match collation.compare(arg1, arg2) {
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

#[xpath_fn(
    "fn:codepoint-equal($comparand1 as xs:string?, $comparand2 as xs:string?) as xs:boolean?"
)]
fn codepoint_equal(comparand1: Option<&str>, comparand2: Option<&str>) -> Option<bool> {
    if let (Some(comparand1), Some(comparand2)) = (comparand1, comparand2) {
        Some(comparand1 == comparand2)
    } else {
        None
    }
}

#[xpath_fn(
    "fn:contains-token($input as xs:string*, $token as xs:string, $collation as xs:string) as xs:boolean",
    collation
)]
fn contains_token(
    context: &DynamicContext,
    input: &[&str],
    token: &str,
    collation: &str,
) -> error::Result<bool> {
    if input.is_empty() {
        return Ok(false);
    }
    let collation = context.static_context.collation(collation)?;
    let token = token.trim();
    for s in input {
        // if any token in s, tokenized, is token, then we return true
        if s.split_whitespace()
            .any(|t| collation.compare(t, token).is_eq())
        {
            return Ok(true);
        }
    }
    Ok(false)
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

#[xpath_fn("fn:substring($sourceString as xs:string?, $start as xs:double) as xs:string")]
fn substring2(source_string: Option<&str>, start: f64) -> String {
    substring_with_length(source_string, start, usize::MAX)
}

#[xpath_fn("fn:substring($sourceString as xs:string?, $start as xs:double, $length as xs:double) as xs:string")]
fn substring3(source_string: Option<&str>, start: f64, length: f64) -> String {
    let length = length.round();
    if length < 0.0 {
        return "".to_string();
    }
    substring_with_length(source_string, start, length as usize)
}

fn substring_with_length(source_string: Option<&str>, start: f64, length: usize) -> String {
    let start = start.round();
    let start = start as i64 - 1;
    // substract any negative start from the length
    let (start, length) = if start < 0 {
        (0, length - start.unsigned_abs() as usize)
    } else {
        (start as usize, length)
    };
    if let Some(source_string) = source_string {
        if source_string.is_empty() {
            return "".to_string();
        }
        source_string
            .chars()
            .skip(start)
            .take(length)
            .collect::<String>()
    } else {
        "".to_string()
    }
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

#[xpath_fn("fn:normalize-space($arg as xs:string?) as xs:string", context_first)]
fn normalize_space(arg: Option<&str>) -> String {
    if let Some(arg) = arg {
        arg.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        "".to_string()
    }
}

#[xpath_fn("fn:tokenize($input as xs:string?) as xs:string*")]
fn tokenize1(input: Option<&str>) -> error::Result<Vec<String>> {
    if let Some(input) = input {
        Ok(input.split_whitespace().map(|s| s.to_string()).collect())
    } else {
        Ok(Vec::new())
    }
}

#[xpath_fn("fn:tokenize($input as xs:string?, $pattern as xs:string) as xs:string*")]
fn tokenize2(input: Option<&str>, pattern: &str) -> error::Result<Vec<String>> {
    if let Some(input) = input {
        Ok(input.split(pattern).map(|s| s.to_string()).collect())
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    let mut r = vec![
        wrap_xpath_fn!(codepoints_to_string),
        wrap_xpath_fn!(string_to_codepoints),
        wrap_xpath_fn!(compare),
        wrap_xpath_fn!(codepoint_equal),
        wrap_xpath_fn!(contains_token),
        wrap_xpath_fn!(string_join),
        wrap_xpath_fn!(string_join_sep),
        wrap_xpath_fn!(substring2),
        wrap_xpath_fn!(substring3),
        wrap_xpath_fn!(string_length),
        wrap_xpath_fn!(normalize_space),
        wrap_xpath_fn!(tokenize1),
        wrap_xpath_fn!(tokenize2),
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

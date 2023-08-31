use std::cmp::Ordering;

use ibig::IBig;
use xee_xpath_macros::xpath_fn;

use crate::context::{DynamicContext, StaticFunctionDescription};
use crate::{error, wrap_xpath_fn};

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

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(compare)]
}

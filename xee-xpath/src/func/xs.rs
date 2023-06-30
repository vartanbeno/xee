use xee_xpath_macros::xpath_fn;

use crate::atomic;
use crate::context::StaticFunctionDescription;
use crate::error;
use crate::wrap_xpath_fn;

#[xpath_fn("xs:string($arg as xs:anyAtomicType?) as xs:string")]
fn xs_string(arg: Option<atomic::Atomic>) -> error::Result<String> {
    if let Some(arg) = arg {
        arg.string_value()
    } else {
        Ok("".to_string())
    }
}

#[xpath_fn("xs:int($arg as xs:anyAtomicType?) as xs:int?")]
fn xs_int(arg: Option<atomic::Atomic>) -> error::Result<Option<i64>> {
    if let Some(arg) = arg {
        // TODO: only handle the string case for now
        Ok(Some(
            arg.string_value()?
                .parse()
                .map_err(|_| error::Error::FORG0001)?,
        ))
    } else {
        Ok(None)
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(xs_string), wrap_xpath_fn!(xs_int)]
}

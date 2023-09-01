// https://www.w3.org/TR/xpath-functions-31/#durations

use ibig::IBig;
use xee_xpath_macros::xpath_fn;

use crate::atomic::Duration;
use crate::context::StaticFunctionDescription;

use crate::wrap_xpath_fn;

#[xpath_fn("fn:years-from-duration($arg as xs:duration?) as xs:integer?")]
fn years_from_duration(arg: Option<Duration>) -> Option<IBig> {
    if let Some(arg) = arg {
        Some(arg.year_month.years().into())
    } else {
        None
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(years_from_duration)]
}

// https://www.w3.org/TR/xpath-functions-31/#dates-times

use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::{
    error, wrap_xpath_fn, NaiveDateTimeWithOffset, NaiveDateWithOffset, NaiveTimeWithOffset,
};

#[xpath_fn("fn:dateTime($arg1 as xs:date?, $arg2 as xs:time?) as xs:dateTime?")]
fn date_time(
    arg1: Option<NaiveDateWithOffset>,
    arg2: Option<NaiveTimeWithOffset>,
) -> error::Result<Option<NaiveDateTimeWithOffset>> {
    match (arg1, arg2) {
        (Some(arg1), Some(arg2)) => {
            let offset = match (arg1.offset, arg2.offset) {
                (Some(arg1), Some(arg2)) => {
                    if arg1 == arg2 {
                        Some(arg1)
                    } else {
                        return Err(error::Error::FORG0008);
                    }
                }
                (Some(arg1), None) => Some(arg1),
                (None, Some(arg2)) => Some(arg2),
                (None, None) => None,
            };
            Ok(Some(NaiveDateTimeWithOffset::new(
                arg1.date.and_time(arg2.time),
                offset,
            )))
        }
        (Some(_), None) => Ok(None),
        (None, Some(_)) => Ok(None),
        (None, None) => Ok(None),
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(date_time)]
}

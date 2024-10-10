// https://www.w3.org/TR/xpath-functions-31/#numeric-functions
use ibig::ops::Abs;
use ibig::IBig;
use num_traits::Float;

use xee_xpath_macros::xpath_fn;

use crate::atomic::round_atomic;
use crate::atomic::round_half_to_even_atomic;
use crate::atomic::Atomic;
use crate::error;
use crate::function::StaticFunctionDescription;
use crate::wrap_xpath_fn;

#[xpath_fn("fn:abs($arg as xs:numeric?) as xs:numeric?")]
fn abs(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        match arg {
            Atomic::Integer(_, i) => Ok(Some(i.as_ref().abs().into())),
            Atomic::Decimal(d) => Ok(Some(d.abs().into())),
            Atomic::Float(f) => Ok(Some(f.abs().into())),
            Atomic::Double(d) => Ok(Some(d.abs().into())),
            _ => Err(error::Error::XPTY0004),
        }
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:ceiling($arg as xs:numeric?) as xs:numeric?")]
fn ceiling(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        match arg {
            Atomic::Integer(_, _) => Ok(Some(arg.clone())),
            Atomic::Decimal(d) => Ok(Some(d.ceil().into())),
            Atomic::Float(f) => Ok(Some(f.ceil().into())),
            Atomic::Double(d) => Ok(Some(d.ceil().into())),
            _ => Err(error::Error::XPTY0004),
        }
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:floor($arg as xs:numeric?) as xs:numeric?")]
fn floor(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        match arg {
            Atomic::Integer(_, _) => Ok(Some(arg.clone())),
            Atomic::Decimal(d) => Ok(Some(d.floor().into())),
            Atomic::Float(f) => Ok(Some(f.floor().into())),
            Atomic::Double(d) => Ok(Some(d.floor().into())),
            _ => Err(error::Error::XPTY0004),
        }
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:round($arg as xs:numeric?) as xs:numeric?")]
fn round1(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        round_atomic(arg, 0).map(Some)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:round($arg as xs:numeric?, $precision as xs:integer) as xs:numeric?")]
fn round2(arg: Option<Atomic>, precision: IBig) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        let precision: i32 = precision.try_into().map_err(|_| error::Error::FOAR0002)?;
        round_atomic(arg, precision).map(Some)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:round-half-to-even($arg as xs:numeric?) as xs:numeric?")]
fn round_half_to_even1(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        round_half_to_even_atomic(arg, 0).map(Some)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:round-half-to-even($arg as xs:numeric?, $precision as xs:integer) as xs:numeric?")]
fn round_half_to_even2(arg: Option<Atomic>, precision: IBig) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        let precision: i32 = precision.try_into().map_err(|_| error::Error::FOAR0002)?;
        round_half_to_even_atomic(arg, precision).map(Some)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:number($arg as xs:anyAtomicType?) as xs:double", context_first)]
fn number(arg: Option<Atomic>) -> error::Result<Atomic> {
    if let Some(arg) = arg {
        match arg.cast_to_double() {
            Ok(d) => Ok(d),
            Err(_) => Ok(f64::NAN.into()),
        }
    } else {
        Ok(f64::NAN.into())
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(abs),
        wrap_xpath_fn!(ceiling),
        wrap_xpath_fn!(floor),
        wrap_xpath_fn!(round1),
        wrap_xpath_fn!(round2),
        wrap_xpath_fn!(round_half_to_even1),
        wrap_xpath_fn!(round_half_to_even2),
        wrap_xpath_fn!(number),
    ]
}

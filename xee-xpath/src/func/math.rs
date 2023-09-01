// https://www.w3.org/TR/xpath-functions-31/#trigonometry
use ordered_float::OrderedFloat;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::wrap_xpath_fn;
use crate::Atomic;

#[xpath_fn("math:pi() as xs:double")]
fn pi() -> f64 {
    std::f64::consts::PI
}

#[xpath_fn("math:exp($arg as xs:double?) as xs:double?")]
fn exp(arg: Option<f64>) -> error::Result<Option<f64>> {
    if let Some(arg) = arg {
        Ok(Some(arg.exp()))
    } else {
        Ok(None)
    }
}

#[xpath_fn("math:exp10($arg as xs:double?) as xs:double?")]
fn exp10(arg: Option<f64>) -> error::Result<Option<f64>> {
    if let Some(arg) = arg {
        Ok(Some(10f64.powf(arg)))
    } else {
        Ok(None)
    }
}

#[xpath_fn("math:log($arg as xs:double?) as xs:double?")]
fn log(arg: Option<f64>) -> error::Result<Option<f64>> {
    if let Some(arg) = arg {
        Ok(Some(arg.ln()))
    } else {
        Ok(None)
    }
}

#[xpath_fn("math:log10($arg as xs:double?) as xs:double?")]
fn log10(arg: Option<f64>) -> error::Result<Option<f64>> {
    if let Some(arg) = arg {
        Ok(Some(arg.log10()))
    } else {
        Ok(None)
    }
}

#[xpath_fn("math:pow($x as xs:double?, $y as xs:numeric) as xs:double?")]
fn pow(x: Option<f64>, y: Atomic) -> error::Result<Option<f64>> {
    if let Some(x) = x {
        match y {
            Atomic::Integer(_, i) => {
                let i: i32 = i.as_ref().try_into()?;
                Ok(Some(x.powi(i)))
            }
            Atomic::Decimal(_) => {
                let f = Atomic::parse_atomic::<f64>(&y.into_canonical())?;
                let f = match f {
                    Atomic::Double(OrderedFloat(d)) => d,
                    _ => unreachable!(),
                };
                Ok(Some(x.powf(f)))
            }
            Atomic::Float(OrderedFloat(f)) => Ok(Some(x.powf(f as f64))),
            Atomic::Double(OrderedFloat(d)) => Ok(Some(x.powf(d))),
            _ => Err(error::Error::Type),
        }
    } else {
        Ok(None)
    }
}

#[xpath_fn("math:sqrt($arg as xs:double?) as xs:double?")]
fn sqrt(arg: Option<f64>) -> error::Result<Option<f64>> {
    if let Some(arg) = arg {
        Ok(Some(arg.sqrt()))
    } else {
        Ok(None)
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(pi),
        wrap_xpath_fn!(exp),
        wrap_xpath_fn!(exp10),
        wrap_xpath_fn!(log),
        wrap_xpath_fn!(log10),
        wrap_xpath_fn!(pow),
        wrap_xpath_fn!(sqrt),
    ]
}

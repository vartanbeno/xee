use std::cmp::Ordering;

use ibig::IBig;
// https://www.w3.org/TR/xpath-functions-31/#numeric-functions
use ibig::ops::Abs;
use num_traits::{Float, Zero};
use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::error;
use crate::wrap_xpath_fn;
use crate::Atomic;

#[xpath_fn("fn:abs($arg as xs:numeric?) as xs:numeric?")]
fn abs(arg: Option<Atomic>) -> error::Result<Option<Atomic>> {
    if let Some(arg) = arg {
        match arg {
            Atomic::Integer(_, i) => Ok(Some(i.as_ref().abs().into())),
            Atomic::Decimal(d) => Ok(Some(d.abs().into())),
            Atomic::Float(f) => Ok(Some(f.abs().into())),
            Atomic::Double(d) => Ok(Some(d.abs().into())),
            _ => Err(error::Error::Type),
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
            _ => Err(error::Error::Type),
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
            _ => Err(error::Error::Type),
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
        let precision: i32 = precision.try_into().map_err(|_| error::Error::XPDY0130)?;
        round_atomic(arg, precision).map(Some)
    } else {
        Ok(None)
    }
}

fn round_atomic(arg: Atomic, precision: i32) -> error::Result<Atomic> {
    match arg {
        Atomic::Integer(_, i) => {
            if precision < 0 {
                Ok(round_integer_negative(
                    i.as_ref().clone(),
                    precision.unsigned_abs(),
                ))
            } else {
                Ok(i.into())
            }
        }
        Atomic::Decimal(d) => round_decimal(*d, precision),
        // even though the spec claims we should cast to an infinite
        // precision decimal, we don't have such a thing, so we
        // make do with doing the operation directly on f32 and f64
        Atomic::Float(OrderedFloat(f)) => Ok(round_float(f, precision)),
        Atomic::Double(OrderedFloat(d)) => Ok(round_double(d, precision)),
        _ => Err(error::Error::Type),
    }
}

fn round_integer_negative(arg: IBig, precision: u32) -> Atomic {
    // TODO: this is definitely not the most optimized way to
    // implement this.

    // The qt3 test suite doesn't seem to cover
    // the integer case very well either, so I wrote a few more tests.
    let d = 10u32.pow(precision);
    let mut divided = arg.clone() / d;
    let remainder = arg.clone() % d;
    if remainder.abs() > (d / 2).into() {
        if arg < 0.into() {
            divided -= 1;
        } else {
            divided += 1;
        }
    }
    (divided * d).into()
}

fn round_decimal(arg: Decimal, precision: i32) -> error::Result<Atomic> {
    let rounding_strategy = if arg >= Decimal::from(0) {
        RoundingStrategy::MidpointAwayFromZero
    } else {
        RoundingStrategy::MidpointTowardZero
    };
    match precision.cmp(&0) {
        Ordering::Equal | Ordering::Greater => Ok(arg
            .round_dp_with_strategy(precision as u32, rounding_strategy)
            .into()),
        Ordering::Less => {
            let d: Decimal = 10u32.pow(precision.unsigned_abs()).into();
            let arg = arg / d;
            let arg = arg.round_dp_with_strategy(0, rounding_strategy);
            let arg = arg * d;
            Ok(arg.into())
        }
    }
}

fn round_float(arg: f32, precision: i32) -> Atomic {
    if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
        return arg.into();
    }

    match precision.cmp(&0) {
        Ordering::Equal => round_f32_ties_to_positive_infinity(arg).into(),
        Ordering::Greater => {
            let d = 10u32.pow(precision.unsigned_abs()) as f32;
            (round_f32_ties_to_positive_infinity(arg * d) / d).into()
        }
        Ordering::Less => {
            let d = 10u32.pow(precision.unsigned_abs()) as f32;
            (round_f32_ties_to_positive_infinity(arg / d) * d).into()
        }
    }
}

fn round_f32_ties_to_positive_infinity(x: f32) -> f32 {
    let y = x.floor();
    if x == y {
        x
    } else {
        let z = (2.0 * x - y).floor();
        z.copysign(x)
    }
}

fn round_double(arg: f64, precision: i32) -> Atomic {
    if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
        return arg.into();
    }
    match precision.cmp(&0) {
        Ordering::Equal => round_f64_ties_to_positive_infinity(arg).into(),
        Ordering::Greater => {
            let d = 10u32.pow(precision.unsigned_abs()) as f64;
            (round_f64_ties_to_positive_infinity(arg * d) / d).into()
        }
        Ordering::Less => {
            let d = 10u32.pow(precision.unsigned_abs()) as f64;
            (round_f64_ties_to_positive_infinity(arg / d) * d).into()
        }
    }
}

fn round_f64_ties_to_positive_infinity(x: f64) -> f64 {
    let y = x.floor();
    if x == y {
        x
    } else {
        let z = (2.0 * x - y).floor();
        z.copysign(x)
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
        wrap_xpath_fn!(number),
    ]
}

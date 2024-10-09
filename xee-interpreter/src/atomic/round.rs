use std::cmp::Ordering;

use ibig::{ops::Abs, IBig};
use ordered_float::OrderedFloat;
use rust_decimal::{Decimal, RoundingStrategy};

use crate::{atomic, error};

pub(crate) fn round_atomic(arg: atomic::Atomic, precision: i32) -> error::Result<atomic::Atomic> {
    match arg {
        atomic::Atomic::Integer(_, i) => round_integer(i, precision),
        atomic::Atomic::Decimal(d) => round_decimal(*d, precision),
        // even though the spec claims we should cast to an infinite
        // precision decimal, we don't have such a thing, so we
        // make do with doing the operation directly on f32 and f64
        atomic::Atomic::Float(OrderedFloat(f)) => Ok(round_float(f, precision)?.into()),
        atomic::Atomic::Double(OrderedFloat(d)) => Ok(round_float(d, precision)?.into()),
        _ => Err(error::Error::XPTY0004),
    }
}

fn round_integer(i: std::rc::Rc<IBig>, precision: i32) -> Result<atomic::Atomic, error::Error> {
    if precision < 0 {
        Ok(round_integer_negative(
            i.as_ref().clone(),
            precision.unsigned_abs(),
        ))
    } else {
        Ok(i.into())
    }
}

fn round_integer_negative(arg: IBig, precision: u32) -> atomic::Atomic {
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

fn round_decimal(arg: Decimal, precision: i32) -> error::Result<atomic::Atomic> {
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

pub(crate) fn round_float<F: num_traits::Float>(arg: F, precision: i32) -> error::Result<F> {
    if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
        return Ok(arg);
    }

    match precision.cmp(&0) {
        Ordering::Equal => Ok(round_float_ties_to_postive_infinity(arg)),
        Ordering::Greater => {
            let d = 10i32.pow(precision.unsigned_abs());
            let d = F::from(d);
            if let Some(d) = d {
                Ok(round_float_ties_to_postive_infinity(arg * d) / d)
            } else {
                Err(error::Error::FOAR0001)
            }
        }
        Ordering::Less => {
            let d = 10i32.pow(precision.unsigned_abs());
            let d = F::from(d);
            if let Some(d) = d {
                Ok(round_float_ties_to_postive_infinity(arg / d) * d)
            } else {
                Err(error::Error::FOAR0001)
            }
        }
    }
}

fn round_float_ties_to_postive_infinity<F: num_traits::Float>(x: F) -> F {
    let y = x.floor();
    if x == y {
        x
    } else {
        let z = ((x + x) - y).floor();
        z.copysign(x)
    }
}

// /// round a float to a given precision
// pub(crate) fn round_float(arg: f32, precision: i32) -> f32 {
//     if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
//         return arg;
//     }

//     match precision.cmp(&0) {
//         Ordering::Equal => round_f32_ties_to_positive_infinity(arg),
//         Ordering::Greater => {
//             let d = 10u32.pow(precision.unsigned_abs()) as f32;
//             round_f32_ties_to_positive_infinity(arg * d) / d
//         }
//         Ordering::Less => {
//             let d = 10u32.pow(precision.unsigned_abs()) as f32;
//             round_f32_ties_to_positive_infinity(arg / d) * d
//         }
//     }
// }

// fn round_f32_ties_to_positive_infinity(x: f32) -> f32 {
//     let y = x.floor();
//     if x == y {
//         x
//     } else {
//         let z = (2.0 * x - y).floor();
//         z.copysign(x)
//     }
// }

// pub(crate) fn round_double(arg: f64, precision: i32) -> f64 {
//     if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
//         return arg;
//     }
//     match precision.cmp(&0) {
//         Ordering::Equal => round_f64_ties_to_positive_infinity(arg),
//         Ordering::Greater => {
//             let d = 10u32.pow(precision.unsigned_abs()) as f64;
//             round_f64_ties_to_positive_infinity(arg * d) / d
//         }
//         Ordering::Less => {
//             let d = 10u32.pow(precision.unsigned_abs()) as f64;
//             round_f64_ties_to_positive_infinity(arg / d) * d
//         }
//     }
// }

// fn round_f64_ties_to_positive_infinity(x: f64) -> f64 {
//     let y = x.floor();
//     if x == y {
//         x
//     } else {
//         let z = (2.0 * x - y).floor();
//         z.copysign(x)
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_double_divide_huge() {
    //     let a: f64 = 12006.;
    //     let b: f64 = -1.7976e308;
    //     let result = round_double(a / b, 0);
    //     assert_eq!(result, 0.);
    // }
}

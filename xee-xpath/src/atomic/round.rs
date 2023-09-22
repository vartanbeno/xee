use std::cmp::Ordering;

use num_traits::Zero;

pub(crate) fn round_float(arg: f32, precision: i32) -> f32 {
    if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
        return arg;
    }

    match precision.cmp(&0) {
        Ordering::Equal => round_f32_ties_to_positive_infinity(arg),
        Ordering::Greater => {
            let d = 10u32.pow(precision.unsigned_abs()) as f32;
            round_f32_ties_to_positive_infinity(arg * d) / d
        }
        Ordering::Less => {
            let d = 10u32.pow(precision.unsigned_abs()) as f32;
            round_f32_ties_to_positive_infinity(arg / d) * d
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

pub(crate) fn round_double(arg: f64, precision: i32) -> f64 {
    if arg.is_nan() || arg.is_infinite() || arg.is_zero() {
        return arg;
    }
    match precision.cmp(&0) {
        Ordering::Equal => round_f64_ties_to_positive_infinity(arg),
        Ordering::Greater => {
            let d = 10u32.pow(precision.unsigned_abs()) as f64;
            round_f64_ties_to_positive_infinity(arg * d) / d
        }
        Ordering::Less => {
            let d = 10u32.pow(precision.unsigned_abs()) as f64;
            round_f64_ties_to_positive_infinity(arg / d) * d
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_divide_huge() {
        let a: f64 = 12006.;
        let b: f64 = -1.7976e308;
        let result = round_double(a / b, 0);
        assert_eq!(result, 0.);
    }
}

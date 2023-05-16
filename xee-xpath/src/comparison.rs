use ahash::{HashSet, HashSetExt};

use crate::op;
use crate::value::{Atomic, ValueError};

type Result<T> = std::result::Result<T, ValueError>;

pub(crate) fn value_eq(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::op_numeric_equal,
            string_op: op::op_string_equal,
            boolean_op: op::op_boolean_equal,
        },
    )
}
struct GenericComparisonOps<NumericOp, StringOp, BooleanOp>
where
    NumericOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
    StringOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
    BooleanOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
{
    numeric_op: NumericOp,
    string_op: StringOp,
    boolean_op: BooleanOp,
}

fn generic_value_compare<NumericOp, StringOp, BooleanOp>(
    a: &Atomic,
    b: &Atomic,
    ops: GenericComparisonOps<NumericOp, StringOp, BooleanOp>,
) -> Result<Atomic>
where
    NumericOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
    StringOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
    BooleanOp: FnOnce(&Atomic, &Atomic) -> Result<bool>,
{
    // If an atomized operand is an empty sequence, the result of the value
    // comparison is an empty sequence
    if matches!(a, Atomic::Empty) || matches!(b, Atomic::Empty) {
        return Ok(Atomic::Empty);
    }
    let r = match (a, b) {
        (
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_),
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_),
        ) => (ops.numeric_op)(a, b),
        (Atomic::String(_), Atomic::String(_)) => (ops.string_op)(a, b),
        (Atomic::Boolean(_), Atomic::Boolean(_)) => (ops.boolean_op)(a, b),
        _ => Err(ValueError::Type),
    }?;
    Ok(Atomic::Boolean(r))
}

// generalized comparison, optimized eq version here we avoid O(n * m)
// complexity by turning the shortest sequence into a hash set
fn general_compare_eq(a: &[Atomic], b: &[Atomic]) -> bool {
    // index the shortest sequence
    if a.len() > b.len() {
        return general_compare_eq(b, a);
    }
    // a should be the shortest sequence, turn into a hash set
    let a: HashSet<_> = a.iter().collect();
    // now look whether we find a match for any item in b
    b.iter().any(|item| a.contains(item))
}

fn general_compare_ne(a: &[Atomic], b: &[Atomic]) -> bool {
    !general_compare_eq(a, b)
}

// fn general_compare_lt(a: &[Atomic], b: &[Atomic]) -> bool {
//     // if any item in a is lt any item in b, then a < b
//     a.iter().any(|a| b.iter().any(|b| a < b))
// }

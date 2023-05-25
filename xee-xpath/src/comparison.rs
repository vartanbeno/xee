use crate::op;
use crate::value::{Atomic, ValueError};

type Result<T> = std::result::Result<T, ValueError>;

pub(crate) fn value_eq(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_equal,
            string_op: op::string_equal,
            boolean_op: op::boolean_equal,
        },
    )
}

pub(crate) fn value_ne(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_not_equal,
            string_op: op::string_not_equal,
            boolean_op: op::boolean_not_equal,
        },
    )
}

pub(crate) fn value_lt(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_less_than,
            string_op: op::string_less_than,
            boolean_op: op::boolean_less_than,
        },
    )
}

pub(crate) fn value_le(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_less_than_or_equal,
            string_op: op::string_less_than_or_equal,
            boolean_op: op::boolean_less_than_or_equal,
        },
    )
}

pub(crate) fn value_gt(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_greater_than,
            string_op: op::string_greater_than,
            boolean_op: op::boolean_greater_than,
        },
    )
}

pub(crate) fn value_ge(a: &Atomic, b: &Atomic) -> Result<Atomic> {
    generic_value_compare(
        a,
        b,
        GenericComparisonOps {
            numeric_op: op::numeric_greater_than_or_equal,
            string_op: op::string_greater_than_or_equal,
            boolean_op: op::boolean_greater_than_or_equal,
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
    let (a, b) = cast_untyped(a, b)?;
    let r = match (&a, &b) {
        (
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_),
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_),
        ) => (ops.numeric_op)(&a, &b),
        (Atomic::String(_), Atomic::String(_)) => (ops.string_op)(&a, &b),
        (Atomic::Boolean(_), Atomic::Boolean(_)) => (ops.boolean_op)(&a, &b),
        _ => Err(ValueError::Type),
    }?;
    Ok(Atomic::Boolean(r))
}

fn cast_untyped(a: &Atomic, b: &Atomic) -> Result<(Atomic, Atomic)> {
    let r = match (a, b) {
        // If both atomic values are instances of xs:untypedAtomic, then the
        // values are cast to the type xs:string.
        (Atomic::Untyped(a), Atomic::Untyped(b)) => {
            (Atomic::String(a.clone()), Atomic::String(b.clone()))
        }
        // If exactly one of the atomic values is an instance of
        // xs:untypedAtomic, it is cast to a type depending on the other
        // value's dynamic type T according to the following rules, in which V
        // denotes the value to be cast:
        (Atomic::Untyped(a), _) => {
            let a = b.general_comparison_cast(a)?;
            (a, b.clone())
        }
        (_, Atomic::Untyped(b)) => {
            let b = a.general_comparison_cast(b)?;
            (a.clone(), b)
        }
        _ => (a.clone(), b.clone()),
    };
    Ok(r)
}

pub(crate) fn general_eq(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_eq)
}

pub(crate) fn general_ne(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ne)
}

pub(crate) fn general_lt(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_lt)
}

pub(crate) fn general_le(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_le)
}

pub(crate) fn general_gt(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_gt)
}

pub(crate) fn general_ge(a_atoms: &[Atomic], b_atoms: &[Atomic]) -> Result<Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ge)
}

fn generic_general_compare<F>(a_atoms: &[Atomic], b_atoms: &[Atomic], compare: F) -> Result<Atomic>
where
    F: Fn(&Atomic, &Atomic) -> Result<Atomic>,
{
    for a in a_atoms {
        for b in b_atoms {
            if compare(a, b)?.to_bool()? {
                return Ok(Atomic::Boolean(true));
            }
        }
    }
    Ok(Atomic::Boolean(false))
}

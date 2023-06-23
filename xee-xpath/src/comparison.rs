use crate::atomic;
use crate::error;
use crate::op;

pub(crate) fn value_eq(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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

pub(crate) fn value_ne(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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

pub(crate) fn value_lt(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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

pub(crate) fn value_le(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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

pub(crate) fn value_gt(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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

pub(crate) fn value_ge(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<atomic::Atomic> {
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
    NumericOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
    StringOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
    BooleanOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
{
    numeric_op: NumericOp,
    string_op: StringOp,
    boolean_op: BooleanOp,
}

fn generic_value_compare<NumericOp, StringOp, BooleanOp>(
    a: &atomic::Atomic,
    b: &atomic::Atomic,
    ops: GenericComparisonOps<NumericOp, StringOp, BooleanOp>,
) -> error::Result<atomic::Atomic>
where
    NumericOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
    StringOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
    BooleanOp: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<bool>,
{
    let (a, b) = cast_untyped(a, b)?;
    let r = match (&a, &b) {
        (
            atomic::Atomic::Integer(_)
            | atomic::Atomic::Decimal(_)
            | atomic::Atomic::Float(_)
            | atomic::Atomic::Double(_),
            atomic::Atomic::Integer(_)
            | atomic::Atomic::Decimal(_)
            | atomic::Atomic::Float(_)
            | atomic::Atomic::Double(_),
        ) => (ops.numeric_op)(&a, &b),
        (atomic::Atomic::String(_), atomic::Atomic::String(_)) => (ops.string_op)(&a, &b),
        (atomic::Atomic::Boolean(_), atomic::Atomic::Boolean(_)) => (ops.boolean_op)(&a, &b),
        _ => Err(error::Error::Type),
    }?;
    Ok(atomic::Atomic::Boolean(r))
}

fn cast_untyped(
    a: &atomic::Atomic,
    b: &atomic::Atomic,
) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    let r = match (a, b) {
        // If both atomic values are instances of xs:untypedAtomic, then the
        // values are cast to the type xs:string.
        (atomic::Atomic::Untyped(a), atomic::Atomic::Untyped(b)) => (
            atomic::Atomic::String(a.clone()),
            atomic::Atomic::String(b.clone()),
        ),
        // If exactly one of the atomic values is an instance of
        // xs:untypedAtomic, it is cast to a type depending on the other
        // value's dynamic type T according to the following rules, in which V
        // denotes the value to be cast:
        (atomic::Atomic::Untyped(a), _) => {
            let a = b.general_comparison_cast(a)?;
            (a, b.clone())
        }
        (_, atomic::Atomic::Untyped(b)) => {
            let b = a.general_comparison_cast(b)?;
            (a.clone(), b)
        }
        _ => (a.clone(), b.clone()),
    };
    Ok(r)
}

pub(crate) fn general_eq(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_eq)
}

pub(crate) fn general_ne(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ne)
}

pub(crate) fn general_lt(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_lt)
}

pub(crate) fn general_le(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_le)
}

pub(crate) fn general_gt(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_gt)
}

pub(crate) fn general_ge(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<atomic::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ge)
}

fn generic_general_compare<F>(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
    compare: F,
) -> error::Result<atomic::Atomic>
where
    F: Fn(&atomic::Atomic, &atomic::Atomic) -> error::Result<atomic::Atomic>,
{
    for a in a_atoms {
        let a = a?;
        for b in b_atoms.clone() {
            if compare(&a, &(b?))?.is_true() {
                return Ok(atomic::Atomic::Boolean(true));
            }
        }
    }
    Ok(atomic::Atomic::Boolean(false))
}

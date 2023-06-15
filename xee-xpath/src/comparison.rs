use crate::op;
use crate::stack;

pub(crate) fn value_eq(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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

pub(crate) fn value_ne(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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

pub(crate) fn value_lt(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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

pub(crate) fn value_le(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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

pub(crate) fn value_gt(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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

pub(crate) fn value_ge(a: &stack::Atomic, b: &stack::Atomic) -> stack::Result<stack::Atomic> {
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
    NumericOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
    StringOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
    BooleanOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
{
    numeric_op: NumericOp,
    string_op: StringOp,
    boolean_op: BooleanOp,
}

fn generic_value_compare<NumericOp, StringOp, BooleanOp>(
    a: &stack::Atomic,
    b: &stack::Atomic,
    ops: GenericComparisonOps<NumericOp, StringOp, BooleanOp>,
) -> stack::Result<stack::Atomic>
where
    NumericOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
    StringOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
    BooleanOp: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<bool>,
{
    // If an atomized operand is an empty sequence, the result of the value
    // comparison is an empty sequence
    if matches!(a, stack::Atomic::Empty) || matches!(b, stack::Atomic::Empty) {
        return Ok(stack::Atomic::Empty);
    }
    let (a, b) = cast_untyped(a, b)?;
    let r = match (&a, &b) {
        (
            stack::Atomic::Integer(_)
            | stack::Atomic::Decimal(_)
            | stack::Atomic::Float(_)
            | stack::Atomic::Double(_),
            stack::Atomic::Integer(_)
            | stack::Atomic::Decimal(_)
            | stack::Atomic::Float(_)
            | stack::Atomic::Double(_),
        ) => (ops.numeric_op)(&a, &b),
        (stack::Atomic::String(_), stack::Atomic::String(_)) => (ops.string_op)(&a, &b),
        (stack::Atomic::Boolean(_), stack::Atomic::Boolean(_)) => (ops.boolean_op)(&a, &b),
        _ => Err(stack::Error::Type),
    }?;
    Ok(stack::Atomic::Boolean(r))
}

fn cast_untyped(
    a: &stack::Atomic,
    b: &stack::Atomic,
) -> stack::Result<(stack::Atomic, stack::Atomic)> {
    let r = match (a, b) {
        // If both atomic values are instances of xs:untypedAtomic, then the
        // values are cast to the type xs:string.
        (stack::Atomic::Untyped(a), stack::Atomic::Untyped(b)) => (
            stack::Atomic::String(a.clone()),
            stack::Atomic::String(b.clone()),
        ),
        // If exactly one of the atomic values is an instance of
        // xs:untypedAtomic, it is cast to a type depending on the other
        // value's dynamic type T according to the following rules, in which V
        // denotes the value to be cast:
        (stack::Atomic::Untyped(a), _) => {
            let a = b.general_comparison_cast(a)?;
            (a, b.clone())
        }
        (_, stack::Atomic::Untyped(b)) => {
            let b = a.general_comparison_cast(b)?;
            (a.clone(), b)
        }
        _ => (a.clone(), b.clone()),
    };
    Ok(r)
}

pub(crate) fn general_eq(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_eq)
}

pub(crate) fn general_ne(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ne)
}

pub(crate) fn general_lt(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_lt)
}

pub(crate) fn general_le(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_le)
}

pub(crate) fn general_gt(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_gt)
}

pub(crate) fn general_ge(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
) -> stack::Result<stack::Atomic> {
    generic_general_compare(a_atoms, b_atoms, value_ge)
}

fn generic_general_compare<F>(
    a_atoms: &[stack::Atomic],
    b_atoms: &[stack::Atomic],
    compare: F,
) -> stack::Result<stack::Atomic>
where
    F: Fn(&stack::Atomic, &stack::Atomic) -> stack::Result<stack::Atomic>,
{
    for a in a_atoms {
        for b in b_atoms {
            if compare(a, b)?.is_true() {
                return Ok(stack::Atomic::Boolean(true));
            }
        }
    }
    Ok(stack::Atomic::Boolean(false))
}

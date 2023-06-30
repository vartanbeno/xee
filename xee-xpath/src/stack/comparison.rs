use crate::atomic;
use crate::error;

// https://www.w3.org/TR/xpath-31/#id-general-comparisons
// step 1, atomization, has already taken place
pub(crate) fn general_comparison<O>(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
) -> error::Result<bool>
where
    O: atomic::ComparisonOp,
{
    let b_atoms = b_atoms.collect::<Vec<_>>();
    for a in a_atoms {
        let a = a?;
        for b in b_atoms.iter() {
            let b = b.as_ref().map_err(|e| e.clone())?;
            let (a, b) = cast(&a, b)?;
            // 2c do value comparison
            if a.value_comparison::<O>(b)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

// step 2: cast
fn cast(a: &atomic::Atomic, b: &atomic::Atomic) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    Ok(match (a, b) {
        // step 2a: if both are untyped atomic, cast them both to string
        (atomic::Atomic::Untyped(_), atomic::Atomic::Untyped(_)) => {
            let a = a.cast_to_string();
            let b = b.cast_to_string();
            (a, b)
        }
        // step 2bi: if untyped is combined with numeric, cast to double
        (atomic::Atomic::Untyped(_), _) => {
            let a = if b.is_numeric() {
                a.cast_to_double()?
            } else {
                // step 2biv: in all other cases, cast untyped to primitive base type of other
                a.cast_to_schema_type_of(b)?
            };
            (a, b.clone())
        }
        (_, atomic::Atomic::Untyped(_)) => {
            let b = if a.is_numeric() {
                b.cast_to_double()?
            } else {
                // step 2biv: in all other cases, cast untyped to primitive base type of other
                b.cast_to_schema_type_of(a)?
            };
            (a.clone(), b)
        } // step 2bii & 2biii skipped until we have datetime stuff
        _ => (a.clone(), b.clone()),
    })
}

use crate::atomic;
use crate::error;

fn general_comparison_op<F>(
    a_atoms: impl Iterator<Item = error::Result<atomic::Atomic>>,
    b_atoms: impl Iterator<Item = error::Result<atomic::Atomic>> + std::clone::Clone,
    compare: F,
) -> error::Result<atomic::Atomic>
where
    F: Fn(atomic::Atomic, atomic::Atomic) -> error::Result<atomic::Atomic>,
{
    let b_atoms = b_atoms.collect::<Vec<_>>();
    for a in a_atoms {
        let a = a?;
        for b in &b_atoms {
            let b = b.clone()?;
            if compare(a.clone(), b)?.is_true() {
                return Ok(atomic::Atomic::Boolean(true));
            }
        }
    }
    Ok(atomic::Atomic::Boolean(false))
}

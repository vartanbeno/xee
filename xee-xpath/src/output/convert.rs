use crate::context::{ContextFrom, ContextTryFrom, ContextTryInto, DynamicContext};
use crate::error;
use crate::output;

// Conversions from Sequence

impl ContextTryFrom<&output::Sequence> for output::Atomic {
    type Error = error::Error;

    fn context_try_from(
        sequence: &output::Sequence,
        _context: &DynamicContext,
    ) -> error::Result<output::Atomic> {
        // TODO: this is incorrect: we want the first atomized value
        sequence.one()?.to_atomic()
    }
}

// TODO iterator of atomic?
// impl ContextTryFrom<&output::Sequence> for Vec<output::Atomic> {
//     type Error = error::Error;

//     fn context_try_from(
//         sequence: &output::Sequence,
//         _context: &DynamicContext,
//     ) -> error::Result<Vec<output::Atomic>> {
//         sequence.as_slice()
//     }
// }

impl ContextTryFrom<output::Sequence> for output::Atomic {
    type Error = error::Error;

    fn context_try_from(
        sequence: output::Sequence,
        context: &DynamicContext,
    ) -> error::Result<output::Atomic> {
        ContextTryFrom::context_try_from(&sequence, context)
    }
}

use crate::context::{ContextFrom, ContextTryFrom, ContextTryInto, DynamicContext};
use crate::error;
use crate::output;

// to atomic

impl<'a> ContextTryFrom<'a, &output::Sequence> for output::Atomic {
    type Error = error::Error;

    fn context_try_from(
        sequence: &output::Sequence,
        context: &DynamicContext<'a>,
    ) -> error::Result<Self> {
        sequence.one_atom(context.xot)
    }
}

// impl<'a> ContextFrom<'a, &'a output::Sequence> for output::AtomizedIter<'a> {
//     fn context_from(
//         sequence: &'a output::Sequence,
//         context: &DynamicContext<'a>,
//     ) -> output::AtomizedIter<'a> {
//         sequence.atomized(context.xot)
//     }
// }

impl<'a> ContextTryFrom<'a, &output::Sequence> for Option<output::Atomic> {
    type Error = error::Error;

    fn context_try_from(
        sequence: &output::Sequence,
        context: &DynamicContext<'a>,
    ) -> error::Result<Self> {
        sequence.option_atom(context.xot)
    }
}

impl<'a> ContextTryFrom<'a, &output::Sequence> for bool {
    type Error = error::Error;

    fn context_try_from(
        sequence: &output::Sequence,
        context: &DynamicContext<'a>,
    ) -> error::Result<Self> {
        sequence.one_generalized_atomic(context.xot, |atomic| atomic.to_bool())
    }
}

// impl<'a, T> ContextFrom<'a, &output::Sequence> for T
// where
//     T: Iterator<Item = bool>,
// {
//     fn context_from(sequence: &output::Sequence, context: &DynamicContext<'a>) -> Self {
//         sequence.generalized_atomic(context.xot, |atomic| atomic.to.bool().ok)
//     }
// }

// impl<'a, T> ContextFrom<'a, &output::Sequence> for T
// where
//     T: Iterator<Item = bool>,
// {
//     fn context_from(sequence: &output::Sequence, context: &DynamicContext<'a>) -> Self {
//         sequence.generalized_atomic(context.xot, |atomic| atomic.to.bool().ok)
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use xee_xpath_ast::Namespaces;
    use xot::Xot;

    use crate::context::{ContextInto, DynamicContext, StaticContext};

    #[test]
    fn test_many_atomics() {
        let xot = Xot::new();
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let sequence = output::Sequence::from_items(&[
            output::Item::from_atomic(output::Atomic::from(true)),
            output::Item::from_atomic(output::Atomic::from(false)),
            output::Item::from_atomic(output::Atomic::from(true)),
        ]);
        // let atomics: Iterator<Item = Atomic> = (&sequence).context_into(&context);
        // assert_eq!(bools, vec![true, false, true]);
    }
    // #[test]
    // fn test_many_bools() {
    //     let mut xot = Xot::new();
    //     let namespaces = Namespaces::default();
    //     let static_context = StaticContext::new(&namespaces);
    //     let context = DynamicContext::new(&mut xot, &static_context);
    //     let sequence = output::Sequence::from_items(&[
    //         output::Item::from_atomic(output::Atomic::from(true)),
    //         output::Item::from_atomic(output::Atomic::from(false)),
    //         output::Item::from_atomic(output::Atomic::from(true)),
    //     ]);
    //     let bools: GeneralizedAtomicIterator<bool> = sequence.context_into(&context).collect();
    //     assert_eq!(bools, vec![true, false, true]);
    // }
}

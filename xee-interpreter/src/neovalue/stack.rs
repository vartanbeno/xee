// this is unfortunately a ridiculous verbose module, wiring everything up
// carefully so we don't use performance due to dynamic dispatch on the inside. I
// hope it's worth it.

// enum_dispatch could be used to simplify it, but that seems to require co-location
// of the trait and implementation in the same module and it's less clear what's
// going on in detail, so I don't use it.

// Note that creation.rs contains various functions that create StackSequences,
// as they only make sense on that level.

use crate::{
    atomic::{self, AtomicCompare},
    context, error, function,
    sequence::Item,
};

use super::{
    core::{Empty, Many, One},
    traits::{BoxedItemIter, Sequence, SequenceCompare, SequenceExt, SequenceOrder},
};

// The name "StackSequence" is a bit obscure but it's the sequence that goes
// onto the interpreter stack. The sequence that holds all the other sequence
// implementations.
// It's the size of an single item, as that's the biggest thing in it.
#[derive(Debug, Clone)]
pub enum StackSequence {
    Empty(Empty),
    One(One),
    Many(Many),
}

impl<'a> Sequence<'a, Box<dyn Iterator<Item = &'a Item> + 'a>> for StackSequence {
    fn is_empty(&self) -> bool {
        match self {
            StackSequence::Empty(inner) => inner.is_empty(),
            StackSequence::One(inner) => inner.is_empty(),
            StackSequence::Many(inner) => inner.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            StackSequence::Empty(inner) => inner.len(),
            StackSequence::One(inner) => inner.len(),
            StackSequence::Many(inner) => inner.len(),
        }
    }

    fn get(&self, index: usize) -> Option<&Item> {
        match self {
            StackSequence::Empty(inner) => inner.get(index),
            StackSequence::One(inner) => inner.get(index),
            StackSequence::Many(inner) => inner.get(index),
        }
    }

    fn items(&'a self) -> Box<dyn Iterator<Item = &'a Item> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.items()),
            StackSequence::One(inner) => Box::new(inner.items()),
            StackSequence::Many(inner) => Box::new(inner.items()),
        }
    }

    fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            StackSequence::Empty(inner) => inner.effective_boolean_value(),
            StackSequence::One(inner) => inner.effective_boolean_value(),
            StackSequence::Many(inner) => inner.effective_boolean_value(),
        }
    }

    fn string_value(&self, xot: &xot::Xot) -> error::Result<String> {
        match self {
            StackSequence::Empty(inner) => inner.string_value(xot),
            StackSequence::One(inner) => inner.string_value(xot),
            StackSequence::Many(inner) => inner.string_value(xot),
        }
    }
}

// we implement these explicitly, because we want to avoid dynamic dispatch until
// the outer layer. This gives the compiler the chance to optimize the inner
// layers better.
impl<'a> SequenceExt<'a, BoxedItemIter<'a>> for StackSequence
where
    StackSequence: Sequence<'a, Box<dyn Iterator<Item = &'a Item>>>,
{
    #[allow(refining_impl_trait)]
    fn nodes(&'a self) -> Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.nodes()),
            StackSequence::One(inner) => Box::new(inner.nodes()),
            StackSequence::Many(inner) => Box::new(inner.nodes()),
        }
    }

    #[allow(refining_impl_trait)]
    fn atomized(
        &'a self,
        xot: &'a xot::Xot,
    ) -> Box<dyn Iterator<Item = error::Result<atomic::Atomic>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.atomized(xot)),
            StackSequence::One(inner) => Box::new(inner.atomized(xot)),
            StackSequence::Many(inner) => Box::new(inner.atomized(xot)),
        }
    }

    #[allow(refining_impl_trait)]
    fn map_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Map>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.map_iter()),
            StackSequence::One(inner) => Box::new(inner.map_iter()),
            StackSequence::Many(inner) => Box::new(inner.map_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn array_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Array>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.array_iter()),
            StackSequence::One(inner) => Box::new(inner.array_iter()),
            StackSequence::Many(inner) => Box::new(inner.array_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn elements(
        &'a self,
        xot: &'a xot::Xot,
    ) -> error::Result<Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a>> {
        match self {
            StackSequence::Empty(inner) => Ok(Box::new(inner.elements(xot)?)),
            StackSequence::One(inner) => Ok(Box::new(inner.elements(xot)?)),
            StackSequence::Many(inner) => Ok(Box::new(inner.elements(xot)?)),
        }
    }

    #[allow(refining_impl_trait)]
    fn to_array(&'a self) -> error::Result<function::Array> {
        match self {
            StackSequence::Empty(inner) => inner.to_array(),
            StackSequence::One(inner) => inner.to_array(),
            StackSequence::Many(inner) => inner.to_array(),
        }
    }
}

impl<'a> SequenceCompare<'a, BoxedItemIter<'a>> for StackSequence
where
    StackSequence: Sequence<'a, BoxedItemIter<'a>>,
{
    #[allow(refining_impl_trait)]
    fn general_comparison<O>(
        &'a self,
        other: &'a impl SequenceExt<'a, BoxedItemIter<'a>>,
        context: &context::DynamicContext,
        xot: &'a xot::Xot,
        op: O,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
    {
        match self {
            // this will specialize over inner as we know the exact type.
            // otherw will have to be a boxed trait object, but that's fine
            StackSequence::Empty(inner) => inner.general_comparison(other, context, xot, op),
            StackSequence::One(inner) => inner.general_comparison(other, context, xot, op),
            StackSequence::Many(inner) => inner.general_comparison(other, context, xot, op),
        }
    }
}

impl<'a> SequenceOrder<'a, BoxedItemIter<'a>> for StackSequence
where
    StackSequence: Sequence<'a, BoxedItemIter<'a>>,
{
    // only one_node can benefit from specialization

    #[allow(refining_impl_trait)]
    fn one_node(&'a self) -> error::Result<xot::Node> {
        match self {
            StackSequence::Empty(inner) => inner.one_node(),
            StackSequence::One(inner) => inner.one_node(),
            StackSequence::Many(inner) => inner.one_node(),
        }
    }
}

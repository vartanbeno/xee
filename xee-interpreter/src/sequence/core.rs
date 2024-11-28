// this is unfortunately a ridiculous verbose module, wiring everything up
// carefully so we don't use performance due to dynamic dispatch on the inside. I
// hope it's worth it. The verbosity is all pretty straightforward though.

// enum_dispatch could be used to simplify it, but that seems to require co-location
// of the trait and implementation in the same module and it's less clear what's
// going on in detail, so I don't use it.

// creation.rs contains various functions that create Sequence
// compare.rs contains various comparison functions

use crate::{
    atomic::{self, AtomicCompare},
    context, error, function,
};

use super::{
    item::Item,
    traits::{BoxedItemIter, SequenceCompare, SequenceCore, SequenceExt, SequenceOrder},
    variant::{Empty, Many, One, Range},
};

// The Sequence that goes onto the stack is the size of an single item, as
// that's the biggest thing in it.
#[derive(Debug, Clone, PartialEq)]
pub enum Sequence {
    Empty(Empty),
    One(One),
    Many(Many),
    Range(Range),
}

impl From<Range> for Sequence {
    fn from(inner: Range) -> Self {
        Self::Range(inner)
    }
}

// a static assertion to ensure that Sequence never grows in size
#[cfg(target_arch = "x86_64")]
static_assertions::assert_eq_size!(Sequence, [u8; 24]);

impl Default for Sequence {
    fn default() -> Self {
        Self::Empty(Empty {})
    }
}

impl<'a> SequenceCore<'a, BoxedItemIter<'a>> for Sequence {
    fn is_empty(&self) -> bool {
        match self {
            Sequence::Empty(inner) => inner.is_empty(),
            Sequence::One(inner) => inner.is_empty(),
            Sequence::Many(inner) => inner.is_empty(),
            Sequence::Range(inner) => inner.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Sequence::Empty(inner) => inner.len(),
            Sequence::One(inner) => inner.len(),
            Sequence::Many(inner) => inner.len(),
            Sequence::Range(inner) => inner.len(),
        }
    }

    fn get(&self, index: usize) -> Option<Item> {
        match self {
            Sequence::Empty(inner) => inner.get(index),
            Sequence::One(inner) => inner.get(index),
            Sequence::Many(inner) => inner.get(index),
            Sequence::Range(inner) => inner.get(index),
        }
    }

    fn one(self) -> error::Result<Item> {
        match self {
            Sequence::Empty(inner) => inner.one(),
            Sequence::One(inner) => inner.one(),
            Sequence::Many(inner) => inner.one(),
            Sequence::Range(inner) => inner.one(),
        }
    }

    fn option(self) -> error::Result<Option<Item>> {
        match self {
            Sequence::Empty(inner) => inner.option(),
            Sequence::One(inner) => inner.option(),
            Sequence::Many(inner) => inner.option(),
            Sequence::Range(inner) => inner.option(),
        }
    }

    fn iter(&'a self) -> BoxedItemIter<'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.iter()),
            Sequence::One(inner) => Box::new(inner.iter()),
            Sequence::Many(inner) => Box::new(inner.iter()),
            Sequence::Range(inner) => Box::new(inner.iter()),
        }
    }

    fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Sequence::Empty(inner) => inner.effective_boolean_value(),
            Sequence::One(inner) => inner.effective_boolean_value(),
            Sequence::Many(inner) => inner.effective_boolean_value(),
            Sequence::Range(inner) => inner.effective_boolean_value(),
        }
    }

    fn string_value(&self, xot: &xot::Xot) -> error::Result<String> {
        match self {
            Sequence::Empty(inner) => inner.string_value(xot),
            Sequence::One(inner) => inner.string_value(xot),
            Sequence::Many(inner) => inner.string_value(xot),
            Sequence::Range(inner) => inner.string_value(xot),
        }
    }
}

// we implement these explicitly, because we want to avoid dynamic dispatch until
// the outer layer. This gives the compiler the chance to optimize the inner
// layers better.
impl<'a> SequenceExt<'a, BoxedItemIter<'a>> for Sequence
where
    Sequence: SequenceCore<'a, BoxedItemIter<'a>>,
{
    #[allow(refining_impl_trait)]
    fn nodes(&'a self) -> Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.nodes()),
            Sequence::One(inner) => Box::new(inner.nodes()),
            Sequence::Many(inner) => Box::new(inner.nodes()),
            Sequence::Range(inner) => Box::new(inner.nodes()),
        }
    }

    #[allow(refining_impl_trait)]
    fn atomized(
        &'a self,
        xot: &'a xot::Xot,
    ) -> Box<dyn Iterator<Item = error::Result<atomic::Atomic>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.atomized(xot)),
            Sequence::One(inner) => Box::new(inner.atomized(xot)),
            Sequence::Many(inner) => Box::new(inner.atomized(xot)),
            Sequence::Range(inner) => Box::new(inner.atomized(xot)),
        }
    }

    #[allow(refining_impl_trait)]
    fn unboxed_atomized<T: 'a>(
        &'a self,
        xot: &'a xot::Xot,
        extract: impl Fn(atomic::Atomic) -> error::Result<T> + 'a,
    ) -> Box<dyn Iterator<Item = error::Result<T>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.unboxed_atomized(xot, extract)),
            Sequence::One(inner) => Box::new(inner.unboxed_atomized(xot, extract)),
            Sequence::Many(inner) => Box::new(inner.unboxed_atomized(xot, extract)),
            Sequence::Range(inner) => Box::new(inner.unboxed_atomized(xot, extract)),
        }
    }

    #[allow(refining_impl_trait)]
    fn map_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Map>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.map_iter()),
            Sequence::One(inner) => Box::new(inner.map_iter()),
            Sequence::Many(inner) => Box::new(inner.map_iter()),
            Sequence::Range(inner) => Box::new(inner.map_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn array_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Array>> + 'a> {
        match self {
            Sequence::Empty(inner) => Box::new(inner.array_iter()),
            Sequence::One(inner) => Box::new(inner.array_iter()),
            Sequence::Many(inner) => Box::new(inner.array_iter()),
            Sequence::Range(inner) => Box::new(inner.array_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn elements(
        &'a self,
        xot: &'a xot::Xot,
    ) -> error::Result<Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a>> {
        match self {
            Sequence::Empty(inner) => Ok(Box::new(inner.elements(xot)?)),
            Sequence::One(inner) => Ok(Box::new(inner.elements(xot)?)),
            Sequence::Many(inner) => Ok(Box::new(inner.elements(xot)?)),
            Sequence::Range(inner) => Ok(Box::new(inner.elements(xot)?)),
        }
    }

    #[allow(refining_impl_trait)]
    fn to_array(&'a self) -> error::Result<function::Array> {
        match self {
            Sequence::Empty(inner) => inner.to_array(),
            Sequence::One(inner) => inner.to_array(),
            Sequence::Many(inner) => inner.to_array(),
            Sequence::Range(inner) => inner.to_array(),
        }
    }
}

impl<'a> SequenceCompare<'a, BoxedItemIter<'a>> for Sequence
where
    Sequence: SequenceCore<'a, BoxedItemIter<'a>>,
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
            Sequence::Empty(inner) => inner.general_comparison(other, context, xot, op),
            Sequence::One(inner) => inner.general_comparison(other, context, xot, op),
            Sequence::Many(inner) => inner.general_comparison(other, context, xot, op),
            Sequence::Range(inner) => inner.general_comparison(other, context, xot, op),
        }
    }
}

impl<'a> SequenceOrder<'a, BoxedItemIter<'a>> for Sequence
where
    Sequence: SequenceCore<'a, BoxedItemIter<'a>>,
{
    // only one_node can benefit from specialization

    #[allow(refining_impl_trait)]
    fn one_node(&'a self) -> error::Result<xot::Node> {
        match self {
            Sequence::Empty(inner) => inner.one_node(),
            Sequence::One(inner) => inner.one_node(),
            Sequence::Many(inner) => inner.one_node(),
            Sequence::Range(inner) => inner.one_node(),
        }
    }
}

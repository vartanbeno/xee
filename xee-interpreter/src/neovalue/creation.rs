use std::rc::Rc;

use ahash::{HashSet, HashSetExt};
use xot::Xot;

use crate::{atomic, context, error, sequence::Item, string::Collation, xml};

use super::{
    core::Sequence,
    normalization::normalize,
    serialization::{serialize_sequence, SerializationParameters},
    traits::{SequenceCore, SequenceExt},
    variant::Empty,
};

impl Sequence {
    /// Concatenate two sequences producing a new sequence.
    pub fn concat(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty(_), Self::Empty(_)) => Self::Empty(Empty {}),
            (Self::Empty(_), Self::One(item)) => Self::One(item),
            (Self::One(item), Self::Empty(_)) => Self::One(item),
            (Self::Empty(_), Self::Many(items)) => Self::Many(items),
            (Self::Many(items), Self::Empty(_)) => Self::Many(items),
            (Self::One(item1), Self::One(item2)) => {
                Self::Many((vec![item1.into_item(), item2.into_item()]).into())
            }
            (Self::One(item), Self::Many(items)) => {
                let mut many = Vec::with_capacity(items.len() + 1);
                many.push(item.into_item());
                for item in items.iter() {
                    many.push(item.clone());
                }
                Self::Many(many.into())
            }
            (Self::Many(items), Self::One(item)) => {
                let mut many = Vec::with_capacity(items.len() + 1);
                for item in items.iter() {
                    many.push(item.clone());
                }
                many.push(item.into_item());
                Self::Many(many.into())
            }
            (Self::Many(items1), Self::Many(items2)) => {
                let mut many = Vec::with_capacity(items1.len() + items2.len());
                for item in items1.iter() {
                    many.push(item.clone());
                }
                for item in items2.iter() {
                    many.push(item.clone());
                }
                Self::Many(many.into())
            }
        }
    }
    // https://www.w3.org/TR/xpath-31/#id-path-operator
    pub(crate) fn deduplicate(self, annotations: &xml::Annotations) -> error::Result<Self> {
        let mut s = HashSet::new();
        let mut non_node_seen = false;

        for item in self.iter() {
            match item {
                Item::Node(n) => {
                    if non_node_seen {
                        return Err(error::Error::XPTY0004);
                    }
                    s.insert(*n);
                }
                _ => {
                    if !s.is_empty() {
                        return Err(error::Error::XPTY0004);
                    }
                    non_node_seen = true;
                }
            }
        }
        if non_node_seen {
            Ok(self)
        } else {
            Ok(Self::process_set_result(s, annotations))
        }
    }

    pub(crate) fn process_set_result(
        s: HashSet<xot::Node>,
        annotations: &xml::Annotations,
    ) -> Self {
        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));
        nodes.into()
    }

    pub fn sorted(
        &self,
        context: &context::DynamicContext,
        collation: Rc<Collation>,
        xot: &Xot,
    ) -> error::Result<Self> {
        self.sorted_by_key(context, collation, |item| {
            // the equivalent of fn:data()
            let seq: Self = item.clone().into();
            seq.atomized(xot).collect::<error::Result<Sequence>>()
        })
    }

    pub fn sorted_by_key<F>(
        &self,
        context: &context::DynamicContext,
        collation: Rc<Collation>,
        get: F,
    ) -> error::Result<Self>
    where
        F: FnMut(&Item) -> error::Result<Sequence>,
    {
        // see also sort_by_sequence in array.rs. The signatures are
        // sufficiently different we don't want to try to unify them.

        let items = self.iter().collect::<Vec<_>>();
        let keys = self.iter().map(get).collect::<error::Result<Vec<_>>>()?;

        let mut keys_and_items = keys.into_iter().zip(items).collect::<Vec<_>>();
        // sort by key. unfortunately sort_by requires the compare function
        // to be infallible. It's not in reality, so we make any failures
        // sort less, so they appear early on in the sequence.
        keys_and_items.sort_by(|(a_key, _), (b_key, _)| {
            a_key.compare(b_key, &collation, context.implicit_timezone())
        });
        // a pass to detect any errors; if sorting between two items is
        // impossible we want to raise a type error
        for ((a_key, _), (b_key, _)) in keys_and_items.iter().zip(keys_and_items.iter().skip(1)) {
            a_key.fallible_compare(b_key, &collation, context.implicit_timezone())?;
        }
        // now pick up items again
        let result = keys_and_items
            .into_iter()
            .map(|(_, item)| item)
            .collect::<Sequence>();
        Ok(result)
    }

    /// Flatten all arrays in this sequence
    pub fn flatten(&self) -> error::Result<Self> {
        let mut result = vec![];
        for item in self.iter() {
            if let Ok(array) = item.to_array() {
                for sequence in array.iter() {
                    for item in sequence.flatten()?.items()? {
                        result.push(item.clone());
                    }
                }
            } else {
                result.push(item.clone());
            }
        }
        Ok(result.into())
    }

    /// Normalize this sequence into a document node, according to
    /// <https://www.w3.org/TR/xslt-xquery-serialization-31/#serdm>
    pub fn normalize(&self, item_separator: &str, xot: &mut Xot) -> error::Result<xot::Node> {
        normalize(self, item_separator, xot)
    }

    /// Serialize this sequence according to serialization parameters
    pub(crate) fn serialize(
        &self,
        params: SerializationParameters,
        xot: &mut Xot,
    ) -> error::Result<String> {
        serialize_sequence(self, params, xot)
    }

    /// Display representation of the sequence
    pub fn display_representation(&self, xot: &Xot, context: &context::DynamicContext) -> String {
        // TODO: various unwraps
        match &self {
            Sequence::Empty(_) => "()".to_string(),
            Sequence::One(item) => item.item().display_representation(xot, context).unwrap(),
            Sequence::Many(items) => {
                let mut representations = Vec::with_capacity(self.len());
                for item in items.iter() {
                    representations.push(item.display_representation(xot, context).unwrap());
                }
                format!("(\n{}\n)", representations.join(",\n"))
            }
        }
    }
}

// turn a single item into a sequence
impl From<Item> for Sequence {
    fn from(item: Item) -> Self {
        Sequence::One(item.into())
    }
}

// turn a single atomic into a sequence
impl From<atomic::Atomic> for Sequence {
    fn from(atomic: atomic::Atomic) -> Self {
        let item: Item = atomic.into();
        item.into()
    }
}

// turn a single node into a sequence
impl From<xot::Node> for Sequence {
    fn from(node: xot::Node) -> Self {
        let item: Item = node.into();
        item.into()
    }
}

// turn a vector of items into a sequence
impl From<Vec<Item>> for Sequence {
    fn from(items: Vec<Item>) -> Self {
        match items.len() {
            0 => Sequence::Empty(Empty {}),
            1 => Sequence::One(items.into_iter().next().unwrap().into()),
            _ => Sequence::Many(items.into()),
        }
    }
}

// turn a vector of atomic into a sequence
impl From<Vec<atomic::Atomic>> for Sequence {
    fn from(atomics: Vec<atomic::Atomic>) -> Self {
        let items = atomics.into_iter().map(Item::from);
        items.collect()
    }
}

// turn a vector of node into a sequence
impl From<Vec<xot::Node>> for Sequence {
    fn from(nodes: Vec<xot::Node>) -> Self {
        let items = nodes.into_iter().map(Item::from);
        items.collect()
    }
}

// turn an iterator of items into a sequence
impl FromIterator<Item> for Sequence {
    fn from_iter<I: IntoIterator<Item = Item>>(iter: I) -> Self {
        let items = iter.into_iter().collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of item references into a sequence
impl<'a> FromIterator<&'a Item> for Sequence {
    fn from_iter<I: IntoIterator<Item = &'a Item>>(iter: I) -> Self {
        let items = iter.into_iter().cloned().collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of atomics into a sequence
impl FromIterator<atomic::Atomic> for Sequence {
    fn from_iter<I: IntoIterator<Item = atomic::Atomic>>(iter: I) -> Self {
        let items = iter.into_iter().map(Item::from).collect::<Vec<_>>();
        items.into()
    }
}

// turn an iterator of nodes into a sequence
impl FromIterator<xot::Node> for Sequence {
    fn from_iter<I: IntoIterator<Item = xot::Node>>(iter: I) -> Self {
        let items = iter.into_iter().map(Item::from).collect::<Vec<_>>();
        items.into()
    }
}

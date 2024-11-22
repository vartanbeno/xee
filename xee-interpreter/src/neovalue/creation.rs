use ahash::{HashSet, HashSetExt};

use crate::{error, sequence::Item, xml};

use super::{core::Empty, stack::StackSequence, traits::Sequence};

impl StackSequence {
    pub(crate) fn concat(self, other: Self) -> Self {
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

        let items = nodes.into_iter().map(Item::Node).collect::<Vec<_>>();
        items.into()
    }
}

impl From<Vec<Item>> for StackSequence {
    fn from(items: Vec<Item>) -> Self {
        match items.len() {
            0 => StackSequence::Empty(Empty {}),
            1 => StackSequence::One(items.into_iter().next().unwrap().into()),
            _ => StackSequence::Many(items.into()),
        }
    }
}

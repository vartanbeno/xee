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
            _ => unreachable!(),
        }
    }
}

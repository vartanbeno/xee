use xee_interpreter::sequence::Item;

/// A sequence of XPath items. You can iterate through these.
#[derive(Debug)]
pub struct Sequence {
    sequence: xee_interpreter::sequence::Sequence,
}

impl Sequence {
    pub(crate) fn new(sequence: xee_interpreter::sequence::Sequence) -> Self {
        Self { sequence }
    }
}

impl IntoIterator for Sequence {
    type Item = Item;
    type IntoIter = xee_interpreter::sequence::ItemIter;

    fn into_iter(self) -> Self::IntoIter {
        // items() can return Err if the sequence is absent but we should
        // already have handled that before this sequence is even created; we
        // don't want to return an absent sequence but instead turn this into
        // an error. So we can safely unwrap here.
        self.sequence.items().expect("sequence is absent")
    }
}

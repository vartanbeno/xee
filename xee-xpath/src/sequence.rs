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

// impl IntoIterator for Sequence {
//     type Item = xee_interpreter::error::Result<Item>;
//     type IntoIter = xee_interpreter::sequence::ItemIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.sequence.items()
//     }
// }

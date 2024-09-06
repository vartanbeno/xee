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
        // TODO: this unwrap is weird, but items can fail to be created
        // if the items are absent. but if we ensure in our API that
        // an absent Sequence is never created, we can remove this unwrap.
        // That is, instead of items being a result we could make *sequence*
        // fail if it would be created as absent
        self.sequence.items().unwrap()
    }
}

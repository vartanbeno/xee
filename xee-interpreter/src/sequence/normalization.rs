// https://www.w3.org/TR/xslt-xquery-serialization-31/#serdm

use crate::{atomic, error};

use super::Sequence;

fn normalize(sequence: Sequence) -> error::Result<Sequence> {
    let sequence = if !sequence.is_empty() {
        // any arrays in the sequences sare flattened
        sequence.flatten()?
    } else {
        // 1. a sequence that consists of a zero length strength
        let atom: atomic::Atomic = "".into();
        Sequence::from(vec![atom])
    };
    todo!()
}

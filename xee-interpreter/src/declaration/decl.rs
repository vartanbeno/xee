use crate::{function, pattern::PatternLookup};

#[derive(Debug)]
pub struct Declarations {
    pub pattern_lookup: PatternLookup<function::InlineFunctionId>,
}

impl Declarations {
    pub(crate) fn new() -> Self {
        Self {
            pattern_lookup: PatternLookup::new(),
        }
    }
}

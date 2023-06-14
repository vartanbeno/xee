use crate::output;

#[derive(Debug, Clone, PartialEq)]
pub struct OutputSequence {
    items: Vec<output::OutputItem>,
}

impl OutputSequence {
    pub fn new(items: Vec<output::OutputItem>) -> Self {
        Self { items }
    }

    pub fn items(&self) -> &[output::OutputItem] {
        &self.items
    }

    // XXX unfortunate duplication with effective_boolean_value
    // on Value
    pub fn effective_boolean_value(&self) -> std::result::Result<bool, crate::error::Error> {
        if self.items.is_empty() {
            return Ok(false);
        }
        if matches!(self.items[0], output::OutputItem::Node(_)) {
            return Ok(true);
        }
        if self.items.len() != 1 {
            return Err(crate::Error::FORG0006);
        }
        match self.items[0].to_bool() {
            Ok(b) => Ok(b),
            Err(_) => Err(crate::Error::FORG0006),
        }
    }
}

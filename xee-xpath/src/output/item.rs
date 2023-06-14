use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, PartialEq, Clone)]
pub enum OutputItem {
    Atomic(output::OutputAtomic),
    Function(output::OutputClosure),
    Node(xml::Node),
}

impl From<&OutputItem> for stack::StackItem {
    fn from(item: &OutputItem) -> Self {
        match item {
            OutputItem::Atomic(a) => stack::StackItem::Atomic(a.into()),
            OutputItem::Function(_f) => todo!("Cannot turn output functions into functions yet"),
            OutputItem::Node(n) => stack::StackItem::Node(*n),
        }
    }
}

impl From<OutputItem> for stack::StackItem {
    fn from(item: OutputItem) -> Self {
        (&item).into()
    }
}

impl OutputItem {
    // TODO these should not return ValueResult as they're in the public API
    pub fn to_atomic(&self) -> stack::ValueResult<&output::OutputAtomic> {
        match self {
            OutputItem::Atomic(a) => Ok(a),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_node(&self) -> stack::ValueResult<xml::Node> {
        match self {
            OutputItem::Node(n) => Ok(*n),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> stack::ValueResult<bool> {
        match self {
            OutputItem::Atomic(a) => a.to_bool(),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> stack::ValueResult<String> {
        match self {
            OutputItem::Atomic(a) => Ok(a.string_value()?),
            OutputItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::ValueError::Type),
        }
    }
}

use xot::Xot;

use super::atomic::OutputAtomic;
use super::error::ValueError;
use super::function::OutputClosure;
use super::node::Node;
use crate::stack::StackItem;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, PartialEq, Clone)]
pub enum OutputItem {
    Atomic(OutputAtomic),
    Function(OutputClosure),
    Node(Node),
}

impl From<&OutputItem> for StackItem {
    fn from(item: &OutputItem) -> Self {
        match item {
            OutputItem::Atomic(a) => StackItem::Atomic(a.into()),
            OutputItem::Function(_f) => todo!("Cannot turn output functions into functions yet"),
            OutputItem::Node(n) => StackItem::Node(*n),
        }
    }
}

impl From<OutputItem> for StackItem {
    fn from(item: OutputItem) -> Self {
        (&item).into()
    }
}

impl OutputItem {
    pub fn to_atomic(&self) -> Result<&OutputAtomic> {
        match self {
            OutputItem::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_node(&self) -> Result<Node> {
        match self {
            OutputItem::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            OutputItem::Atomic(a) => a.to_bool(),
            _ => Err(ValueError::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            OutputItem::Atomic(a) => Ok(a.string_value()?),
            OutputItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(ValueError::Type),
        }
    }
}

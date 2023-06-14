use xot::Xot;

use super::atomic::OutputAtomic;
use super::function::OutputClosure;
use super::node::Node;
use crate::stack;

type Result<T> = std::result::Result<T, stack::ValueError>;

#[derive(Debug, PartialEq, Clone)]
pub enum OutputItem {
    Atomic(OutputAtomic),
    Function(OutputClosure),
    Node(Node),
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
    pub fn to_atomic(&self) -> Result<&OutputAtomic> {
        match self {
            OutputItem::Atomic(a) => Ok(a),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_node(&self) -> Result<Node> {
        match self {
            OutputItem::Node(n) => Ok(*n),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            OutputItem::Atomic(a) => a.to_bool(),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            OutputItem::Atomic(a) => Ok(a.string_value()?),
            OutputItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::ValueError::Type),
        }
    }
}

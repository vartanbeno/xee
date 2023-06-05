use ahash::{HashSet, HashSetExt};
use miette::SourceSpan;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::vec;
use xot::Xot;

use crate::annotation::Annotations;
use crate::ast;
use crate::comparison;
use crate::context::DynamicContext;
use crate::ir;
use crate::value::atomic::Atomic;
use crate::value::error::ValueError;
use crate::value::node::Node;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct FunctionId(pub(crate) usize);

impl FunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Step {
    pub(crate) axis: ast::Axis,
    pub(crate) node_test: ast::NodeTest,
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) constants: Vec<Value>,
    pub(crate) closure_names: Vec<ir::Name>,
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClosureFunctionId {
    Static(StaticFunctionId),
    Dynamic(FunctionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub(crate) function_id: ClosureFunctionId,
    pub(crate) values: Vec<Value>,
}

// Speculation: A rc value would be a lot smaller, though at the
// cost of indirection. So I'm not sure it would be faster; we'd get
// faster stack operations but slower heap access and less cache locality.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atomic(Atomic),
    Sequence(Rc<RefCell<Sequence>>),
    Closure(Rc<Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<Step>),
    Node(Node),
}

impl Value {
    pub(crate) fn from_item(item: Item) -> Self {
        match item {
            Item::Atomic(a) => Value::Atomic(a),
            Item::Node(n) => Value::Node(n),
            Item::Function(f) => Value::Closure(f),
        }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        match self {
            Value::Atomic(a) => Ok(a.clone()),
            Value::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }

    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self {
            Value::Atomic(a) => a.to_bool(),
            Value::Sequence(s) => {
                let s = s.borrow();
                // If its operand is an empty sequence, fn:boolean returns false.
                if s.is_empty() {
                    return Ok(false);
                }
                // If its operand is a sequence whose first item is a node, fn:boolean returns true.
                if matches!(s.items[0], Item::Node(_)) {
                    return Ok(true);
                }
                // If its operand is a singleton value
                let singleton = s.singleton()?;
                singleton.to_bool()
            }
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Value::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            Value::Closure(_) => Err(ValueError::Type),
            Value::Step(_) => Err(ValueError::Type),
        }
    }

    pub fn to_sequence(&self) -> Result<Rc<RefCell<Sequence>>> {
        match self {
            Value::Sequence(s) => Ok(s.clone()),
            Value::Atomic(a) => Ok(Rc::new(RefCell::new(Sequence::from_atomic(a.clone())))),
            Value::Node(a) => Ok(Rc::new(RefCell::new(Sequence::from_node(*a)))),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_closure(&self) -> Result<&Closure> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_step(&self) -> Result<Rc<Step>> {
        match self {
            Value::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_node(&self) -> Result<Node> {
        match self {
            Value::Node(n) => Ok(*n),
            Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            Value::Sequence(s) => s.borrow().is_empty(),
            Value::Atomic(Atomic::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> Result<String> {
        let value = match self {
            Value::Atomic(atomic) => atomic.string_value()?,
            Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => Value::from_item(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(ValueError::Type)?,
                }
            }
            Value::Node(node) => node.string_value(xot),
            Value::Closure(_) => Err(ValueError::Type)?,
            Value::Step(_) => Err(ValueError::Type)?,
        };
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

impl Item {
    pub fn to_atomic(&self) -> Result<&Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_node(&self) -> Result<Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            Item::Atomic(a) => a.to_bool(),
            _ => Err(ValueError::Type),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(ValueError::Type),
        }
    }

    pub fn to_stack_value(self) -> Value {
        match self {
            Item::Atomic(a) => Value::Atomic(a),
            Item::Node(n) => Value::Node(n),
            Item::Function(f) => Value::Closure(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub(crate) items: Vec<Item>,
}

impl Sequence {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn as_slice(&self) -> &[Item] {
        &self.items
    }

    pub(crate) fn from_vec(items: Vec<Item>) -> Self {
        Self { items }
    }

    pub(crate) fn from_atomic(atomic: Atomic) -> Self {
        if matches!(atomic, Atomic::Empty) {
            return Self::new();
        }
        Self {
            items: vec![Item::Atomic(atomic)],
        }
    }

    pub(crate) fn from_node(node: Node) -> Self {
        Self {
            items: vec![Item::Node(node)],
        }
    }

    pub(crate) fn singleton(&self) -> Result<&Item> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(ValueError::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: Value) {
        match value {
            Value::Atomic(a) => self.items.push(Item::Atomic(a)),
            Value::Closure(c) => self.items.push(Item::Function(c)),
            Value::Sequence(s) => self.extend(s),
            Value::Node(n) => self.items.push(Item::Node(n)),
            _ => panic!("unexpected value: {:?}", value),
        }
    }

    pub(crate) fn push(&mut self, item: &Item) {
        self.items.push(item.clone());
    }

    pub(crate) fn extend(&mut self, other: Rc<RefCell<Sequence>>) {
        for item in &other.borrow().items {
            self.push(item);
        }
    }

    pub(crate) fn atomize(&self, xot: &Xot) -> Sequence {
        let mut items = Vec::new();
        for item in &self.items {
            match item {
                Item::Atomic(a) => items.push(Item::Atomic(a.clone())),
                Item::Node(n) => {
                    for typed_value in n.typed_value(xot) {
                        items.push(Item::Atomic(typed_value));
                    }
                }
                // XXX need code to handle array case
                Item::Function(..) => panic!("cannot atomize a function"),
            }
        }
        Sequence { items }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        // we avoid using atomize as an optimization, so we don't
        // have to atomize all entries just to get the first item
        match self.len() {
            0 => Ok(Atomic::Empty),
            1 => {
                let item = &self.items[0];
                match item {
                    Item::Atomic(a) => Ok(a.clone()),
                    Item::Node(n) => {
                        let mut t = n.typed_value(context.xot);
                        if t.len() != 1 {
                            return Err(ValueError::XPTY0004);
                        }
                        Ok(t.remove(0))
                    }
                    // XXX need code to handle array case
                    Item::Function(..) => panic!("cannot atomize a function"),
                }
            }
            _ => Err(ValueError::XPTY0004),
        }
    }

    pub(crate) fn to_atoms(&self, xot: &Xot) -> Vec<Atomic> {
        let mut atoms = Vec::new();
        let atomized = self.atomize(xot);
        for item in atomized.items {
            match item {
                Item::Atomic(a) => atoms.push(a),
                _ => unreachable!("atomize returned a non-atomic item"),
            }
        }
        atoms
    }

    pub(crate) fn concat(&self, other: &Sequence) -> Sequence {
        let mut items = self.items.clone();
        items.extend(other.items.clone());
        Sequence { items }
    }

    pub(crate) fn union(&self, other: &Sequence, annotations: &Annotations) -> Result<Sequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                Item::Node(node) => *node,
                Item::Atomic(..) => return Err(ValueError::Type),
                Item::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                Item::Node(node) => *node,
                Item::Atomic(..) => return Err(ValueError::Type),
                Item::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes.into_iter().map(Item::Node).collect::<Vec<_>>();
        Ok(Sequence { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_compares_with_decimal() {
        let a = Atomic::Integer(1);
        let b = Atomic::Decimal(Decimal::from(1));
        assert_eq!(a, b);
    }
}

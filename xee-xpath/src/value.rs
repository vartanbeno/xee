use ahash::{HashSet, HashSetExt};
use miette::{Diagnostic, SourceSpan};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::vec;
use thiserror::Error;
use xot::Xot;

use crate::annotation::Annotations;
use crate::ast;
use crate::comparison;
use crate::dynamic_context::DynamicContext;
use crate::error::Error;
use crate::instruction::{decode_instructions, Instruction};
use crate::ir;

#[derive(Debug, Error, Diagnostic, Clone, PartialEq)]
pub enum ValueError {
    #[error("Type error")]
    XPTY0004,
    #[error("Type error")]
    Type,
    #[error("Overflow/underflow")]
    Overflow,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Absent")]
    Absent,
    // Explicit error raised with Error
    #[error("Error")]
    Error(Error),
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Node {
    Xot(xot::Node),
    Attribute(xot::Node, xot::NameId),
    Namespace(xot::Node, xot::PrefixId),
}

impl Node {
    pub(crate) fn parent(&self, xot: &Xot) -> Option<Node> {
        match self {
            Node::Xot(node) => xot.parent(*node).map(Self::Xot),
            Node::Attribute(node, _) => Some(Self::Xot(*node)),
            Node::Namespace(..) => None,
        }
    }

    pub fn xot_node(&self) -> xot::Node {
        match self {
            Node::Xot(node) => *node,
            Node::Attribute(node, _) => *node,
            Node::Namespace(node, _) => *node,
        }
    }

    // if node is a Node::Xot, then we can apply a Xot iterator to it and then wrap them
    // with Node::Xot and box the results. Otherwise we always get an empty iterator.
    pub(crate) fn xot_iterator<'a, F, G>(&self, f: F) -> Box<dyn Iterator<Item = Node> + 'a>
    where
        G: Iterator<Item = xot::Node> + 'a,
        F: Fn(xot::Node) -> G,
    {
        match self {
            Node::Xot(node) => Box::new(f(*node).map(Node::Xot)),
            Node::Attribute(..) | Node::Namespace(..) => Box::new(std::iter::empty()),
        }
    }

    pub(crate) fn node_name(&self, xot: &Xot) -> Option<xot::NameId> {
        match self {
            Node::Xot(node) => match xot.value(*node) {
                xot::Value::Element(element) => Some(element.name()),
                xot::Value::Text(..) => None,
                // XXX this is incorrect; should return a named based on the
                // target property. this requires a modification in Xot to make
                // this accessible.
                xot::Value::ProcessingInstruction(..) => None,
                xot::Value::Comment(..) => None,
                xot::Value::Root => None,
            },
            Node::Attribute(_, name_id) => Some(*name_id),
            // XXX could return something if there is a prefix
            Node::Namespace(_, _) => None,
        }
    }

    pub(crate) fn local_name(&self, xot: &Xot) -> String {
        if let Some(name) = self.node_name(xot) {
            let (local_name, _uri) = xot.name_ns_str(name);
            local_name.to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn namespace_uri(&self, xot: &Xot) -> String {
        if let Some(name) = self.node_name(xot) {
            let (_local_name, uri) = xot.name_ns_str(name);
            uri.to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn typed_value(&self, xot: &Xot) -> Vec<Atomic> {
        // for now we don't know any types of nodes yet
        let s = self.string_value(xot);
        vec![Atomic::Untyped(Rc::new(s))]
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> String {
        match self {
            Node::Xot(node) => match xot.value(*node) {
                xot::Value::Element(_) => descendants_to_string(xot, *node),
                xot::Value::Text(text) => text.get().to_string(),
                xot::Value::ProcessingInstruction(pi) => pi.data().unwrap_or("").to_string(),
                xot::Value::Comment(comment) => comment.get().to_string(),
                xot::Value::Root => descendants_to_string(xot, *node),
            },
            Node::Attribute(node, name) => {
                let element = xot.element(*node).unwrap();
                element.get_attribute(*name).unwrap().to_string()
            }
            Node::Namespace(..) => {
                todo!("not yet: return the value of the uri property")
            }
        }
    }
}

fn descendants_to_string(xot: &Xot, node: xot::Node) -> String {
    let texts = xot.descendants(node).filter_map(|n| xot.text_str(n));
    let mut r = String::new();
    for text in texts {
        r.push_str(text);
    }
    r
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) constants: Vec<StackValue>,
    pub(crate) closure_names: Vec<ir::Name>,
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

impl Function {
    pub(crate) fn decoded(&self) -> Vec<Instruction> {
        decode_instructions(&self.chunk)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClosureFunctionId {
    Static(StaticFunctionId),
    Dynamic(FunctionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub(crate) function_id: ClosureFunctionId,
    pub(crate) values: Vec<StackValue>,
}

// Speculation: A rc stack value would be a lot smaller, though at the
// cost of indirection. So I'm not sure it would be faster; we'd get
// faster stack operations but slower heap access and less cache locality.

#[derive(Debug, Clone, PartialEq)]
pub enum StackValue {
    Atomic(Atomic),
    Sequence(Rc<RefCell<Sequence>>),
    Closure(Rc<Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<Step>),
    Node(Node),
}

impl StackValue {
    pub(crate) fn from_item(item: Item) -> Self {
        match item {
            Item::Atomic(a) => StackValue::Atomic(a),
            Item::Node(n) => StackValue::Node(n),
            Item::Function(f) => StackValue::Closure(f),
        }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        match self {
            StackValue::Atomic(a) => Ok(a.clone()),
            StackValue::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }

    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self {
            StackValue::Atomic(a) => a.to_bool(),
            StackValue::Sequence(s) => {
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
            StackValue::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            StackValue::Closure(_) => Err(ValueError::Type),
            StackValue::Step(_) => Err(ValueError::Type),
        }
    }

    pub fn to_sequence(&self) -> Result<Rc<RefCell<Sequence>>> {
        match self {
            StackValue::Sequence(s) => Ok(s.clone()),
            StackValue::Atomic(a) => Ok(Rc::new(RefCell::new(Sequence::from_atomic(a.clone())))),
            StackValue::Node(a) => Ok(Rc::new(RefCell::new(Sequence::from_node(*a)))),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_closure(&self) -> Result<&Closure> {
        match self {
            StackValue::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_step(&self) -> Result<Rc<Step>> {
        match self {
            StackValue::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_node(&self) -> Result<Node> {
        match self {
            StackValue::Node(n) => Ok(*n),
            StackValue::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            StackValue::Sequence(s) => s.borrow().is_empty(),
            StackValue::Atomic(Atomic::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> Result<String> {
        let value = match self {
            StackValue::Atomic(atomic) => atomic.string_value()?,
            StackValue::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => StackValue::from_item(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(ValueError::Type)?,
                }
            }
            StackValue::Node(node) => node.string_value(xot),
            StackValue::Closure(_) => Err(ValueError::Type)?,
            StackValue::Step(_) => Err(ValueError::Type)?,
        };
        Ok(value)
    }
}

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, Eq)]
pub enum Atomic {
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),
    Decimal(Decimal),
    String(Rc<String>),
    Untyped(Rc<String>),
    // a special marker to note empty sequences after atomization
    // This should be treated as an emtpy sequence.
    Empty,
    // a special marker to indicate an absent context item
    Absent,
}

impl Display for Atomic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Atomic::Boolean(b) => write!(f, "{}", b),
            Atomic::Integer(i) => write!(f, "{}", i),
            Atomic::Float(n) => write!(f, "{}", n),
            Atomic::Double(d) => write!(f, "{}", d),
            Atomic::Decimal(d) => write!(f, "{}", d),
            Atomic::String(s) => write!(f, "{}", s),
            Atomic::Untyped(s) => write!(f, "{}", s),
            Atomic::Empty => write!(f, "()"),
            Atomic::Absent => write!(f, "absent"),
        }
    }
}

impl Atomic {
    pub(crate) fn to_integer(&self) -> Result<i64> {
        match self {
            Atomic::Integer(i) => Ok(*i),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_decimal(&self) -> Result<Decimal> {
        match self {
            Atomic::Decimal(d) => Ok(*d),
            Atomic::Integer(i) => Ok(Decimal::from(*i)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_float(&self) -> Result<OrderedFloat<f32>> {
        match self {
            Atomic::Float(f) => Ok(*f),
            Atomic::Decimal(d) => Ok(OrderedFloat(d.to_f32().ok_or(ValueError::Type)?)),
            Atomic::Integer(_) => Ok(OrderedFloat(
                self.to_decimal()?.to_f32().ok_or(ValueError::Type)?,
            )),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_double(&self) -> Result<OrderedFloat<f64>> {
        match self {
            Atomic::Double(d) => Ok(*d),
            Atomic::Float(OrderedFloat(f)) => Ok(OrderedFloat(*f as f64)),
            Atomic::Decimal(d) => Ok(OrderedFloat(d.to_f64().ok_or(ValueError::Type)?)),
            Atomic::Integer(_) => Ok(OrderedFloat(
                self.to_decimal()?.to_f64().ok_or(ValueError::Type)?,
            )),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
            Atomic::Boolean(b) => Ok(*b),
            Atomic::String(s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
            Atomic::Empty => Ok(false),
            Atomic::Absent => Err(ValueError::Absent),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub fn to_str(&self) -> Result<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(ValueError::Type),
        }
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub fn string_value(&self) -> Result<String> {
        Ok(match self {
            Atomic::String(s) => s.to_string(),
            Atomic::Untyped(s) => s.to_string(),
            Atomic::Boolean(b) => b.to_string(),
            Atomic::Integer(i) => i.to_string(),
            Atomic::Float(f) => f.to_string(),
            Atomic::Double(d) => d.to_string(),
            Atomic::Decimal(d) => d.to_string(),
            Atomic::Empty => "".to_string(),
            Atomic::Absent => Err(ValueError::Absent)?,
        })
    }

    pub(crate) fn is_nan(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_nan(),
            Atomic::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    pub(crate) fn is_infinite(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_infinite(),
            Atomic::Double(d) => d.is_infinite(),
            _ => false,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_zero(),
            Atomic::Double(d) => d.is_zero(),
            Atomic::Decimal(d) => d.is_zero(),
            Atomic::Integer(i) => *i == 0,
            _ => false,
        }
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            Atomic::Float(_) | Atomic::Double(_) | Atomic::Decimal(_) | Atomic::Integer(_)
        )
    }

    pub(crate) fn general_comparison_cast(&self, v: &str) -> Result<Atomic> {
        match self {
            // i. If T is a numeric type or is derived from a numeric type, then V
            // is cast to xs:double.
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_) => {
                // cast string to double
                // Need to unify the parsing code with literal parser in parse_ast
                Ok(Atomic::Double(OrderedFloat(
                    v.parse::<f64>().map_err(|_| ValueError::Overflow)?,
                )))
            }
            // don't handle ii and iii for now
            // iv. In all other cases, V is cast to the primitive base type of T.
            Atomic::String(_) => Ok(Atomic::String(Rc::new(v.to_string()))),
            Atomic::Boolean(_) => {
                todo!();
            }
            Atomic::Untyped(_) => unreachable!(),
            Atomic::Empty => unreachable!(),
            Atomic::Absent => Err(ValueError::Type),
        }
    }
}

impl PartialEq for Atomic {
    fn eq(&self, other: &Self) -> bool {
        match comparison::value_eq(self, other) {
            Ok(b) => b.to_bool().unwrap(),
            Err(_) => false,
        }
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

    pub fn to_stack_value(self) -> StackValue {
        match self {
            Item::Atomic(a) => StackValue::Atomic(a),
            Item::Node(n) => StackValue::Node(n),
            Item::Function(f) => StackValue::Closure(f),
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

    pub(crate) fn push_stack_value(&mut self, value: StackValue) {
        match value {
            StackValue::Atomic(a) => self.items.push(Item::Atomic(a)),
            StackValue::Closure(c) => self.items.push(Item::Function(c)),
            StackValue::Sequence(s) => self.extend(s),
            StackValue::Node(n) => self.items.push(Item::Node(n)),
            _ => panic!("unexpected stack value: {:?}", value),
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

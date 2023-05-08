use ahash::{HashSet, HashSetExt};
use miette::SourceSpan;
use rust_decimal::prelude::*;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use xot::Xot;

use crate::annotation::Annotations;
use crate::ast;
use crate::context::Context;
use crate::instruction::{decode_instructions, Instruction};
use crate::ir;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueError {
    XPTY0004,
    Type,
    Overflow,
    DivisionByZero,
    StackOverflow,
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

    pub(crate) fn string(&self, xot: &Xot) -> String {
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

    pub(crate) fn as_atomic(&self, context: &Context) -> Result<Atomic> {
        match self {
            StackValue::Atomic(a) => Ok(a.clone()),
            StackValue::Sequence(s) => s.borrow().as_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }

    pub fn as_sequence(&self) -> Result<Rc<RefCell<Sequence>>> {
        match self {
            StackValue::Sequence(s) => Ok(s.clone()),
            StackValue::Atomic(a) => Ok(Rc::new(RefCell::new(Sequence::from_atomic(a.clone())))),
            StackValue::Node(a) => Ok(Rc::new(RefCell::new(Sequence::from_node(*a)))),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_closure(&self) -> Result<&Closure> {
        match self {
            StackValue::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_step(&self) -> Result<Rc<Step>> {
        match self {
            StackValue::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_node(&self) -> Result<Node> {
        match self {
            StackValue::Node(n) => Ok(*n),
            StackValue::Sequence(s) => s.borrow().singleton().and_then(|n| n.as_node()),
            _ => Err(ValueError::Type),
        }
    }
}

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, PartialEq)]
pub enum Atomic {
    Boolean(bool),
    Integer(i64),
    Float(f32),
    Double(f64),
    Decimal(Decimal),
    // and many more
    String(Rc<String>),
    // a special marker to note empty sequences after atomization
    Empty,
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
            Atomic::Empty => write!(f, "()"),
        }
    }
}

impl Atomic {
    pub(crate) fn as_integer(&self) -> Result<i64> {
        match self {
            Atomic::Integer(i) => Ok(*i),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_decimal(&self) -> Result<Decimal> {
        match self {
            Atomic::Decimal(d) => Ok(*d),
            Atomic::Integer(i) => Ok(Decimal::from(*i)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_float(&self) -> Result<f32> {
        match self {
            Atomic::Float(f) => Ok(*f),
            Atomic::Decimal(d) => d.to_f32().ok_or(ValueError::Type),
            Atomic::Integer(_) => self.as_decimal()?.to_f32().ok_or(ValueError::Type),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_double(&self) -> Result<f64> {
        match self {
            Atomic::Double(d) => Ok(*d),
            Atomic::Float(f) => Ok(*f as f64),
            Atomic::Decimal(d) => d.to_f64().ok_or(ValueError::Type),
            Atomic::Integer(_) => self.as_decimal()?.to_f64().ok_or(ValueError::Type),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_bool(&self) -> Result<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Boolean(b) => Ok(*b),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn as_string(&self) -> Result<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(ValueError::Type),
        }
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

impl Item {
    pub(crate) fn as_atomic(&self) -> Result<&Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub(crate) fn as_node(&self) -> Result<Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
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
                    // this should get the typed-value, which is a sequence,
                    // but for now we get the string value
                    let a = n.string(xot);
                    items.push(Item::Atomic(Atomic::String(Rc::new(a))));
                }
                // XXX need code to handle array case
                Item::Function(..) => panic!("cannot atomize a function"),
            }
        }
        Sequence { items }
    }

    pub(crate) fn as_atomic(&self, context: &Context) -> Result<Atomic> {
        let mut atomized = self.atomize(context.xot);
        let len = atomized.items.len();
        match len {
            0 => Ok(Atomic::Empty),
            1 => {
                let item = atomized.items.remove(0);
                match item {
                    Item::Atomic(a) => Ok(a),
                    _ => unreachable!("atomize returned a non-atomic item"),
                }
            }
            _ => Err(ValueError::XPTY0004),
        }
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

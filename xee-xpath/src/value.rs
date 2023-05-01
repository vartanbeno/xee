use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use xot::Xot;

use crate::annotation::Annotations;
use crate::ast;
use crate::error::{Error, Result};
use crate::instruction::{decode_instructions, Instruction};
use crate::ir;

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
pub(crate) struct Step {
    pub(crate) axis: ast::Axis,
    pub(crate) node_test: ast::NodeTest,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum Node {
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
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) constants: Vec<StackValue>,
    pub(crate) closure_names: Vec<ir::Name>,
    pub(crate) chunk: Vec<u8>,
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
pub(crate) struct Closure {
    pub(crate) function_id: ClosureFunctionId,
    pub(crate) values: Vec<StackValue>,
}

// Speculation: A rc stack value would be a lot smaller, though at the
// cost of indirection. So I'm not sure it would be faster; we'd get
// faster stack operations but slower heap access and less cache locality.

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackValue {
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

    pub(crate) fn as_atomic(&self) -> Option<&Atomic> {
        match self {
            StackValue::Atomic(a) => Some(a),
            _ => None,
        }
    }

    pub(crate) fn as_sequence(&self) -> Option<Rc<RefCell<Sequence>>> {
        match self {
            StackValue::Sequence(s) => Some(s.clone()),
            StackValue::Atomic(a) => Some(Rc::new(RefCell::new(Sequence::from_atomic(a.clone())))),
            StackValue::Node(a) => Some(Rc::new(RefCell::new(Sequence::from_node(*a)))),
            _ => None,
        }
    }

    pub(crate) fn as_closure(&self) -> Option<&Closure> {
        match self {
            StackValue::Closure(c) => Some(c),
            _ => None,
        }
    }

    // pub(crate) fn as_static_function(&self) -> Option<StaticFunctionId> {
    //     match self {
    //         StackValue::StaticFunction(f) => Some(*f),
    //         _ => None,
    //     }
    // }

    pub(crate) fn as_step(&self) -> Option<Rc<Step>> {
        match self {
            StackValue::Step(s) => Some(Rc::clone(s)),
            _ => None,
        }
    }

    pub(crate) fn as_node(&self) -> Option<Node> {
        match self {
            StackValue::Node(n) => Some(*n),
            StackValue::Sequence(s) => s.borrow().singleton().and_then(|n| n.as_node()),
            _ => None,
        }
    }
}

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Atomic {
    // should string be Rc?
    // String(String),
    Boolean(bool),
    // Decimal, use a decimal type
    Integer(i64), // is really a decimal, but special case it for now
    Float(f32),
    Double(f64),
    // and many more
    String(Rc<String>),
}

impl Atomic {
    pub(crate) fn as_integer(&self) -> Option<i64> {
        match self {
            Atomic::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Atomic::Integer(i) => Some(*i != 0),
            Atomic::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub(crate) fn as_string(&self) -> Option<String> {
        match self {
            Atomic::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

impl Item {
    pub(crate) fn as_atomic(&self) -> Option<&Atomic> {
        match self {
            Item::Atomic(a) => Some(a),
            _ => None,
        }
    }
    pub(crate) fn as_node(&self) -> Option<Node> {
        match self {
            Item::Node(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence {
    pub(crate) items: Vec<Item>,
}

impl Sequence {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
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

    pub(crate) fn singleton(&self) -> Option<&Item> {
        if self.items.len() == 1 {
            Some(&self.items[0])
        } else {
            None
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
                Item::Atomic(..) => return Err(Error::TypeError),
                Item::Function(..) => return Err(Error::TypeError),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                Item::Node(node) => *node,
                Item::Atomic(..) => return Err(Error::TypeError),
                Item::Function(..) => return Err(Error::TypeError),
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

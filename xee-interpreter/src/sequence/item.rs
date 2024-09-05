use std::rc::Rc;
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::function;
use crate::stack;

/// An XPath item. These are the items that make up an XPath sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// An atomic value.
    ///
    /// One of the value types defined by XPath, indicated by an `xs:*` type
    /// such as `xs:integer` or `xs:string`.
    Atomic(atomic::Atomic),
    /// A node in an XML document.
    ///
    /// This is defined using the [`xot`] library.
    Node(xot::Node),
    /// An XPath function type.
    Function(Rc<function::Function>),
}

impl Item {
    /// Try to get the atomic value of the item.
    pub fn to_atomic(&self) -> error::Result<atomic::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(error::Error::XPTY0004),
        }
    }

    /// Try to get the node value of the item.
    pub fn to_node(&self) -> error::Result<xot::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(error::Error::XPTY0004),
        }
    }

    /// Try to get the function value of the item.
    pub fn to_function(&self) -> error::Result<Rc<function::Function>> {
        match self {
            Item::Function(f) => Ok(f.clone()),
            _ => Err(error::Error::XPTY0004),
        }
    }

    /// Try to get the value as an XPath Map.
    pub fn to_map(&self) -> error::Result<function::Map> {
        match self {
            Item::Function(function) => match function.as_ref() {
                function::Function::Map(map) => Ok(map.clone()),
                _ => Err(error::Error::XPTY0004),
            },
            _ => Err(error::Error::XPTY0004),
        }
    }

    /// Try to get the value as an XPath Array.
    pub fn to_array(&self) -> error::Result<function::Array> {
        match self {
            Item::Function(function) => match function.as_ref() {
                function::Function::Array(array) => Ok(array.clone()),
                _ => Err(error::Error::XPTY0004),
            },
            _ => Err(error::Error::XPTY0004),
        }
    }

    /// Obtain the [effective boolean
    /// value](https://www.w3.org/TR/xpath-31/#id-ebv) of the item.
    ///
    /// - If the item is a node, it's true.
    ///
    /// - If the item is a boolean, it's the value of the boolean.
    ///
    /// - If the item is a string, it's false if it's empty, otherwise true.
    ///
    /// - If the item is a numeric type, it's false if it's NaN or zero,
    ///   otherwise true.
    ///
    /// - Functions are always errors.
    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Item::Atomic(a) => a.effective_boolean_value(),
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Item::Node(_) => Ok(true),
            Item::Function(_) => Err(error::Error::FORG0006),
        }
    }

    /// Construct the string value.
    ///
    /// - For an atomic value, it casts it to a string using the canonical
    ///   lexical representation rules as defined by XML Schema.
    ///
    /// - For a node, it returns the [string value of the
    ///   node](https://www.w3.org/TR/xpath-31/#id-typed-value).
    ///
    /// - For a function, it errors.
    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        match self {
            Item::Atomic(atomic) => atomic.string_value(),
            Item::Node(node) => Ok(xot.string_value(*node)),
            Item::Function(_) => Err(error::Error::FOTY0014),
        }
    }

    /// Check whether this item is represents an XPath Map.
    pub(crate) fn is_map(&self) -> bool {
        match self {
            Item::Function(function) => matches!(function.as_ref(), function::Function::Map(_)),
            _ => false,
        }
    }

    /// Check whether this item is represents an XPath Array.
    pub(crate) fn is_array(&self) -> bool {
        match self {
            Item::Function(function) => matches!(function.as_ref(), function::Function::Array(_)),
            _ => false,
        }
    }
}

impl<T> From<T> for Item
where
    T: Into<atomic::Atomic>,
{
    fn from(a: T) -> Self {
        Self::Atomic(a.into())
    }
}

impl From<xot::Node> for Item {
    fn from(node: xot::Node) -> Self {
        Self::Node(node)
    }
}

impl From<function::Function> for Item {
    fn from(f: function::Function) -> Self {
        Self::Function(Rc::new(f))
    }
}

impl From<Rc<function::Function>> for Item {
    fn from(f: Rc<function::Function>) -> Self {
        Self::Function(f)
    }
}

impl From<function::Array> for Item {
    fn from(array: function::Array) -> Self {
        Self::Function(Rc::new(function::Function::Array(array)))
    }
}

impl From<function::Map> for Item {
    fn from(map: function::Map) -> Self {
        Self::Function(Rc::new(function::Function::Map(map)))
    }
}

#[derive(Clone)]
pub enum AtomizedItemIter<'a> {
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(AtomizedNodeIter),
    Array(AtomizedArrayIter<'a>),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a> AtomizedItemIter<'a> {
    pub(crate) fn new(item: Item, xot: &'a Xot) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a)),
            Item::Node(n) => Self::Node(AtomizedNodeIter::new(n, xot)),
            Item::Function(function) => match function.as_ref() {
                function::Function::Array(a) => Self::Array(AtomizedArrayIter::new(a.clone(), xot)),
                _ => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
            },
        }
    }
}

impl<'a> Iterator for AtomizedItemIter<'a> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Array(iter) => iter.next(),
            Self::Erroring(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomizedNodeIter {
    typed_value: Vec<atomic::Atomic>,
    typed_value_index: usize,
}

impl AtomizedNodeIter {
    fn new(node: xot::Node, xot: &Xot) -> Self {
        Self {
            typed_value: typed_value(xot, node),
            typed_value_index: 0,
        }
    }
}

fn typed_value(xot: &Xot, node: xot::Node) -> Vec<atomic::Atomic> {
    // for now we don't know any types of nodes yet; everything is untyped
    let s = xot.string_value(node);
    vec![atomic::Atomic::Untyped(Rc::from(s))]
}

impl Iterator for AtomizedNodeIter {
    type Item = atomic::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        if self.typed_value_index < self.typed_value.len() {
            let item = self.typed_value[self.typed_value_index].clone();
            self.typed_value_index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct AtomizedArrayIter<'a> {
    xot: &'a Xot,
    array: function::Array,
    array_index: usize,
    iter: Option<Box<stack::AtomizedIter<'a>>>,
}

impl<'a> AtomizedArrayIter<'a> {
    fn new(array: function::Array, xot: &'a Xot) -> Self {
        Self {
            xot,
            array,
            array_index: 0,
            iter: None,
        }
    }
}

impl<'a> Iterator for AtomizedArrayIter<'a> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if there there are any more atoms in this array entry,
            // supply those
            if let Some(iter) = &mut self.iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                } else {
                    self.iter = None;
                }
            }
            let array = &self.array.0;
            // if we're at the end of the array, we're done
            if self.array_index >= array.len() {
                return None;
            }
            let sequence = array[self.array_index].clone();
            self.array_index += 1;

            self.iter = Some(Box::new(sequence.atomized(self.xot)));
        }
    }
}

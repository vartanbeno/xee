use ordered_float::OrderedFloat;
use std::rc::Rc;

use crate::stack;
use crate::xml;

impl From<stack::Item> for stack::Value {
    fn from(item: stack::Item) -> stack::Value {
        stack::Value::from_item(item)
    }
}

use std::rc::Rc;
use std::vec;

use xot::Xot;

use crate::data::atomic::Atomic;

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

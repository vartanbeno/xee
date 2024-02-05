use std::rc::Rc;
use std::vec;

use xot::Xot;

use xee_name::Name;
use xee_schema_type::Xs;

use crate::atomic;
use crate::string::Collation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Node {
    Xot(xot::Node),
    Attribute(xot::Node, xot::NameId),
    Namespace(xot::Node, xot::PrefixId),
}

impl Node {
    #[inline]
    pub(crate) fn parent(&self, xot: &Xot) -> Option<Node> {
        match self {
            Node::Xot(node) => xot.parent(*node).map(Self::Xot),
            Node::Attribute(node, _) => Some(Self::Xot(*node)),
            Node::Namespace(..) => None,
        }
    }

    #[inline]
    pub(crate) fn is_element(&self, xot: &Xot) -> bool {
        match self {
            Node::Xot(node) => xot.is_element(*node),
            Node::Attribute(..) => false,
            Node::Namespace(..) => false,
        }
    }

    #[inline]
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

    pub(crate) fn node_name_id(&self, xot: &Xot) -> Option<xot::NameId> {
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

    pub(crate) fn node_name(&self, xot: &Xot) -> Option<Name> {
        let name_id = self.node_name_id(xot)?;
        Some(Name::from_xot(name_id, xot))
    }

    pub(crate) fn local_name(&self, xot: &Xot) -> String {
        if let Some(name) = self.node_name_id(xot) {
            let (local_name, _uri) = xot.name_ns_str(name);
            local_name.to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn namespace_uri(&self, xot: &Xot) -> String {
        if let Some(name) = self.node_name_id(xot) {
            let (_local_name, uri) = xot.name_ns_str(name);
            uri.to_string()
        } else {
            String::new()
        }
    }

    pub(crate) fn typed_value(&self, xot: &Xot) -> Vec<atomic::Atomic> {
        // for now we don't know any types of nodes yet
        let s = self.string_value(xot);
        vec![atomic::Atomic::Untyped(Rc::new(s))]
    }

    pub(crate) fn type_annotation(&self) -> Xs {
        // for now we don't know any types of nodes yet
        Xs::UntypedAtomic
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

    pub(crate) fn deep_equal(&self, other: &Node, collation: &Collation, xot: &Xot) -> bool {
        // https://www.w3.org/TR/xpath-functions-31/#func-deep-equal
        match (self, other) {
            (Node::Xot(a), Node::Xot(b)) => Self::deep_equal_xot(a, b, xot, collation),
            (Node::Attribute(a, a_name), Node::Attribute(b, b_name)) => {
                if a_name != b_name {
                    return false;
                }
                let a_element = xot.element(*a).unwrap();
                let a_value = a_element.get_attribute(*a_name).unwrap().to_string();
                let b_element = xot.element(*b).unwrap();
                let b_value = b_element.get_attribute(*b_name).unwrap().to_string();
                collation.compare(&a_value, &b_value).is_eq()
            }
            _ => false,
        }
    }

    fn deep_equal_xot(a: &xot::Node, b: &xot::Node, xot: &Xot, collation: &Collation) -> bool {
        // the top level comparison needs to compare the node, even if processing instruction or a comment, though for elements,
        // we want to compare the structure and filter comments and processing instructions out.
        use xot::ValueType::*;
        match (xot.value_type(*a), xot.value_type(*b)) {
            (Element, Element) | (Root, Root) => xot.advanced_compare(
                *a,
                *b,
                |node| xot.is_element(node) || xot.is_text(node),
                |a, b| collation.compare(a, b).is_eq(),
            ),
            (Text, Text) => {
                let a = xot.text_str(*a).unwrap();
                let b = xot.text_str(*b).unwrap();
                collation.compare(a, b).is_eq()
            }
            (Comment, Comment) => {
                let a = xot.comment_str(*a).unwrap();
                let b = xot.comment_str(*b).unwrap();
                collation.compare(a, b).is_eq()
            }
            (ProcessingInstruction, ProcessingInstruction) => {
                let a = xot.processing_instruction(*a).unwrap();
                let b = xot.processing_instruction(*b).unwrap();
                let a_data = a.data();
                let b_data = b.data();
                if a.target() != b.target() {
                    return false;
                }
                match (a_data, b_data) {
                    (Some(a_data), Some(b_data)) => collation.compare(a_data, b_data).is_eq(),
                    (None, None) => true,
                    _ => false,
                }
            }
            _ => false,
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

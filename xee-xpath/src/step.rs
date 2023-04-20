use xot::{ValueType, Xot};

use crate::ast;
use crate::value::{Item, Node, Sequence, Step};

pub(crate) fn resolve_step(step: &Step, node: Node, xot: &Xot) -> Sequence {
    let mut new_sequence = Sequence::new();
    for axis_node in node_take_axis(&step.axis, xot, node) {
        if node_test(&step.node_test, &step.axis, xot, axis_node) {
            new_sequence.push(&Item::Node(axis_node));
        }
    }
    new_sequence
}

fn node_take_axis<'a>(
    axis: &ast::Axis,
    xot: &'a Xot,
    node: Node,
) -> Box<dyn Iterator<Item = Node> + 'a> {
    match axis {
        ast::Axis::Child => Box::new(xot.children(node.xot_node()).map(Node::Node)),
        ast::Axis::Descendant => {
            let mut descendants = xot.descendants(node.xot_node());
            // consume the self descendant
            descendants.next();
            Box::new(descendants.map(Node::Node))
        }
        ast::Axis::Parent => {
            let parent_node = match node {
                Node::Node(node) => xot.parent(node),
                Node::Attribute(node, _) => Some(node),
                Node::Namespace(..) => None,
            };
            Box::new(parent_node.into_iter().map(Node::Node))
        }
        ast::Axis::Ancestor => {
            let parent_node = match node {
                Node::Node(node) => xot.parent(node),
                Node::Attribute(node, _) => Some(node),
                Node::Namespace(..) => None,
            };
            if let Some(parent_node) = parent_node {
                let mut ancestors = xot.ancestors(parent_node);
                // consume the self ancestor
                ancestors.next();
                Box::new(ancestors.map(Node::Node))
            } else {
                Box::new(std::iter::empty())
            }
        }
        ast::Axis::FollowingSibling => {
            let mut siblings = xot.following_siblings(node.xot_node());
            // consume the self sibling
            siblings.next();
            Box::new(siblings.map(Node::Node))
        }
        ast::Axis::PrecedingSibling => {
            let mut siblings = xot.preceding_siblings(node.xot_node());
            // consume the self sibling
            siblings.next();
            Box::new(siblings.map(Node::Node))
        }
        ast::Axis::Following => {
            todo!("following not supported yet")
        }
        ast::Axis::Preceding => {
            todo!("preceding not supported yet");
        }
        ast::Axis::Attribute => {
            let xot_node = node.xot_node();
            let element = xot.element(xot_node);
            if let Some(element) = element {
                Box::new(
                    element
                        .attributes()
                        .keys()
                        .map(move |name| Node::Attribute(xot_node, *name)),
                )
            } else {
                Box::new(std::iter::empty())
            }
        }
        ast::Axis::Namespace => {
            // namespaces aren't Node in Xot either
            todo!("namespaces not supported yet");
        }
        ast::Axis::Self_ => {
            let vec = vec![node];
            Box::new(vec.into_iter())
        }
        ast::Axis::DescendantOrSelf => Box::new(xot.descendants(node.xot_node()).map(Node::Node)),
        ast::Axis::AncestorOrSelf => Box::new(xot.ancestors(node.xot_node()).map(Node::Node)),
    }
}

fn node_test(node_test: &ast::NodeTest, axis: &ast::Axis, xot: &Xot, node: Node) -> bool {
    match node_test {
        ast::NodeTest::KindTest(kind_test) => {
            todo!("kind test not implemented yet")
        }
        ast::NodeTest::NameTest(name_test) => {
            if node_kind(xot, node) != principal_node_kind(axis) {
                return false;
            }
            match name_test {
                ast::NameTest::Name(name) => {
                    let name_id = ast_name_to_name_id(xot, name);
                    // if name isn't present in XML document it's certainly
                    // false
                    if let Some(name_id) = name_id {
                        match node {
                            Node::Node(node) => {
                                if let Some(element) = xot.element(node) {
                                    element.name() == name_id
                                } else {
                                    false
                                }
                            }
                            Node::Attribute(_, attr_name) => attr_name == name_id,
                            Node::Namespace(..) => false,
                        }
                    } else {
                        false
                    }
                }
                ast::NameTest::Star => true,
                ast::NameTest::LocalName(local_name) => match node {
                    Node::Node(node) => {
                        if let Some(element) = xot.element(node) {
                            let name_id = element.name();
                            let (_, name_str) = xot.name_ns_str(name_id);
                            name_str == local_name
                        } else {
                            false
                        }
                    }
                    Node::Attribute(_, attr_name) => {
                        let (_, name_str) = xot.name_ns_str(attr_name);
                        name_str == local_name
                    }
                    Node::Namespace(..) => false,
                },
                ast::NameTest::Namespace(uri) => match node {
                    Node::Node(node) => {
                        if let Some(element) = xot.element(node) {
                            let name_id = element.name();
                            let (namespace_str, _) = xot.name_ns_str(name_id);
                            namespace_str == uri
                        } else {
                            false
                        }
                    }
                    Node::Attribute(_, attr_name) => {
                        let (namespace_str, _) = xot.name_ns_str(attr_name);
                        namespace_str == uri
                    }
                    Node::Namespace(..) => false,
                },
            }
        }
    }
}

fn ast_name_to_name_id(xot: &Xot, name: &ast::Name) -> Option<xot::NameId> {
    if let Some(namespace) = &name.namespace {
        let namespace_id = xot.namespace(namespace);
        if let Some(namespace_id) = namespace_id {
            xot.name_ns(&name.name, namespace_id)
        } else {
            None
        }
    } else {
        xot.name(&name.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrincipalNodeKind {
    Element,
    Attribute,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Document,
    Element,
    Attribute,
    Text,
    Namespace,
    ProcessingInstruction,
    Comment,
}

fn node_kind(xot: &Xot, node: Node) -> NodeKind {
    match node {
        Node::Node(node) => {
            let node = xot.value_type(node);
            match node {
                ValueType::Element => NodeKind::Element,
                ValueType::Text => NodeKind::Text,
                ValueType::ProcessingInstruction => NodeKind::ProcessingInstruction,
                ValueType::Comment => NodeKind::Comment,
                ValueType::Root => NodeKind::Document,
            }
        }
        Node::Attribute(..) => NodeKind::Attribute,
        Node::Namespace(..) => NodeKind::Namespace,
    }
}

fn principal_node_kind(axis: &ast::Axis) -> NodeKind {
    match axis {
        ast::Axis::Attribute => NodeKind::Attribute,
        ast::Axis::Namespace => NodeKind::Namespace,
        _ => NodeKind::Element,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xot_nodes_to_sequence(node: &[xot::Node]) -> Sequence {
        Sequence {
            items: node
                .iter()
                .map(|&node| Item::Node(Node::Node(node)))
                .collect(),
        }
    }

    #[test]
    fn test_child_axis_star() -> Result<(), xot::Error> {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc)?;
        let a = xot.first_child(doc_el).unwrap();
        let b = xot.next_sibling(a).unwrap();

        let step = Step {
            axis: ast::Axis::Child,
            node_test: ast::NodeTest::NameTest(ast::NameTest::Star),
        };
        let sequence = resolve_step(&step, Node::Node(doc_el), &xot);
        assert_eq!(sequence, xot_nodes_to_sequence(&[a, b]));
        Ok(())
    }

    #[test]
    fn test_child_axis_name() -> Result<(), xot::Error> {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc)?;
        let a = xot.first_child(doc_el).unwrap();

        let step = Step {
            axis: ast::Axis::Child,
            node_test: ast::NodeTest::NameTest(ast::NameTest::Name(ast::Name {
                name: "a".to_string(),
                namespace: None,
            })),
        };
        let sequence = resolve_step(&step, Node::Node(doc_el), &xot);
        assert_eq!(sequence, xot_nodes_to_sequence(&[a]));
        Ok(())
    }
}

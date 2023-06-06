use xot::{ValueType, Xot};

use xee_xpath_ast::ast;

use crate::data::{InnerSequence, Item, Node, Sequence, Step};

pub(crate) fn resolve_step(step: &Step, node: Node, xot: &Xot) -> Sequence {
    let mut new_sequence = InnerSequence::new();
    for axis_node in node_take_axis(&step.axis, xot, node) {
        if node_test(&step.node_test, &step.axis, xot, axis_node) {
            new_sequence.push(&Item::Node(axis_node));
        }
    }
    Sequence::new(new_sequence)
}

fn node_take_axis<'a>(
    axis: &ast::Axis,
    xot: &'a Xot,
    node: Node,
) -> Box<dyn Iterator<Item = Node> + 'a> {
    match axis {
        ast::Axis::Child => node.xot_iterator(|n| xot.children(n)),
        ast::Axis::Descendant => node.xot_iterator(|n| {
            let mut descendants = xot.descendants(n);
            // since this includes self we get rid of it here
            descendants.next();
            descendants
        }),
        ast::Axis::Parent => {
            let parent_node = node.parent(xot);
            Box::new(parent_node.into_iter())
        }
        ast::Axis::Ancestor => {
            let parent_node = node.parent(xot);
            // the ancestors of the parents include self, which is
            // what we want as the parent is already taken
            // We can't get a Node::Attribute or Node::Namespace
            // because we just took the parent
            parent_node.map_or(Box::new(std::iter::empty()), |node| {
                node.xot_iterator(|n| xot.ancestors(n))
            })
        }
        ast::Axis::FollowingSibling => node.xot_iterator(|n| {
            let mut siblings = xot.following_siblings(n);
            // consume the self sibling
            siblings.next();
            siblings
        }),
        ast::Axis::PrecedingSibling => node.xot_iterator(|n| {
            let mut siblings = xot.preceding_siblings(n);
            // consume the self sibling
            siblings.next();
            siblings
        }),
        ast::Axis::Following => {
            todo!("following not supported yet")
        }
        ast::Axis::Preceding => {
            todo!("preceding not supported yet");
        }
        ast::Axis::Attribute => match node {
            Node::Xot(node) => {
                let element = xot.element(node);
                if let Some(element) = element {
                    Box::new(
                        element
                            .attributes()
                            .keys()
                            .map(move |name| Node::Attribute(node, *name)),
                    )
                } else {
                    Box::new(std::iter::empty())
                }
            }
            Node::Attribute(..) | Node::Namespace(..) => Box::new(std::iter::empty()),
        },
        ast::Axis::Namespace => {
            // namespaces aren't Node in Xot either
            todo!("namespaces not supported yet");
        }
        ast::Axis::Self_ => {
            let vec = vec![node];
            Box::new(vec.into_iter())
        }
        ast::Axis::DescendantOrSelf => node.xot_iterator(|n| xot.descendants(n)),
        ast::Axis::AncestorOrSelf => node.xot_iterator(|n| xot.ancestors(n)),
    }
}

fn node_test(node_test: &ast::NodeTest, axis: &ast::Axis, xot: &Xot, node: Node) -> bool {
    match node_test {
        ast::NodeTest::KindTest(kind_test) => match kind_test {
            ast::KindTest::Any => true,
            ast::KindTest::Text => {
                if let Node::Xot(node) = node {
                    xot.value_type(node) == ValueType::Text
                } else {
                    false
                }
            }
            ast::KindTest::Comment => {
                if let Node::Xot(node) = node {
                    xot.value_type(node) == ValueType::Comment
                } else {
                    false
                }
            }
            _ => {
                todo!("kind test not implemented yet {:?}", kind_test);
            }
        },
        ast::NodeTest::NameTest(name_test) => {
            if node_kind(xot, node) != principal_node_kind(axis) {
                return false;
            }
            match name_test {
                ast::NameTest::Name(name) => {
                    let name_id = name.to_name_id(xot);
                    // if name isn't present in XML document it's certainly
                    // false
                    if let Some(name_id) = name_id {
                        match node {
                            Node::Xot(node) => {
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
                    Node::Xot(node) => {
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
                    Node::Xot(node) => {
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
        Node::Xot(node) => {
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
        Sequence::new(InnerSequence {
            items: node
                .iter()
                .map(|&node| Item::Node(Node::Xot(node)))
                .collect(),
        })
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
        let sequence = resolve_step(&step, Node::Xot(doc_el), &xot);
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
            node_test: ast::NodeTest::NameTest(ast::NameTest::Name(ast::Name::without_ns("a"))),
        };
        let sequence = resolve_step(&step, Node::Xot(doc_el), &xot);
        assert_eq!(sequence, xot_nodes_to_sequence(&[a]));
        Ok(())
    }
}

use xot::{ValueType, Xot};

use xee_xpath_ast::ast;

use crate::sequence;
use crate::stack;
use crate::xml;

use super::kind_test::kind_test;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub(crate) axis: ast::Axis,
    pub(crate) node_test: ast::NodeTest,
}

pub(crate) fn resolve_step(step: &Step, node: xml::Node, xot: &Xot) -> stack::Value {
    let mut new_items = Vec::new();
    for axis_node in node_take_axis(&step.axis, xot, node) {
        if node_test(&step.node_test, &step.axis, xot, axis_node) {
            new_items.push(sequence::Item::Node(axis_node));
        }
    }
    new_items.into()
}

fn node_take_axis<'a>(
    axis: &ast::Axis,
    xot: &'a Xot,
    node: xml::Node,
) -> Box<dyn Iterator<Item = xml::Node> + 'a> {
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
        ast::Axis::Following =>
        // TODO: bad implementation so we don't get a crash
        {
            node.xot_iterator(|n| xot.children(n))
        }
        ast::Axis::Preceding =>
        // TODO: bad implementation so we don't get a crash
        {
            node.xot_iterator(|n| xot.children(n))
        }
        ast::Axis::Attribute => match node {
            xml::Node::Xot(node) => {
                let element = xot.element(node);
                if let Some(element) = element {
                    Box::new(
                        element
                            .attributes()
                            .keys()
                            .map(move |name| xml::Node::Attribute(node, *name)),
                    )
                } else {
                    Box::new(std::iter::empty())
                }
            }
            xml::Node::Attribute(..) | xml::Node::Namespace(..) => Box::new(std::iter::empty()),
        },
        ast::Axis::Namespace => {
            // namespaces aren't xml::Node in Xot either
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

fn node_test(node_test: &ast::NodeTest, axis: &ast::Axis, xot: &Xot, node: xml::Node) -> bool {
    match node_test {
        ast::NodeTest::KindTest(kt) => kind_test(kt, xot, node),
        ast::NodeTest::NameTest(name_test) => {
            if node_kind(xot, node) != principal_node_kind(axis) {
                return false;
            }
            match name_test {
                ast::NameTest::Name(name) => {
                    let name_id = name.value.to_name_id(xot);
                    // if name isn't present in XML document it's certainly
                    // false
                    if let Some(name_id) = name_id {
                        match node {
                            xml::Node::Xot(node) => {
                                if let Some(element) = xot.element(node) {
                                    element.name() == name_id
                                } else {
                                    false
                                }
                            }
                            xml::Node::Attribute(_, attr_name) => attr_name == name_id,
                            xml::Node::Namespace(..) => false,
                        }
                    } else {
                        false
                    }
                }
                ast::NameTest::Star => true,
                ast::NameTest::LocalName(local_name) => match node {
                    xml::Node::Xot(node) => {
                        if let Some(element) = xot.element(node) {
                            let name_id = element.name();
                            let (_, name_str) = xot.name_ns_str(name_id);
                            name_str == local_name
                        } else {
                            false
                        }
                    }
                    xml::Node::Attribute(_, attr_name) => {
                        let (_, name_str) = xot.name_ns_str(attr_name);
                        name_str == local_name
                    }
                    xml::Node::Namespace(..) => false,
                },
                ast::NameTest::Namespace(uri) => match node {
                    xml::Node::Xot(node) => {
                        if let Some(element) = xot.element(node) {
                            let name_id = element.name();
                            let (namespace_str, _) = xot.name_ns_str(name_id);
                            namespace_str == uri
                        } else {
                            false
                        }
                    }
                    xml::Node::Attribute(_, attr_name) => {
                        let (namespace_str, _) = xot.name_ns_str(attr_name);
                        namespace_str == uri
                    }
                    xml::Node::Namespace(..) => false,
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

fn node_kind(xot: &Xot, node: xml::Node) -> NodeKind {
    match node {
        xml::Node::Xot(node) => {
            let node = xot.value_type(node);
            match node {
                ValueType::Element => NodeKind::Element,
                ValueType::Text => NodeKind::Text,
                ValueType::ProcessingInstruction => NodeKind::ProcessingInstruction,
                ValueType::Comment => NodeKind::Comment,
                ValueType::Root => NodeKind::Document,
            }
        }
        xml::Node::Attribute(..) => NodeKind::Attribute,
        xml::Node::Namespace(..) => NodeKind::Namespace,
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
    use xee_xpath_ast::{ast, WithSpan};

    use super::*;

    fn xot_nodes_to_value(node: &[xot::Node]) -> stack::Value {
        node.iter()
            .map(|&node| sequence::Item::Node(xml::Node::Xot(node)))
            .collect::<Vec<_>>()
            .into()
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
        let value = resolve_step(&step, xml::Node::Xot(doc_el), &xot);
        assert_eq!(value, xot_nodes_to_value(&[a, b]));
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
            node_test: ast::NodeTest::NameTest(ast::NameTest::Name(
                ast::Name::unprefixed("a").with_empty_span(),
            )),
        };
        let value = resolve_step(&step, xml::Node::Xot(doc_el), &xot);
        assert_eq!(value, xot_nodes_to_value(&[a]));
        Ok(())
    }
}

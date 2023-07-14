use xot::{ValueType, Xot};

use xee_xpath_ast::ast;

use crate::xml;

pub(crate) fn kind_test(kind_test: &ast::KindTest, xot: &Xot, node: xml::Node) -> bool {
    match kind_test {
        ast::KindTest::Document(dt) => {
            if let xml::Node::Xot(node) = node {
                if let Some(_document_test) = dt {
                    todo!();
                } else {
                    xot.value_type(node) == ValueType::Root
                }
            } else {
                false
            }
        }
        ast::KindTest::Element(et) => {
            if let xml::Node::Xot(node) = node {
                if let Some(et) = et {
                    element_test(et, xot, node)
                } else {
                    xot.value_type(node) == ValueType::Element
                }
            } else {
                false
            }
        }
        ast::KindTest::Any => true,
        ast::KindTest::Text => {
            if let xml::Node::Xot(node) = node {
                xot.value_type(node) == ValueType::Text
            } else {
                false
            }
        }
        ast::KindTest::Comment => {
            if let xml::Node::Xot(node) = node {
                xot.value_type(node) == ValueType::Comment
            } else {
                false
            }
        }
        _ => {
            todo!("kind test not implemented yet {:?}", kind_test);
        }
    }
}

fn element_test(element_test: &ast::ElementTest, xot: &Xot, node: xot::Node) -> bool {
    let name_matches = match &element_test.element_name_or_wildcard {
        ast::ElementNameOrWildcard::Name(name) => {
            if let Some(element) = xot.element(node) {
                let name_id = name_id_for_name(xot, &name.value);
                Some(element.name()) == name_id
            } else {
                false
            }
        }
        ast::ElementNameOrWildcard::Wildcard => xot.value_type(node) == ValueType::Element,
    };
    if !name_matches {
        return false;
    }
    if let Some(_type_name) = &element_test.type_name {
        todo!();
    } else {
        true
    }
}

fn name_id_for_name(xot: &Xot, name: &ast::Name) -> Option<xot::NameId> {
    if let Some(namespace) = name.namespace() {
        let ns = xot.namespace(namespace);
        if let Some(ns) = ns {
            xot.name_ns(name.local_name(), ns)
        } else {
            None
        }
    } else {
        xot.name(name.local_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrincipalNodeKind {
    Element,
    Attribute,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Document,
    Element,
    Attribute,
    Text,
    Namespace,
    ProcessingInstruction,
    Comment,
}

pub(crate) fn node_kind(xot: &Xot, node: xml::Node) -> NodeKind {
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

pub(crate) fn principal_node_kind(axis: &ast::Axis) -> NodeKind {
    match axis {
        ast::Axis::Attribute => NodeKind::Attribute,
        ast::Axis::Namespace => NodeKind::Namespace,
        _ => NodeKind::Element,
    }
}

#[cfg(test)]
mod tests {
    use xee_xpath_ast::ast;

    use super::*;

    #[test]
    fn test_kind_test_any() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();

        let kt = ast::KindTest::parse("node()").unwrap();
        assert!(kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a)));
    }

    #[test]
    fn test_kind_test_text() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a>content</a><b/></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let a_text = xot.first_child(a).unwrap();

        let kt = ast::KindTest::parse("text()").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(a)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a_text)));
    }

    #[test]
    fn test_kind_test_comment() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><!-- comment --></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let comment = xot.first_child(doc_el).unwrap();

        let kt = ast::KindTest::parse("comment()").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(comment)));
    }

    #[test]
    fn test_kind_test_document() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let kt = ast::KindTest::parse("document-node()").unwrap();
        assert!(kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
    }

    #[test]
    fn test_kind_test_element_without_name() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let text = xot.first_child(a).unwrap();

        let kt = ast::KindTest::parse("element()").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(text)));
    }

    #[test]
    fn test_kind_test_with_wildcard() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let text = xot.first_child(a).unwrap();

        let kt = ast::KindTest::parse("element(*)").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(text)));
    }

    #[test]
    fn test_kind_test_with_name() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let text = xot.first_child(a).unwrap();

        let kt = ast::KindTest::parse("element(a)").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(text)));
    }
}

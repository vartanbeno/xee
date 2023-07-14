use xee_schema_type::Xs;
use xot::{ValueType, Xot};

use xee_xpath_ast::ast;

use crate::xml;

pub(crate) fn kind_test(kind_test: &ast::KindTest, xot: &Xot, node: xml::Node) -> bool {
    match kind_test {
        ast::KindTest::Document(dt) => {
            if let xml::Node::Xot(node) = node {
                if let Some(_document_test) = dt {
                    // document-node(E) matches any document node that contains
                    // exactly one element node, optionally accompanied by one or more
                    // comment or processing nodes, and E is an ElementTest or SchemaElementTest
                    // that matches the element node
                    todo!();
                } else {
                    // document-node() matches any document node
                    xot.value_type(node) == ValueType::Root
                }
            } else {
                false
            }
        }
        ast::KindTest::Element(et) => element_test(et.as_ref(), xot, node),
        ast::KindTest::SchemaElement(set) => {
            todo!()
        }
        ast::KindTest::Attribute(at) => {
            todo!()
        }
        ast::KindTest::SchemaAttribute(sat) => {
            todo!()
        }
        ast::KindTest::Any => true,
        // text() matches any text node
        ast::KindTest::Text => {
            if let xml::Node::Xot(node) = node {
                xot.value_type(node) == ValueType::Text
            } else {
                false
            }
        }
        // comment() matches any comment node
        ast::KindTest::Comment => {
            if let xml::Node::Xot(node) = node {
                xot.value_type(node) == ValueType::Comment
            } else {
                false
            }
        }
        ast::KindTest::NamespaceNode => {
            // namespace-node() matches any namespace node
            todo!();
        }
        ast::KindTest::PI(pi_test) => {
            if let xml::Node::Xot(node) = node {
                if let Some(_pi_test) = pi_test {
                    // processing-instruction N matches any processing-instruction node
                    // whose PITarget is equal to fn:normalize-space(N)
                    todo!();
                } else {
                    // processing-instruction() matches any processing-instruction node
                    xot.value_type(node) == ValueType::ProcessingInstruction
                }
            } else {
                false
            }
        }
    }
}

fn element_test(test: Option<&ast::ElementOrAttributeTest>, xot: &Xot, node: xml::Node) -> bool {
    element_or_attribute_test(test, xot, node, |node, xot| {
        if let xml::Node::Xot(node) = node {
            xot.value_type(node) == ValueType::Element
        } else {
            false
        }
    })
}

fn attribute_test(test: Option<&ast::ElementOrAttributeTest>, xot: &Xot, node: xml::Node) -> bool {
    element_or_attribute_test(test, xot, node, |node, _| {
        matches!(node, xml::Node::Attribute(_, _))
    })
}

fn element_or_attribute_test(
    test: Option<&ast::ElementOrAttributeTest>,
    xot: &Xot,
    node: xml::Node,
    node_type_match: impl Fn(xml::Node, &Xot) -> bool,
) -> bool {
    // if we're not the right node type (element, or attribute) then we
    // bail out
    if !node_type_match(node, xot) {
        return false;
    }

    if let Some(test) = test {
        // the name has to match first
        let name_matches = match &test.name_or_wildcard {
            ast::NameOrWildcard::Name(name) => {
                if let Some(node_name) = node.node_name(xot) {
                    let name_id = name_id_for_name(xot, &name.value);
                    Some(node_name) == name_id
                } else {
                    false
                }
            }
            ast::NameOrWildcard::Wildcard => true,
        };
        if !name_matches {
            return false;
        }
        // the type also has to match
        if let Some(type_name) = &test.type_name {
            // derives-from(type-annotation, TypeName) must be true
            let name = &type_name.name.value;
            let type_ = Xs::by_name(name.namespace(), name.local_name());
            if let Some(type_) = type_ {
                node.type_annotation().derives_from(type_)
            } else {
                // unknown type
                false
            }
            // ignoring can_be_nilled for now
        } else {
            true
        }
    } else {
        // there is further test, so we're done
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
    fn test_kind_test_element_with_wildcard() {
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
    fn test_kind_test_element_with_name() {
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

    #[test]
    fn test_kind_test_element_with_type_name() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a>text</a></root>"#).unwrap();
        let doc_el = xot.document_element(doc).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let text = xot.first_child(a).unwrap();

        let kt = ast::KindTest::parse("element(a, xs:untypedAtomic)").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(doc_el)));
        assert!(kind_test(&kt, &xot, xml::Node::Xot(a)));
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(text)));

        // but we're not an xs:string
        let kt = ast::KindTest::parse("element(a, xs:string)").unwrap();
        assert!(!kind_test(&kt, &xot, xml::Node::Xot(a)));
    }
}

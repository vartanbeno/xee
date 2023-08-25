use ::next_gen::prelude::*;
use ahash::{HashMap, HashMapExt};
use xot::Xot;

use crate::xml;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub(crate) struct DocumentOrder(usize, usize);

impl DocumentOrder {
    pub(crate) fn generate_id(&self) -> String {
        format!("id_{}_{}", self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Annotation {
    document_order: DocumentOrder,
}

impl Annotation {
    pub(crate) fn generate_id(&self) -> String {
        self.document_order.generate_id()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Annotations {
    // each document has a different id, so track this
    document_id: usize,
    map: HashMap<xml::Node, Annotation>,
}

impl Annotations {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
            document_id: 0,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.document_id = 0;
    }

    pub(crate) fn add(&mut self, xot: &Xot, doc: xml::Node) {
        // if we already know this document, we are done
        if self.map.contains_key(&doc) {
            return;
        }
        mk_gen!(let gen = document_descendants(xot, doc));
        let map_init = gen.enumerate().map(|(i, node)| {
            (
                node,
                Annotation {
                    document_order: DocumentOrder(self.document_id, i),
                },
            )
        });
        self.map.extend(map_init);
        self.document_id += 1;
    }

    pub(crate) fn get(&self, node: xml::Node) -> Option<&Annotation> {
        self.map.get(&node)
    }

    pub(crate) fn document_order(&self, node: xml::Node) -> DocumentOrder {
        self.get(node)
            .map(|annotation| annotation.document_order)
            .expect("node not found")
    }
}

#[generator(yield(xml::Node))]
fn document_descendants(xot: &Xot, doc: xml::Node) {
    match doc {
        xml::Node::Xot(node) => {
            if !xot.is_root(node) {
                panic!("node is not a document");
            }
            for descendant in xot.descendants(node) {
                yield_!(xml::Node::Xot(descendant));
                if let Some(element) = xot.element(descendant) {
                    for prefix in element.prefixes().keys() {
                        yield_!(xml::Node::Namespace(descendant, *prefix));
                    }
                    for attr_name in element.attributes().keys() {
                        yield_!(xml::Node::Attribute(descendant, *attr_name));
                    }
                }
            }
        }
        xml::Node::Attribute(..) | xml::Node::Namespace(..) => {
            panic!("node is not a document");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_document() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let root = xot.document_element(doc).unwrap();
        let a = xot.first_child(root).unwrap();
        let b = xot.next_sibling(a).unwrap();
        let root = xml::Node::Xot(root);
        let a = xml::Node::Xot(a);
        let b = xml::Node::Xot(b);
        let mut annotations = Annotations::new();
        annotations.add(&xot, xml::Node::Xot(doc));

        assert!(annotations.document_order(a) < annotations.document_order(b));
        assert!(annotations.document_order(root) < annotations.document_order(a));
    }

    #[test]
    fn test_multiple_documents() {
        let mut xot = Xot::new();
        let doc0 = xot.parse(r#"<root><a/><b/></root>"#).unwrap();
        let root0 = xot.document_element(doc0).unwrap();
        let a = xot.first_child(root0).unwrap();
        let b = xot.next_sibling(a).unwrap();
        let root0 = xml::Node::Xot(root0);
        let a = xml::Node::Xot(a);
        let b = xml::Node::Xot(b);

        let doc1 = xot.parse(r#"<root><c/><d/></root>"#).unwrap();
        let root1 = xot.document_element(doc1).unwrap();
        let c = xot.first_child(root1).unwrap();
        let d = xot.next_sibling(c).unwrap();
        let root1 = xml::Node::Xot(root1);
        let c = xml::Node::Xot(c);
        let d = xml::Node::Xot(d);

        let mut annotations = Annotations::new();
        annotations.add(&xot, xml::Node::Xot(doc0));
        annotations.add(&xot, xml::Node::Xot(doc1));

        assert!(annotations.document_order(a) < annotations.document_order(b));
        assert!(annotations.document_order(root0) < annotations.document_order(a));
        assert!(annotations.document_order(c) < annotations.document_order(d));
        assert!(annotations.document_order(root1) < annotations.document_order(c));
        assert!(annotations.document_order(root0) < annotations.document_order(root1));
        assert!(annotations.document_order(a) < annotations.document_order(c));
    }

    #[test]
    fn test_attributes() {
        let mut xot = Xot::new();
        let doc = xot
            .parse(r#"<root><x a="1" b="2"/><y c="3"/></root>"#)
            .unwrap();
        let root = xot.document_element(doc).unwrap();
        let x = xot.first_child(root).unwrap();
        let y = xot.next_sibling(x).unwrap();
        let a = xml::Node::Attribute(x, xot.name("a").unwrap());
        let b = xml::Node::Attribute(x, xot.name("b").unwrap());
        let c = xml::Node::Attribute(y, xot.name("c").unwrap());
        let x = xml::Node::Xot(x);
        let y = xml::Node::Xot(y);

        let mut annotations = Annotations::new();
        annotations.add(&xot, xml::Node::Xot(doc));

        assert!(annotations.document_order(a) < annotations.document_order(b));
        assert!(annotations.document_order(x) < annotations.document_order(a));
        assert!(annotations.document_order(y) < annotations.document_order(c));
        assert!(annotations.document_order(b) < annotations.document_order(y));
    }
}

use ahash::{HashMap, HashMapExt};
use xot::Xot;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub(crate) struct DocumentOrder(usize, usize);

impl DocumentOrder {
    pub(crate) fn generate_id(&self) -> String {
        format!("id_{}_{}", self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Annotation {
    pub(crate) document_order: DocumentOrder,
}

impl Annotation {
    pub(crate) fn generate_id(&self) -> String {
        self.document_order.generate_id()
    }
}

#[derive(Debug, Clone)]
pub struct Annotations {
    // each document has a different id, so track this
    document_id: usize,
    map: HashMap<xot::Node, Annotation>,
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

    pub(crate) fn add(&mut self, xot: &Xot, doc: xot::Node) {
        // if we already know this document, we are done
        if self.map.contains_key(&doc) {
            return;
        }
        let map_init = xot.all_descendants(doc).enumerate().map(|(i, node)| {
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

    pub(crate) fn get(&self, node: xot::Node) -> Option<&Annotation> {
        self.map.get(&node)
    }

    pub(crate) fn document_order(&self, node: xot::Node) -> DocumentOrder {
        self.get(node)
            .map(|annotation| annotation.document_order)
            .expect("node not found")
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

        let mut annotations = Annotations::new();
        annotations.add(&xot, doc);

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

        let doc1 = xot.parse(r#"<root><c/><d/></root>"#).unwrap();
        let root1 = xot.document_element(doc1).unwrap();
        let c = xot.first_child(root1).unwrap();
        let d = xot.next_sibling(c).unwrap();

        let mut annotations = Annotations::new();
        annotations.add(&xot, doc0);
        annotations.add(&xot, doc1);

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
        let a = xot.attributes(x).get_node(xot.name("a").unwrap()).unwrap();
        let b = xot.attributes(x).get_node(xot.name("b").unwrap()).unwrap();
        let c = xot.attributes(y).get_node(xot.name("c").unwrap()).unwrap();

        let mut annotations = Annotations::new();
        annotations.add(&xot, doc);

        assert!(annotations.document_order(a) < annotations.document_order(b));
        assert!(annotations.document_order(x) < annotations.document_order(a));
        assert!(annotations.document_order(y) < annotations.document_order(c));
        assert!(annotations.document_order(b) < annotations.document_order(y));
    }
}

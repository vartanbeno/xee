use ahash::{HashMap, HashMapExt};
use std::fmt::Debug;
use xot::Xot;

use super::annotation::Annotations;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Uri(pub(crate) String);

impl Uri {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) uri: Uri,
    pub root: xot::Node,
}

impl Document {
    pub fn root(&self) -> xot::Node {
        self.root
    }

    pub fn cleanup(&self, xot: &mut Xot) {
        xot.remove(self.root).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct Documents {
    pub(crate) annotations: Annotations,
    documents: HashMap<Uri, Document>,
}

impl Documents {
    pub fn new() -> Self {
        Self {
            annotations: Annotations::new(),
            documents: HashMap::new(),
        }
    }

    pub fn cleanup(&mut self, xot: &mut Xot) {
        for document in self.documents.values() {
            document.cleanup(xot);
        }
        self.annotations.clear();
        self.documents.clear();
    }

    pub fn add(&mut self, xot: &mut Xot, uri: &Uri, xml: &str) -> Result<(), xot::Error> {
        let root = xot.parse(xml)?;
        self.add_root(xot, uri, root);
        Ok(())
    }

    pub fn add_root(&mut self, xot: &Xot, uri: &Uri, root: xot::Node) {
        let document = Document {
            uri: uri.clone(),
            root,
        };
        self.documents.insert(uri.clone(), document);
        self.annotations.add(xot, root);
    }

    pub fn get(&self, uri: &Uri) -> Option<&Document> {
        self.documents.get(uri)
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

impl Default for Documents {
    fn default() -> Self {
        Self::new()
    }
}

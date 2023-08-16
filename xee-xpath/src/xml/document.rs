use ahash::{HashMap, HashMapExt};
use std::fmt::Debug;
use xot::Xot;

use crate::xml;

use super::annotation::Annotations;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Uri(pub(crate) String);

impl Uri {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) uri: Uri,
    pub root: xot::Node,
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

    pub fn add(&mut self, xot: &mut Xot, uri: &Uri, xml: &str) -> Result<(), xot::Error> {
        let root = xot.parse(xml)?;
        self.add_root(xot, uri, root);
        Ok(())
    }

    pub(crate) fn add_root(&mut self, xot: &Xot, uri: &Uri, root: xot::Node) {
        self.documents.insert(
            uri.clone(),
            Document {
                uri: uri.clone(),
                root,
            },
        );
        self.annotations.add(xot, xml::Node::Xot(root));
    }

    pub fn get(&self, uri: &Uri) -> Option<&Document> {
        self.documents.get(uri)
    }
}

impl Default for Documents {
    fn default() -> Self {
        Self::new()
    }
}

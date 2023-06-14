use ahash::{HashMap, HashMapExt};
use std::fmt::Debug;
use xot::Xot;

use crate::xml;

use super::annotation::Annotations;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Uri(pub(crate) String);

#[derive(Debug, Clone)]
pub(crate) struct Document {
    pub(crate) uri: Uri,
    pub(crate) root: xot::Node,
}

#[derive(Debug, Clone)]
pub(crate) struct Documents {
    pub(crate) annotations: Annotations,
    documents: HashMap<Uri, Document>,
}

impl Documents {
    pub(crate) fn new() -> Self {
        Self {
            annotations: Annotations::new(),
            documents: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, xot: &mut Xot, uri: &Uri, xml: &str) -> Result<(), xot::Error> {
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

    pub(crate) fn get(&self, uri: &Uri) -> Option<&Document> {
        self.documents.get(uri)
    }
}

use std::{cell::RefCell, rc::Rc};
use xot::Xot;

use xee_interpreter::xml;

use crate::{documents::Documents, queries::Queries};

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session<'namespaces> {
    pub(crate) queries: &'namespaces Queries<'namespaces>,
    pub(crate) documents: Rc<RefCell<xml::Documents>>,
    pub(crate) xot: Xot,
}

impl<'namespaces> Session<'namespaces> {
    pub(crate) fn new(queries: &'namespaces Queries, documents: Documents) -> Self {
        Self {
            queries,
            documents: Rc::new(documents.documents),
            xot: documents.xot,
        }
    }

    pub fn xot(&self) -> &Xot {
        &self.xot
    }

    pub fn xot_mut(&mut self) -> &mut Xot {
        &mut self.xot
    }
}

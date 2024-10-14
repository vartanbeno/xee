#![cfg_attr(windows, feature(abi_vectorcall))]
use std::{cell::RefCell, rc::Rc, sync::Arc};

use ext_php_rs::{
    boxed::ZBox,
    prelude::*,
    types::{ZendClassObject, ZendObject},
};

use xee_xpath::Query as XPathQuery;

#[php_class]
pub struct Documents {
    documents: Arc<RefCell<xee_xpath::Documents>>,
}

#[php_class]
pub struct DocumentHandle {
    handle: xee_xpath::DocumentHandle,
}

#[php_impl]
impl Documents {
    #[constructor]
    pub fn make_new() -> Documents {
        Documents {
            documents: Arc::new(RefCell::new(xee_xpath::Documents::new())),
        }
    }

    pub fn add_string(
        &mut self,
        uri: &str,
        content: &str,
    ) -> PhpResult<ZBox<ZendClassObject<DocumentHandle>>> {
        Ok(ZendClassObject::new(DocumentHandle {
            handle: self
                .documents
                .borrow_mut()
                .add_string(&xee_xpath::Uri::new(uri), content)
                .map_err(|e| e.to_string())?,
        }))
    }

    pub fn session(&mut self) -> ZBox<ZendClassObject<Session>> {
        ZendClassObject::new(Session {
            documents: self.documents.clone(),
        })
    }
}

#[php_class]
pub struct Queries {
    queries: Arc<xee_xpath::Queries<'static>>,
}

#[php_impl]
impl Queries {
    #[constructor]
    pub fn make_new() -> Queries {
        Queries {
            queries: Arc::new(xee_xpath::Queries::default()),
        }
    }

    pub fn sequence(&mut self, query: &str) -> PhpResult<ZBox<ZendClassObject<SequenceQuery>>> {
        Ok(ZendClassObject::new(SequenceQuery {
            query: self.queries.sequence(query).map_err(|e| e.to_string())?,
        }))
    }
}

#[php_class]
pub struct SequenceQuery {
    query: xee_xpath::query::SequenceQuery,
}

#[php_impl]
impl SequenceQuery {
    pub fn execute(
        &self,
        session: &mut ZendClassObject<Session>,
        doc: &ZendClassObject<DocumentHandle>,
    ) -> PhpResult<ZBox<ZendClassObject<Sequence>>> {
        let mut documents = session.documents.borrow_mut();
        let mut session = documents.session();
        Ok(ZendClassObject::new(Sequence {
            sequence: self
                .query
                .execute(&mut session, doc.handle)
                .map_err(|e| e.to_string())?,
        }))
    }
}

#[php_class]
pub struct Session {
    documents: Arc<RefCell<xee_xpath::Documents>>,
}

#[php_impl]
impl Session {}

#[php_class]
pub struct Sequence {
    sequence: xee_xpath::Sequence,
}

#[php_impl]
impl Sequence {
    pub fn len(&self) -> usize {
        self.sequence.len()
    }
}

#[php_function]
pub fn hello_world(name: &str) -> String {
    format!("Hello, {}?", name)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}

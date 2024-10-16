#![cfg_attr(windows, feature(abi_vectorcall))]
use std::{cell::RefCell, sync::Arc};

use ext_php_rs::{
    boxed::ZBox,
    exception::PhpResult,
    prelude::*,
    types::{ZendClassObject, Zval},
    zend::ce,
};

use xee_xpath::Query as XPathQuery;

use crate::atomic::atomic_to_zval;

#[php_class(name = "Xee\\Documents")]
pub struct Documents {
    documents: Arc<RefCell<xee_xpath::Documents>>,
}
#[php_class(name = "Xee\\DocumentHandle")]
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

#[php_class(name = "Xee\\Queries")]
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

#[php_class(name = "Xee\\SequenceQuery")]
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

#[php_class(name = "Xee\\Session")]
pub struct Session {
    documents: Arc<RefCell<xee_xpath::Documents>>,
}

#[php_impl]
impl Session {}

#[php_class(name = "Xee\\Sequence")]
#[implements(ce::arrayaccess())]
#[implements(ce::countable())]
pub struct Sequence {
    sequence: xee_xpath::Sequence,
}

#[php_impl]
impl Sequence {
    pub fn count(&self) -> usize {
        self.sequence.len()
    }

    pub fn offset_exists(&self, offset: &'_ Zval) -> bool {
        if let Some(offset) = offset.extract::<usize>() {
            offset < self.sequence.len()
        } else {
            false
        }
    }

    pub fn offset_get(&self, offset: &'_ Zval) -> PhpResult<Zval> {
        if let Some(offset) = offset.extract::<usize>() {
            let item = self.sequence.get(offset).map_err(|e| e.to_string())?;
            match item {
                xee_xpath::Item::Atomic(atomic) => Ok(atomic_to_zval(&atomic, false)?),
                xee_xpath::Item::Node(_) => todo!(),
                xee_xpath::Item::Function(_) => todo!(),
            }
        } else {
            Err("Invalid offset".into())
        }
    }

    pub fn offset_set(&mut self, _offset: &'_ Zval, _value: &'_ Zval) -> PhpResult {
        Err("Setting values for Sequence is not supported".into())
    }

    pub fn offset_unset(&mut self, _offset: &'_ Zval) -> PhpResult {
        Err("Setting values for Sequence is not supported".into())
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}

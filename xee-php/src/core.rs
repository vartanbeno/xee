#![cfg_attr(windows, feature(abi_vectorcall))]
use std::{cell::RefCell, sync::Arc};

use ext_php_rs::{
    boxed::ZBox,
    convert::IntoZval,
    exception::PhpResult,
    prelude::*,
    types::{ZendClassObject, Zval},
    zend::ce,
};

use xee_xpath::Query as XPathQuery;

use crate::atomic::atomic_to_zval;

/// Documents hold XML documents that can be queried.
#[php_class(name = "Xee\\Documents")]
pub struct Documents {
    documents: xee_xpath::Documents,
}

/// A handle to a document in the documents store.
///
/// You can use it to perform a query.
#[php_class(name = "Xee\\DocumentHandle")]
pub struct DocumentHandle {
    handle: xee_xpath::DocumentHandle,
}

#[php_impl]
impl Documents {
    #[constructor]
    pub fn make_new() -> Documents {
        Documents {
            documents: xee_xpath::Documents::new(),
        }
    }

    /// Add a document to the Documents store from a string.
    ///
    /// The string must be well-formed XML.
    pub fn add_string(
        &mut self,
        uri: &str,
        content: &str,
    ) -> PhpResult<ZBox<ZendClassObject<DocumentHandle>>> {
        Ok(ZendClassObject::new(DocumentHandle {
            handle: self
                .documents
                .add_string(&xee_xpath::Uri::new(uri), content)
                .map_err(|e| e.to_string())?,
        }))
    }
}

/// A collection of XPath queries that can be executed against a document.
///
/// You can compile XPath expressions into queries using this store.
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

    /// A sequence query returns an XPath sequence.
    ///
    /// The query must be a valid XPath 3.1 expression.
    pub fn sequence(&mut self, query: &str) -> PhpResult<ZBox<ZendClassObject<SequenceQuery>>> {
        Ok(ZendClassObject::new(SequenceQuery {
            query: self.queries.sequence(query).map_err(|e| e.to_string())?,
        }))
    }
}

/// A compiled XPath query that returns a sequence.
#[php_class(name = "Xee\\SequenceQuery")]
pub struct SequenceQuery {
    query: xee_xpath::query::SequenceQuery,
}

#[php_impl]
impl SequenceQuery {
    /// Execute the query against a session and a document handle.
    pub fn execute(
        &self,
        documents: &mut ZendClassObject<Documents>,
        doc: &ZendClassObject<DocumentHandle>,
    ) -> PhpResult<ZBox<ZendClassObject<Sequence>>> {
        Ok(ZendClassObject::new(Sequence {
            sequence: self
                .query
                .execute(&mut documents.documents, doc.handle)
                .map_err(|e| e.to_string())?,
        }))
    }
}

/// A sequence of items returned by an XPath query.
///
/// This can be treated as an array and you can iterate over it.
#[php_class(name = "Xee\\Sequence")]
#[implements(ce::arrayaccess())]
#[implements(ce::countable())]
#[implements(ce::aggregate())]
pub struct Sequence {
    sequence: xee_xpath::Sequence,
}

/// An iterator over a sequence
#[php_class(name = "Xee\\SequenceIterator")]
#[implements(ce::iterator())]
pub struct SequenceIterator {
    // PHP interators unfortunately really drive you to implement them using
    // a position and explicit indexing.
    sequence: xee_xpath::Sequence,
    position: usize,
}

#[php_impl]
impl SequenceIterator {
    /// Rewind the iterator to the start.
    fn rewind(&mut self) {
        self.position = 0;
    }

    /// Get the current item in the sequence.
    fn current(&self) -> PhpResult<Zval> {
        sequence_offset_get(&self.sequence, self.position)
    }

    /// Get the key of the current item in the sequence.
    ///
    /// This is the position in the sequence.
    fn key(&self) -> PhpResult<Zval> {
        Ok(self.position.into_zval(false)?)
    }

    /// Move to the next item in the sequence.
    fn next(&mut self) {
        self.position += 1;
    }

    /// Check if the current position is valid.
    fn valid(&self) -> bool {
        self.position < self.sequence.len()
    }
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
            sequence_offset_get(&self.sequence, offset)
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

    pub fn get_iterator(&self) -> PhpResult<ZBox<ZendClassObject<SequenceIterator>>> {
        Ok(ZendClassObject::new(SequenceIterator {
            sequence: self.sequence.clone(),
            position: 0,
        }))
    }
}

fn sequence_offset_get(sequence: &xee_xpath::Sequence, offset: usize) -> PhpResult<Zval> {
    let item = sequence.get(offset).map_err(|e| e.to_string())?;
    match item {
        xee_xpath::Item::Atomic(atomic) => Ok(atomic_to_zval(&atomic, false)?),
        xee_xpath::Item::Node(_) => todo!(),
        xee_xpath::Item::Function(_) => todo!(),
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}

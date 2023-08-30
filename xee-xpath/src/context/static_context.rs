use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use icu::collator::Collator;
use icu_provider_blob::BlobDataProvider;
use xee_xpath_ast::ast;
use xee_xpath_ast::Namespaces;

use super::static_function::StaticFunctions;
use crate::string::provider;
use crate::string::CollatorQuery;
use crate::string::Collators;

#[derive(Debug)]
pub struct StaticContext<'a> {
    pub(crate) namespaces: &'a Namespaces<'a>,
    // XXX need to add in type later
    pub(crate) variables: Vec<ast::Name>,
    pub(crate) functions: StaticFunctions,
    provider: BlobDataProvider,
    pub(crate) collators: RefCell<Collators>,
}

impl<'a> StaticContext<'a> {
    pub fn new(namespaces: &'a Namespaces<'a>) -> Self {
        Self {
            namespaces,
            variables: Vec::new(),
            functions: StaticFunctions::new(),
            collators: RefCell::new(Collators::new()),
            provider: provider(),
        }
    }

    pub fn with_variable_names(namespaces: &'a Namespaces<'a>, variables: &[ast::Name]) -> Self {
        Self {
            namespaces,
            variables: variables.to_vec(),
            functions: StaticFunctions::new(),
            collators: RefCell::new(Collators::new()),
            provider: provider(),
        }
    }

    pub fn default_collator(&self) -> Rc<Collator> {
        let collator_query = CollatorQuery {
            lang: None, // the implies the default, undefined collator
            ..Default::default()
        };
        let mut collators = self.collators.borrow_mut();
        collators
            .load(self.provider.clone(), &collator_query)
            .unwrap()
    }
}

use std::path::PathBuf;

use xee_xpath::{
    context::{DynamicContext, StaticContext, Variables},
    parse, sequence,
    xml::Documents,
    Name,
};
use xot::Xot;

use crate::error::Result;

use super::{
    collation::Collation,
    collection::Collection,
    resource::Resource,
    source::{Source, SourceRole},
};

// environment information shared by XPath and XSLT
#[derive(Debug, Default, Clone)]
pub(crate) struct EnvironmentSpec {
    pub(crate) base_dir: PathBuf,

    pub(crate) sources: Vec<Source>,
    pub(crate) params: Vec<Param>,
    // TODO
    pub(crate) collations: Vec<Collation>,
    // TODO: needs to wait until the interpreter has a resource abstraction
    pub(crate) resources: Vec<Resource>,
    // TODO: needs to wait until the interpreter has a collection abstraction
    pub(crate) collections: Vec<Collection>,
    // not supported as Xee doesn't support XML schema
    pub(crate) schemas: Vec<Schema>,
    // Not in use at all?
    // pub(crate) function_libraries: Vec<FunctionLibrary>,
}

// Not supported yet: schema support not implemented in Xee
#[derive(Debug, Clone)]
pub(crate) struct Schema {}

#[derive(Debug, Clone)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) select: Option<String>,
    // TODO: not supported yet
    pub(crate) as_: Option<String>,
    // TODO: not supported yet
    pub(crate) static_: bool,
    // XQuery related, not supported
    pub(crate) declared: bool,
    // Doesn't appear to be in use, even though it's in the schema
    pub(crate) source: Option<String>,
}

impl EnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn context_item(
        &self,
        xot: &mut Xot,
        documents: &mut Documents,
    ) -> Result<Option<sequence::Item>> {
        for source in &self.sources {
            if let SourceRole::Context = source.role {
                let node = source.node(xot, &self.base_dir, documents)?;
                return Ok(Some(sequence::Item::from(node)));
            }
        }
        Ok(None)
    }

    pub(crate) fn variables(&self, xot: &mut Xot, documents: &mut Documents) -> Result<Variables> {
        let mut variables = Variables::new();
        for source in &self.sources {
            if let SourceRole::Var(name) = &source.role {
                let name = &name[1..]; // without $
                let node = source.node(xot, &self.base_dir, documents)?;
                variables.insert(Name::name(name), sequence::Item::from(node).into());
            }
        }
        for param in &self.params {
            let static_context = StaticContext::default();
            let select = (param.select.as_ref()).expect("param: missing select not supported");
            let program = parse(&static_context, select);
            if program.is_err() {
                println!("param: select xpath parse failed: {}", select);
                continue;
            }
            let program = program.unwrap();
            let dynamic_context = DynamicContext::empty(&static_context);
            let runnable = program.runnable(&dynamic_context);
            let result = runnable.many(None, xot).map_err(|e| e.error)?;
            variables.insert(param.name.clone(), result);
        }
        Ok(variables)
    }
}

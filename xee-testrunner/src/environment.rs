use std::path::PathBuf;

use crate::{
    collation::Collation, collection::Collection, decimal_format::DecimalFormat,
    resource::Resource, source::Source,
};

type Name = xot::xmlname::OwnedName;

// environment information shared by XPath and XSLT
#[derive(Debug, Default, Clone)]
pub(crate) struct EnvironmentSpec {
    pub(crate) schemas: Vec<Schema>,
    pub(crate) sources: Vec<Source>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) params: Vec<Param>,
    pub(crate) collections: Vec<Collection>,
    pub(crate) collations: Vec<Collation>,
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
    pub(crate) as_: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) declared: bool,
}

impl EnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }
}

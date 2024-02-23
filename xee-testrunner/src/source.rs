use std::path::PathBuf;

use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    pub(crate) role: Option<SourceRole>,
    pub(crate) file: Option<PathBuf>,
    pub(crate) uri: Option<String>,
    pub(crate) validation: Option<Validation>,
}

#[derive(Debug, Clone)]
pub(crate) enum Validation {
    Strict,
    Lax,
    Skip,
}

#[derive(Debug, Clone)]
pub(crate) enum SourceRole {
    Context,
    Var(String),
    Doc(String), // URI
}

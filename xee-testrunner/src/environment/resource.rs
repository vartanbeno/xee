use std::path::PathBuf;

use super::metadata::Metadata;

#[derive(Debug, Clone)]
pub(crate) struct Resource {
    metadata: Metadata,
    file: Option<PathBuf>,
    uri: String,
    media_type: Option<String>,
    encoding: Option<String>,
}

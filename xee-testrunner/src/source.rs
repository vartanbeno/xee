use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use xee_xpath::xml::{Documents, Uri};
use xot::Xot;

use crate::error::Result;
use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    // note that in a collection source the role can be ommitted, so
    // we may need to define this differently
    pub(crate) role: SourceRole,
    pub(crate) file: PathBuf,
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

impl Source {
    pub(crate) fn node(
        &self,
        xot: &mut Xot,
        base_dir: &Path,
        documents: &mut Documents,
    ) -> Result<xot::Node> {
        let full_path = base_dir.join(&self.file);
        // construct a Uri
        // TODO: this is not really a proper URI but
        // what matters is that it's unique here
        let uri = Uri::new(&full_path.to_string_lossy());

        // try to get the cached version of the document
        let document = documents.get(&uri);
        if let Some(document) = document {
            let root = document.root();
            return Ok(root);
        }

        // could not get cached version, so load up document
        let xml_file = File::open(&full_path)?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader.read_to_string(&mut xml)?;

        documents.add(xot, &uri, &xml)?;
        // now obtain what we just added
        Ok(documents.get(&uri).unwrap().root())
    }
}

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use xee_xpath::xml::{Documents, Uri};
use xee_xpath::{Queries, Query};
use xot::Xot;

use crate::error::Result;

use crate::load::{convert_string, Loadable};
use crate::metadata::Metadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    // note that in a collection source the role can be ommitted, so
    // we may need to define this differently
    pub(crate) role: SourceRole,
    pub(crate) file: PathBuf,
    pub(crate) uri: Option<String>,
    pub(crate) validation: Option<Validation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Validation {
    Strict,
    Lax,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub(crate) fn query(
        mut queries: Queries,
    ) -> Result<(Queries, impl Query<Vec<Vec<Self>>> + '_)> {
        let file_query = queries.one("@file/string()", convert_string)?;
        let role_query = queries.option("@role/string()", convert_string)?;
        let uri_query = queries.option("@uri/string()", convert_string)?;
        let (mut queries, metadata_query) = Metadata::query(queries)?;

        let sources_query = queries.many("source", move |session, item| {
            let file = PathBuf::from(file_query.execute(session, item)?);
            let role = role_query.execute(session, item)?;
            let uri = uri_query.execute(session, item)?;
            let metadata = metadata_query.execute(session, item)?;
            // we can return multiple sources if both role and uri are set
            // we flatten it later
            let mut sources = Vec::new();
            if let Some(role) = role {
                if role == "." {
                    sources.push(Source {
                        metadata: metadata.clone(),
                        role: SourceRole::Context,
                        file: file.clone(),
                        // TODO
                        uri: None,
                        validation: None,
                    })
                } else {
                    sources.push(Source {
                        metadata: metadata.clone(),
                        role: SourceRole::Var(role),
                        file: file.clone(),
                        // TODO
                        uri: None,
                        validation: None,
                    });
                }
            };

            if let Some(uri) = uri {
                sources.push(Source {
                    metadata,
                    role: SourceRole::Doc(uri),
                    file,
                    // TODO
                    uri: None,
                    validation: None,
                });
            }

            Ok(sources)
        })?;
        Ok((queries, sources_query))
    }
}

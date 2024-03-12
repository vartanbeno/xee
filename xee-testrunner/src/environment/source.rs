use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use xee_xpath::xml::{Documents, Uri};
use xee_xpath_load::{convert_string, Loadable, Queries, Query};
use xot::Xot;

use crate::metadata::Metadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Source {
    pub(crate) metadata: Metadata,
    // note that in a collection source the role can be ommitted, so
    // we may need to define this differently
    pub(crate) role: SourceRole,
    pub(crate) content: SourceContent,
    pub(crate) uri: Option<String>,
    pub(crate) validation: Option<Validation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceContent {
    Path(PathBuf),
    String(String),
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
        match &self.content {
            SourceContent::Path(path) => {
                let full_path = base_dir.join(path);
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
            SourceContent::String(value) => {
                // create a new unique uri
                let uri = Uri::new(&format!("string-source-{}", documents.len()));
                // we don't try to get a cached version of the document, as
                // that would be different each time. we just add it to documents
                // and return it
                documents.add(xot, &uri, value)?;
                Ok(documents.get(&uri).unwrap().root())
            }
        }
    }

    pub(crate) fn load(mut queries: Queries) -> Result<(Queries, impl Query<Vec<Vec<Self>>> + '_)> {
        let file_query = queries.option("@file/string()", convert_string)?;
        let content_query = queries.one("content/string()", convert_string)?;
        let role_query = queries.option("@role/string()", convert_string)?;
        let uri_query = queries.option("@uri/string()", convert_string)?;
        let (mut queries, metadata_query) = Metadata::load(queries)?;

        let sources_query = queries.many("source", move |session, item| {
            let content = if let Some(file) = file_query.execute(session, item)? {
                SourceContent::Path(PathBuf::from(file))
            } else {
                // look for content inside
                let s = content_query.execute(session, item)?;
                SourceContent::String(s)
            };
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
                        content: content.clone(),
                        // TODO
                        uri: None,
                        validation: None,
                    })
                } else {
                    sources.push(Source {
                        metadata: metadata.clone(),
                        role: SourceRole::Var(role),
                        content: content.clone(),
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
                    content,
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

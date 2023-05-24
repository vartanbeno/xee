use fxhash::FxHashMap;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use xee_xpath::{Item, Node};
use xot::Xot;

use crate::qt;
use crate::qt::EnvironmentSpec;
use crate::qt::Source;

impl EnvironmentSpec {
    pub(crate) fn context_item(
        &self,
        xot: &mut Xot,
        base_dir: &Path,
        source_cache: &mut SourceCache,
    ) -> Result<Option<Item>> {
        for source in &self.sources {
            if let qt::SourceRole::Context = source.role {
                let node = source.node(xot, base_dir, source_cache)?;
                return Ok(Some(Item::Node(node)));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourceCache {
    nodes: FxHashMap<PathBuf, Node>,
}

impl SourceCache {
    pub(crate) fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
        }
    }

    pub(crate) fn cleanup(&self, xot: &mut Xot) {
        for node in self.nodes.values() {
            if let Node::Xot(root) = node {
                xot.remove(*root).unwrap();
            }
        }
    }
}

impl Source {
    pub(crate) fn node(
        &self,
        xot: &mut Xot,
        base_dir: &Path,
        source_cache: &mut SourceCache,
    ) -> Result<Node> {
        let full_path = base_dir.join(&self.file);
        let node = source_cache.nodes.get(&full_path);
        if let Some(node) = node {
            return Ok(*node);
        }

        let xml_file = File::open(&full_path).into_diagnostic().wrap_err_with(|| {
            format!("Cannot open XML file for source: {}", full_path.display())
        })?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader
            .read_to_string(&mut xml)
            .into_diagnostic()
            .wrap_err("Cannot read XML file for source")?;
        let root = xot
            .parse(&xml)
            .into_diagnostic()
            .wrap_err("Cannot parse XML file for source")?;
        let node = Node::Xot(root);

        source_cache.nodes.insert(full_path, node);
        Ok(node)
    }
}

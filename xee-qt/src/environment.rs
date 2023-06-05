use fxhash::FxHashMap;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use xee_xpath::Name;
use xee_xpath::Value;

use xee_xpath::{Item, Node};
use xot::Xot;

use crate::collection::FxIndexMap;
use crate::qt;
use crate::qt::EnvironmentSpec;
use crate::qt::Source;

impl EnvironmentSpec {
    pub(crate) fn context_item(
        &self,
        xot: &mut Xot,
        source_cache: &mut SourceCache,
    ) -> Result<Option<Item>> {
        for source in &self.sources {
            if let qt::SourceRole::Context = source.role {
                let node = source.node(xot, &self.base_dir, source_cache)?;
                return Ok(Some(Item::Node(node)));
            }
        }
        Ok(None)
    }

    pub(crate) fn variables(
        &self,
        xot: &mut Xot,
        source_cache: &mut SourceCache,
    ) -> Result<Vec<(Name, Value)>> {
        let mut variables = Vec::new();
        for source in &self.sources {
            if let qt::SourceRole::Var(name) = &source.role {
                let name = &name[1..]; // without $
                let node = source.node(xot, &self.base_dir, source_cache)?;
                variables.push((Name::without_ns(name), Value::Node(node)));
            }
        }
        Ok(variables)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SharedEnvironments {
    environments: FxIndexMap<String, EnvironmentSpec>,
}

impl SharedEnvironments {
    pub(crate) fn new(mut environments: FxIndexMap<String, EnvironmentSpec>) -> Self {
        // there is always an empty environment
        if !environments.contains_key("empty") {
            let empty = EnvironmentSpec::empty();
            environments.insert("empty".to_string(), empty);
        }
        Self { environments }
    }

    pub(crate) fn get(&self, environment_ref: &qt::EnvironmentRef) -> Option<&EnvironmentSpec> {
        self.environments.get(&environment_ref.ref_)
    }
}

pub(crate) struct EnvironmentSpecIterator<'a> {
    pub(crate) catalog_shared_environments: &'a SharedEnvironments,
    pub(crate) test_set_shared_environments: &'a SharedEnvironments,
    pub(crate) environments: &'a [qt::TestCaseEnvironment],
    pub(crate) index: usize,
}

impl<'a> Iterator for EnvironmentSpecIterator<'a> {
    type Item = Result<&'a qt::EnvironmentSpec>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.environments.len() {
            return None;
        }
        let environment = &self.environments[self.index];
        self.index += 1;
        match environment {
            qt::TestCaseEnvironment::Local(local_environment_spec) => {
                Some(Ok(local_environment_spec))
            }
            qt::TestCaseEnvironment::Ref(environment_ref) => {
                for shared_environments in [
                    self.test_set_shared_environments,
                    self.catalog_shared_environments,
                ] {
                    let environment_spec = shared_environments.get(environment_ref);
                    if let Some(environment_spec) = environment_spec {
                        return Some(Ok(environment_spec));
                    }
                }
                Some(Err(miette!(
                    "Unknown environment reference: {}",
                    environment_ref
                )))
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
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

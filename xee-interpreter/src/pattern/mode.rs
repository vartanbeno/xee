use ahash::{HashMap, HashMapExt};

use xee_xpath_ast::Pattern;
use xot::xmlname;

use crate::function;

use super::pattern_lookup::PatternLookup;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModeValue {
    Named(xmlname::OwnedName),
    Default,
    Unnamed,
    All,
}

#[derive(Debug, Default)]
pub struct ModeLookup<V: Clone> {
    pub(crate) modes: HashMap<Option<xmlname::OwnedName>, PatternLookup<V>>,
}

impl<V: Clone> ModeLookup<V> {
    pub(crate) fn new() -> Self {
        Self {
            modes: HashMap::new(),
        }
    }

    pub(crate) fn lookup(
        &self,
        mode: &Option<xmlname::OwnedName>,
        mut matches: impl FnMut(&Pattern<function::InlineFunctionId>) -> bool,
    ) -> Option<&V> {
        self.modes
            .get(mode)
            .and_then(|lookup| lookup.lookup(&mut matches))
    }

    pub fn add_rules(
        &mut self,
        mode: Option<xmlname::OwnedName>,
        rules: Vec<(Pattern<function::InlineFunctionId>, V)>,
    ) {
        self.modes
            .entry(mode)
            .or_insert_with(PatternLookup::new)
            .add_rules(rules);
    }
}

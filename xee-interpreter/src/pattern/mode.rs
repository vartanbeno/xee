use ahash::{HashMap, HashMapExt};

use xee_xpath_ast::Pattern;

use crate::function;

use super::pattern_lookup::PatternLookup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModeId(usize);

impl ModeId {
    pub fn new(id: usize) -> Self {
        ModeId(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct ModeLookup<V: Clone> {
    pub(crate) unnamed_mode: Option<PatternLookup<V>>,
    pub(crate) modes: HashMap<ModeId, PatternLookup<V>>,
}

impl<V: Clone> ModeLookup<V> {
    pub(crate) fn new() -> Self {
        Self {
            unnamed_mode: None,
            modes: HashMap::new(),
        }
    }

    pub(crate) fn lookup(
        &self,
        mode: Option<ModeId>,
        mut matches: impl FnMut(&Pattern<function::InlineFunctionId>) -> bool,
    ) -> Option<&V> {
        let pattern_lookup = if let Some(mode) = mode {
            self.modes.get(&mode)
        } else {
            self.unnamed_mode.as_ref()
        }?;
        pattern_lookup.lookup(&mut matches)
    }

    pub fn add_rules(
        &mut self,
        mode: Option<ModeId>,
        rules: Vec<(Pattern<function::InlineFunctionId>, V)>,
    ) {
        let pattern_lookup = if let Some(mode) = mode {
            self.modes.entry(mode).or_insert_with(PatternLookup::new)
        } else {
            self.unnamed_mode.get_or_insert_with(PatternLookup::new)
        };
        pattern_lookup.add_rules(rules);
    }
}

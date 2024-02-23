use crate::hashmap::FxIndexMap;

use super::core::{Environment, EnvironmentRef};

#[derive(Debug, Default, Clone)]
pub(crate) struct SharedEnvironments<E: Environment> {
    environments: FxIndexMap<String, E>,
}

impl<E: Environment> SharedEnvironments<E> {
    pub(crate) fn new(mut environments: FxIndexMap<String, E>) -> Self {
        // there is always an empty environment
        if !environments.contains_key("empty") {
            let empty = E::empty();
            environments.insert("empty".to_string(), empty);
        }
        Self { environments }
    }

    pub(crate) fn get(&self, environment_ref: &EnvironmentRef) -> Option<&E> {
        self.environments.get(&environment_ref.ref_)
    }
}

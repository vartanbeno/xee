use std::path::Path;

use xee_xpath::{Queries, Query};
use xot::Xot;

use crate::hashmap::FxIndexMap;
use crate::{error::Result, load::convert_string};

use super::core::{Environment, EnvironmentRef};
use super::EnvironmentSpec;

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

    // pub(crate) fn shared_environments_query<'a>(
    //     xot: &Xot,
    //     path: &'a Path,
    //     mut queries: Queries<'a>,
    // ) -> Result<(Queries<'a>, impl Query<Self> + 'a)> {
    //     let name_query = queries.one("@name/string()", convert_string)?;
    //     let (mut queries, environment_spec_query) =
    //         EnvironmentSpec::environment_spec_query(xot, path, queries)?;
    //     let environments_query = queries.many("environment", move |session, item| {
    //         let name = name_query.execute(session, item)?;
    //         let environment_spec = environment_spec_query.execute(session, item)?;
    //         Ok((name, environment_spec))
    //     })?;
    //     let shared_environments_query = queries.one(".", move |session, item| {
    //         let environments = environments_query.execute(session, item)?;
    //         Ok(SharedEnvironments::new(environments.into_iter().collect()))
    //     })?;
    //     Ok((queries, shared_environments_query))
    // }
}

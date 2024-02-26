use xee_xpath::xml::Documents;
use xot::Xot;

use crate::{catalog::Catalog, dependency::KnownDependencies, environment::Environment};

pub(crate) struct RunContext {
    pub(crate) xot: Xot,
    pub(crate) documents: Documents,
    pub(crate) known_dependencies: KnownDependencies,
    // pub(crate) verbose: bool,
}

impl RunContext {
    pub(crate) fn new(
        xot: Xot,
        documents: Documents,
        known_dependencies: KnownDependencies,
    ) -> Self {
        Self {
            xot,
            documents,
            known_dependencies,
        }
    }
}

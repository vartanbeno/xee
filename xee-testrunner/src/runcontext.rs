use xee_xpath::xml::Documents;
use xot::Xot;

use crate::{catalog::Catalog, dependency::KnownDependencies, environment::Environment};

pub(crate) struct RunContext<E: Environment> {
    pub(crate) xot: Xot,
    pub(crate) catalog: Catalog<E>,
    pub(crate) documents: Documents,
    pub(crate) known_dependencies: KnownDependencies,
    // pub(crate) verbose: bool,
}

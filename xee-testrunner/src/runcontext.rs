use xee_xpath::xml::Documents;
use xot::Xot;

use crate::dependency::KnownDependencies;

pub(crate) struct RunContext {
    pub(crate) xot: Xot,
    pub(crate) documents: Documents,
    pub(crate) known_dependencies: KnownDependencies,
    pub(crate) ns: String,
    pub(crate) verbose: bool,
}

impl RunContext {
    pub(crate) fn new(
        xot: Xot,
        documents: Documents,
        known_dependencies: KnownDependencies,
        ns: String,
        verbose: bool,
    ) -> Self {
        Self {
            xot,
            documents,
            known_dependencies,
            ns,
            verbose,
        }
    }
}

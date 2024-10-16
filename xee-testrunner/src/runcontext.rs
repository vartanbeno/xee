use xee_xpath::Documents;

use crate::{
    dependency::KnownDependencies,
    environment::Environment,
    renderer::{CharacterRenderer, Renderer, VerboseRenderer},
    testcase::Runnable,
};

pub(crate) struct RunContext<'a> {
    pub(crate) documents: &'a mut Documents,
    pub(crate) known_dependencies: KnownDependencies,
    pub(crate) verbose: bool,
}

impl<'a> RunContext<'a> {
    pub(crate) fn new(
        documents: &'a mut Documents,
        known_dependencies: KnownDependencies,
        verbose: bool,
    ) -> Self {
        Self {
            documents,
            known_dependencies,
            verbose,
        }
    }

    pub(crate) fn renderer<E: Environment, R: Runnable<E>>(&self) -> Box<dyn Renderer<E, R>> {
        if self.verbose {
            Box::new(VerboseRenderer::new())
        } else {
            Box::new(CharacterRenderer::new())
        }
    }
}

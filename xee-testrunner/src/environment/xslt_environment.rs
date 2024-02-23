use super::core::{Environment, EnvironmentSpec};

#[derive(Debug, Clone)]
pub(crate) struct Package {
    // TODO
}

#[derive(Debug, Clone)]
pub(crate) struct Stylesheet {
    // TODO
}

#[derive(Debug, Clone)]
pub(crate) struct Output {
    // TODO
}

#[derive(Debug, Clone)]
pub(crate) struct XsltEnvironmentSpec {
    pub(crate) environment_spec: EnvironmentSpec,

    pub(crate) packages: Vec<Package>,
    pub(crate) stylesheets: Vec<Stylesheet>,
    pub(crate) outputs: Vec<Output>,
}

impl XsltEnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            environment_spec: EnvironmentSpec::empty(),
            packages: vec![],
            stylesheets: vec![],
            outputs: vec![],
        }
    }
}

impl Environment for XsltEnvironmentSpec {
    fn empty() -> Self {
        Self::empty()
    }

    fn environment_spec(&self) -> &EnvironmentSpec {
        &self.environment_spec
    }
}

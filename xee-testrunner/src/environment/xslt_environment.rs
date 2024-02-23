use super::core::EnvironmentSpec;

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

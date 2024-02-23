#[derive(Debug, Clone)]
pub(crate) struct Collation {
    pub(crate) uri: Option<String>,
    pub(crate) default: bool,
}

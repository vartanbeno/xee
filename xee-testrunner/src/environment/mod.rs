mod collation;
mod collection;
mod core;
mod decimal_format;
mod resource;
mod shared;
mod source;
mod xpath_environment;
mod xslt_environment;

pub(crate) use core::Environment;
pub(crate) use shared::{EnvironmentRef, SharedEnvironments};

/// XML integration.
mod annotation;
mod base;
mod document;
mod kind_test;
mod step;

pub use annotation::Annotations;
pub(crate) use base::BaseUriResolver;
pub use document::{Document, DocumentHandle, Documents, DocumentsError};
pub(crate) use kind_test::kind_test;
pub(crate) use step::resolve_step;
pub use step::Step;

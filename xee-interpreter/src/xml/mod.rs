/// XML integration.
mod annotation;
mod document2;
mod kind_test;
mod step;

pub use annotation::Annotations;
// pub use document::Document;
pub use document2::{Document, DocumentHandle, Documents, DocumentsError, Uri};
// pub use document::{Documents, Uri};
pub(crate) use kind_test::kind_test;
pub(crate) use step::resolve_step;
pub use step::Step;

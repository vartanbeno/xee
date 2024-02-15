/// XML integration.
mod annotation;
mod document;
mod kind_test;
mod step;

pub use annotation::Annotations;
pub use document::Document;
pub use document::{Documents, Uri};
pub(crate) use kind_test::kind_test;
pub(crate) use step::resolve_step;
pub use step::Step;

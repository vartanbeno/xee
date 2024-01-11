mod name;
mod namespaces;
mod variable_names;

pub use name::Name;
pub use namespaces::{NamespaceLookup, Namespaces, FN_NAMESPACE, XS_NAMESPACE};
pub use variable_names::VariableNames;

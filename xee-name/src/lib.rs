mod name;
mod namespaces;
mod variable_names;

// pub use name::{Error};
pub use namespaces::{NamespaceLookup, Namespaces, FN_NAMESPACE, XS_NAMESPACE};
pub use variable_names::VariableNames;
pub use xot::xmlname::OwnedName as Name;

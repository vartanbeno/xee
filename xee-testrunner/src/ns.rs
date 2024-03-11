use xee_name::Namespaces;

pub(crate) const XPATH_NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

pub(crate) fn namespaces(ns: &str) -> Namespaces {
    Namespaces::new(
        Namespaces::default_namespaces(),
        ns,
        Namespaces::FN_NAMESPACE,
    )
}

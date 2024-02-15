use xot::Xot;

use crate::namespaces::NamespaceLookup;

// TODO: use Cow<str> for the various fields

#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name {
    name: String,
    prefix: Option<String>,
    namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MissingPrefix,
}

// a custom hasher that ignores the prefix
impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.namespace.hash(state);
    }
}

// and partial eq that ignores the prefix
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.namespace == other.namespace
    }
}

impl Name {
    pub fn new(name: String, namespace: Option<String>, prefix: Option<String>) -> Self {
        Name {
            name,
            namespace,
            prefix,
        }
    }

    pub fn from_xot(name: xot::NameId, xot: &Xot) -> Self {
        let (name, namespace) = xot.name_ns_str(name);
        let namespace = if !namespace.is_empty() {
            Some(namespace.to_string())
        } else {
            None
        };
        Name {
            name: name.to_string(),
            namespace,
            prefix: None,
        }
    }

    /// Get the full name from xot including any prefix information.
    pub fn from_xot_with_prefix(
        name: xot::NameId,
        context: xot::Node,
        xot: &Xot,
    ) -> Result<Self, Error> {
        let namespace = xot.namespace_for_name(name);
        let local_name = xot.local_name_str(name);
        if namespace == xot.no_namespace() {
            return Ok(Name {
                name: local_name.to_string(),
                namespace: None,
                prefix: None,
            });
        }
        let prefix = xot.prefix_for_namespace(context, namespace);
        if let Some(prefix) = prefix {
            let namespace = xot.namespace_str(namespace);

            let prefix = xot.prefix_str(prefix);
            Ok(Name {
                name: local_name.to_string(),
                namespace: Some(namespace.to_string()),
                prefix: Some(prefix.to_string()),
            })
        } else {
            Err(Error::MissingPrefix)
        }
    }

    /// Take the node name of the xot node. If it exists, pull out full
    /// information, including prefix.
    ///
    /// If the node has no name, return None.
    pub fn from_xot_node(node: xot::Node, xot: &Xot) -> Result<Option<Self>, Error> {
        let name = xot.node_name(node);
        if let Some(name) = name {
            Ok(Some(Name::from_xot_with_prefix(name, node, xot)?))
        } else {
            Ok(None)
        }
    }

    pub fn prefixed(prefix: &str, name: &str, namespaces: impl NamespaceLookup) -> Option<Self> {
        let namespace = namespaces.by_prefix(prefix)?;
        Some(Name {
            name: name.to_string(),
            namespace: Some(namespace.to_string()),
            prefix: Some(prefix.to_string()),
        })
    }

    pub fn unprefixed(name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: None,
            prefix: None,
        }
    }

    pub fn uri_qualified(uri: &str, name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: Some(uri.to_string()),
            prefix: None,
        }
    }

    pub fn with_default_namespace(self, uri: Option<&str>) -> Self {
        if let Some(uri) = uri {
            if self.namespace.is_none() {
                return Name {
                    name: self.name,
                    namespace: Some(uri.to_string()),
                    prefix: None,
                };
            }
        }
        self
    }

    pub fn has_namespace_without_prefix(&self) -> bool {
        self.namespace.is_some() && self.prefix.is_none()
    }

    #[inline]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    #[inline]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    #[inline]
    pub fn local_name(&self) -> &str {
        &self.name
    }

    pub fn to_full_name(&self) -> String {
        if let Some(prefix) = &self.prefix {
            if !prefix.is_empty() {
                format!("{}:{}", prefix, self.name)
            } else {
                self.name.clone()
            }
        } else {
            self.name.clone()
        }
    }

    pub fn add_name_id(&self, xot: &mut Xot) -> xot::NameId {
        if let Some(namespace) = &self.namespace {
            let ns = xot.add_namespace(namespace);
            xot.add_name_ns(&self.name, ns)
        } else {
            xot.add_name(&self.name)
        }
    }

    pub fn to_name_id(&self, xot: &Xot) -> Option<xot::NameId> {
        if let Some(namespace) = &self.namespace {
            let namespace_id = xot.namespace(namespace);
            if let Some(namespace_id) = namespace_id {
                xot.name_ns(&self.name, namespace_id)
            } else {
                None
            }
        } else {
            xot.name(&self.name)
        }
    }

    pub fn with_suffix(&self) -> Name {
        let mut name = self.name.clone();
        name.push('*');
        Name {
            name,
            namespace: self.namespace.clone(),
            prefix: self.prefix.clone(),
        }
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self {
        Name {
            name: name.to_string(),
            namespace: None,
            prefix: None,
        }
    }
}

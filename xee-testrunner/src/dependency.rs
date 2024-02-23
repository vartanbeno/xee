use crate::hashmap::FxIndexSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DependencySpec {
    pub(crate) type_: String,
    pub(crate) value: String,
}

#[derive(Debug)]
pub(crate) struct Dependency {
    pub(crate) spec: DependencySpec,
    pub(crate) satisfied: bool,
}

#[derive(Debug)]
pub(crate) struct Dependencies {
    pub(crate) dependencies: Vec<Dependency>,
}

#[derive(Debug)]
pub(crate) struct KnownDependencies {
    specs: FxIndexSet<DependencySpec>,
}

fn xpath_known_dependencies() -> KnownDependencies {
    let specs = vec![
        DependencySpec {
            type_: "spec".to_string(),
            value: "XP20+".to_string(),
        },
        DependencySpec {
            type_: "spec".to_string(),
            value: "XP30+".to_string(),
        },
        DependencySpec {
            type_: "spec".to_string(),
            value: "XP31+".to_string(),
        },
        DependencySpec {
            type_: "feature".to_string(),
            value: "higherOrderFunctions".to_string(),
        },
        DependencySpec {
            type_: "xml-version".to_string(),
            value: "1.0".to_string(),
        },
    ];
    KnownDependencies::new(&specs)
}

impl KnownDependencies {
    fn new(specs: &[DependencySpec]) -> Self {
        let specs = specs.iter().cloned().collect();
        Self { specs }
    }

    fn is_supported(&self, dependency: &Dependency) -> bool {
        let contains = self.specs.contains(&dependency.spec);
        if dependency.satisfied {
            contains
        } else {
            !contains
        }
    }
}

impl Dependencies {
    // the spec is supported if any of the spec dependencies is supported
    pub(crate) fn is_spec_supported(&self, known_dependencies: &KnownDependencies) -> bool {
        let mut spec_dependency_seen: bool = false;
        for dependency in &self.dependencies {
            if dependency.spec.type_ == "spec" {
                spec_dependency_seen = true;
                if known_dependencies.is_supported(dependency) {
                    return true;
                }
            }
        }
        // if we haven't seen any spec dependencies, then we're supported
        // otherwise, we aren't
        !spec_dependency_seen
    }

    pub(crate) fn is_feature_supported(&self, known_dependencies: &KnownDependencies) -> bool {
        for dependency in &self.dependencies {
            // if a listed feature dependency is not supported, we don't support this
            if dependency.spec.type_ == "feature" && !known_dependencies.is_supported(dependency) {
                return false;
            }
        }
        true
    }

    // the XML version is supported if the the xml-version is the same
    pub(crate) fn is_xml_version_supported(&self, known_dependencies: &KnownDependencies) -> bool {
        for dependency in &self.dependencies {
            if dependency.spec.type_ == "xml-version"
                && !known_dependencies.is_supported(dependency)
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_supported(&self, known_dependencies: &KnownDependencies) -> bool {
        // if we have no dependencies, we're always supported
        if self.dependencies.is_empty() {
            return true;
        }
        // if we don't support the spec, we don't support it
        if !self.is_spec_supported(known_dependencies) {
            return false;
        }
        if !self.is_xml_version_supported(known_dependencies) {
            return false;
        }
        self.is_feature_supported(known_dependencies)
    }
}

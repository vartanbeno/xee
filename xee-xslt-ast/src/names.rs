use std::collections::BTreeMap;
use std::str::FromStr;

use strum::VariantNames;
use xot::{NameId, NamespaceId, Xot};

use crate::ast_core::{self as ast, DeclarationName, SequenceConstructorName};
use crate::combinator::ElementError;
use crate::element::Element;
use crate::instruction::{DeclarationParser, SequenceConstructorParser};

impl SequenceConstructorName {
    pub(crate) fn parse(
        &self,
        element: &Element,
    ) -> Result<ast::SequenceConstructorItem, ElementError> {
        match self {
            SequenceConstructorName::ApplyImports => {
                ast::ApplyImports::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::AnalyzeString => {
                ast::AnalyzeString::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::ApplyTemplates => {
                ast::ApplyTemplates::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::Assert => {
                ast::Assert::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::Attribute => {
                ast::Attribute::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::Copy => ast::Copy::parse_sequence_constructor_item(element),
            SequenceConstructorName::CopyOf => {
                ast::CopyOf::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::If => ast::If::parse_sequence_constructor_item(element),
            SequenceConstructorName::Variable => {
                ast::Variable::parse_sequence_constructor_item(element)
            }
            SequenceConstructorName::Fallback => {
                ast::Fallback::parse_sequence_constructor_item(element)
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn names(xot: &mut Xot, xsl_ns: xot::NamespaceId) -> BTreeMap<NameId, SequenceConstructorName> {
        let mut sequence_constructor_names = BTreeMap::new();
        for variant_name in Self::VARIANTS {
            let name = xot.add_name_ns(variant_name, xsl_ns);
            let constructor = SequenceConstructorName::from_str(variant_name).unwrap();
            sequence_constructor_names.insert(name, constructor);
        }
        sequence_constructor_names
    }
}

impl DeclarationName {
    pub(crate) fn parse(&self, element: &Element) -> Result<ast::Declaration, ElementError> {
        match self {
            DeclarationName::Accumulator => ast::Accumulator::parse_declaration(element),
            _ => {
                unimplemented!()
            }
        }
    }

    fn names(xot: &mut Xot, xsl_ns: xot::NamespaceId) -> BTreeMap<NameId, DeclarationName> {
        let mut declaration_names = BTreeMap::new();
        for variant_name in Self::VARIANTS {
            let name = xot.add_name_ns(variant_name, xsl_ns);
            let constructor = DeclarationName::from_str(variant_name).unwrap();
            declaration_names.insert(name, constructor);
        }
        declaration_names
    }
}

pub(crate) struct Names {
    pub(crate) xsl_ns: NamespaceId,

    pub(crate) sequence_constructor_names: BTreeMap<NameId, SequenceConstructorName>,
    pub(crate) declaration_names: BTreeMap<NameId, DeclarationName>,

    // XSL elements
    pub(crate) xsl_accumulator_rule: xot::NameId,
    pub(crate) xsl_attribute: xot::NameId,
    pub(crate) xsl_fallback: xot::NameId,
    pub(crate) xsl_matching_substring: xot::NameId,
    pub(crate) xsl_non_matching_substring: xot::NameId,
    pub(crate) xsl_sort: xot::NameId,
    pub(crate) xsl_transform: xot::NameId,
    pub(crate) xsl_with_param: xot::NameId,

    // attributes on XSLT elements
    pub(crate) as_: xot::NameId,
    pub(crate) case_order: xot::NameId,
    pub(crate) component: xot::NameId,
    pub(crate) collation: xot::NameId,
    pub(crate) copy_accumulators: xot::NameId,
    pub(crate) copy_namespaces: xot::NameId,
    pub(crate) data_type: xot::NameId,
    pub(crate) error_code: xot::NameId,
    pub(crate) extension_element_prefixes: xot::NameId,
    pub(crate) flags: xot::NameId,
    pub(crate) id: xot::NameId,
    pub(crate) inherit_namespaces: xot::NameId,
    pub(crate) initial_value: xot::NameId,
    pub(crate) input_type_annotations: xot::NameId,
    pub(crate) lang: xot::NameId,
    pub(crate) match_: xot::NameId,
    pub(crate) mode: xot::NameId,
    pub(crate) name: xot::NameId,
    pub(crate) names: xot::NameId,
    pub(crate) namespace: xot::NameId,
    pub(crate) order: xot::NameId,
    pub(crate) phase: xot::NameId,
    pub(crate) regex: xot::NameId,
    pub(crate) select: xot::NameId,
    pub(crate) separator: xot::NameId,
    pub(crate) stable: xot::NameId,
    pub(crate) static_: xot::NameId,
    pub(crate) streamable: xot::NameId,
    pub(crate) test: xot::NameId,
    pub(crate) type_: xot::NameId,
    pub(crate) tunnel: xot::NameId,
    pub(crate) use_attribute_sets: xot::NameId,
    pub(crate) validation: xot::NameId,
    pub(crate) visibility: xot::NameId,

    // standard attributes on XSLT elements
    pub(crate) standard: StandardNames,
    // standard attributes on literal result elements
    pub(crate) xsl_standard: StandardNames,
}

pub(crate) struct StandardNames {
    pub(crate) default_collation: xot::NameId,
    pub(crate) default_mode: xot::NameId,
    pub(crate) default_validation: xot::NameId,
    pub(crate) exclude_result_prefixes: xot::NameId,
    pub(crate) expand_text: xot::NameId,
    pub(crate) extension_element_prefixes: xot::NameId,
    pub(crate) use_when: xot::NameId,
    pub(crate) version: xot::NameId,
    pub(crate) xpath_default_namespace: xot::NameId,
}

impl StandardNames {
    fn no_ns(xot: &mut Xot) -> Self {
        Self {
            default_collation: xot.add_name("default-collation"),
            default_mode: xot.add_name("default-mode"),
            default_validation: xot.add_name("default-validation"),
            exclude_result_prefixes: xot.add_name("exclude-result-prefixes"),
            expand_text: xot.add_name("expand-text"),
            extension_element_prefixes: xot.add_name("extension-element-prefixes"),
            use_when: xot.add_name("use-when"),
            version: xot.add_name("version"),
            xpath_default_namespace: xot.add_name("xpath-default-namespace"),
        }
    }

    fn xsl(xot: &mut Xot, xsl_ns: NamespaceId) -> Self {
        Self {
            default_collation: xot.add_name_ns("default-collation", xsl_ns),
            default_mode: xot.add_name_ns("default-mode", xsl_ns),
            default_validation: xot.add_name_ns("default-validation", xsl_ns),
            exclude_result_prefixes: xot.add_name_ns("exclude-result-prefixes", xsl_ns),
            expand_text: xot.add_name_ns("expand-text", xsl_ns),
            extension_element_prefixes: xot.add_name_ns("extension-element-prefixes", xsl_ns),
            use_when: xot.add_name_ns("use-when", xsl_ns),
            version: xot.add_name_ns("version", xsl_ns),
            xpath_default_namespace: xot.add_name_ns("xpath-default-namespace", xsl_ns),
        }
    }
}

impl Names {
    pub(crate) fn new(xot: &mut Xot) -> Self {
        let xsl_ns = xot.add_namespace("http://www.w3.org/1999/XSL/Transform");

        Self {
            xsl_ns,

            sequence_constructor_names: SequenceConstructorName::names(xot, xsl_ns),
            declaration_names: DeclarationName::names(xot, xsl_ns),

            xsl_accumulator_rule: xot.add_name_ns("accumulator-rule", xsl_ns),
            xsl_attribute: xot.add_name_ns("attribute", xsl_ns),
            xsl_fallback: xot.add_name_ns("fallback", xsl_ns),
            xsl_matching_substring: xot.add_name_ns("matching-substring", xsl_ns),
            xsl_non_matching_substring: xot.add_name_ns("non-matching-substring", xsl_ns),
            xsl_sort: xot.add_name_ns("sort", xsl_ns),
            xsl_transform: xot.add_name_ns("transform", xsl_ns),
            xsl_with_param: xot.add_name_ns("with-param", xsl_ns),

            as_: xot.add_name("as"),
            case_order: xot.add_name("case-order"),
            collation: xot.add_name("collation"),
            component: xot.add_name("component"),
            copy_accumulators: xot.add_name("copy-accumulators"),
            copy_namespaces: xot.add_name("copy-namespaces"),
            data_type: xot.add_name("data-type"),
            error_code: xot.add_name("error-code"),
            extension_element_prefixes: xot.add_name("extension-element-prefixes"),
            flags: xot.add_name("flags"),
            id: xot.add_name("id"),
            inherit_namespaces: xot.add_name("inherit-namespaces"),
            initial_value: xot.add_name("initial-value"),
            input_type_annotations: xot.add_name("input-type-annotations"),
            lang: xot.add_name("language"),
            match_: xot.add_name("match"),
            mode: xot.add_name("mode"),
            name: xot.add_name("name"),
            names: xot.add_name("names"),
            namespace: xot.add_name("namespace"),
            order: xot.add_name("order"),
            phase: xot.add_name("phase"),
            regex: xot.add_name("regex"),
            select: xot.add_name("select"),
            separator: xot.add_name("separator"),
            stable: xot.add_name("stable"),
            static_: xot.add_name("static"),
            streamable: xot.add_name("streamable"),
            test: xot.add_name("test"),
            tunnel: xot.add_name("tunnel"),
            type_: xot.add_name("type"),
            use_attribute_sets: xot.add_name("use-attribute-sets"),
            validation: xot.add_name("validation"),
            visibility: xot.add_name("visibility"),

            // standard attributes
            standard: StandardNames::no_ns(xot),
            // standard attributes on literal result elements
            xsl_standard: StandardNames::xsl(xot, xsl_ns),
        }
    }

    pub(crate) fn sequence_constructor_name(
        &self,
        name: NameId,
    ) -> Option<SequenceConstructorName> {
        self.sequence_constructor_names.get(&name).copied()
    }

    pub(crate) fn declaration_name(&self, name: NameId) -> Option<DeclarationName> {
        self.declaration_names.get(&name).copied()
    }
}

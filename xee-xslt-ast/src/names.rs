use std::collections::BTreeMap;

use xot::{NameId, NamespaceId, Xot};

use crate::ast_core as ast;
use crate::error::Error;
use crate::instruction::InstructionParser;
use crate::parse::Element;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum SequenceConstructorName {
    Assert,
    Fallback,
    Copy,
    If,
    Variable,
}

impl SequenceConstructorName {
    pub(crate) fn parse(&self, element: &Element) -> Result<ast::SequenceConstructorItem, Error> {
        match self {
            SequenceConstructorName::Assert => ast::Assert::parse(element),
            SequenceConstructorName::Copy => ast::Copy::parse(element),
            SequenceConstructorName::If => ast::If::parse(element),
            SequenceConstructorName::Variable => ast::Variable::parse(element),
            SequenceConstructorName::Fallback => ast::Fallback::parse(element),
        }
    }

    fn names(xot: &mut Xot, xsl_ns: xot::NamespaceId) -> BTreeMap<NameId, SequenceConstructorName> {
        let assert = xot.add_name_ns("assert", xsl_ns);
        let copy = xot.add_name_ns("copy", xsl_ns);
        let fallback = xot.add_name_ns("fallback", xsl_ns);
        let if_ = xot.add_name_ns("if", xsl_ns);
        let variable = xot.add_name_ns("variable", xsl_ns);
        let mut sequence_constructor_names = BTreeMap::new();
        sequence_constructor_names.insert(assert, SequenceConstructorName::Assert);
        sequence_constructor_names.insert(copy, SequenceConstructorName::Copy);
        sequence_constructor_names.insert(fallback, SequenceConstructorName::Fallback);
        sequence_constructor_names.insert(if_, SequenceConstructorName::If);
        sequence_constructor_names.insert(variable, SequenceConstructorName::Variable);
        sequence_constructor_names
    }
}

pub(crate) struct Names {
    pub(crate) xsl_ns: NamespaceId,

    pub(crate) sequence_constructor_names: BTreeMap<NameId, SequenceConstructorName>,

    pub(crate) test: xot::NameId,
    pub(crate) select: xot::NameId,
    pub(crate) name: xot::NameId,
    pub(crate) as_: xot::NameId,
    pub(crate) static_: xot::NameId,
    pub(crate) visibility: xot::NameId,
    pub(crate) copy_namespaces: xot::NameId,
    pub(crate) inherit_namespaces: xot::NameId,
    pub(crate) use_attribute_sets: xot::NameId,
    pub(crate) validation: xot::NameId,
    pub(crate) error_code: xot::NameId,

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

            test: xot.add_name("test"),
            select: xot.add_name("select"),
            name: xot.add_name("name"),
            as_: xot.add_name("as"),
            static_: xot.add_name("static"),
            visibility: xot.add_name("visibility"),
            copy_namespaces: xot.add_name("copy-namespaces"),
            inherit_namespaces: xot.add_name("inherit-namespaces"),
            use_attribute_sets: xot.add_name("use-attribute-sets"),
            validation: xot.add_name("validation"),
            error_code: xot.add_name("error-code"),

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
}

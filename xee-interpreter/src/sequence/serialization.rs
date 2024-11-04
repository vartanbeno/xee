use ahash::HashMap;
use rust_decimal::Decimal;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;
use xot::{xmlname::OwnedName, Xot};

use crate::occurrence::Occurrence;
use crate::{atomic, context, error, function::Map};

enum QNameOrString {
    QName(OwnedName),
    String(String),
}

pub(crate) struct SerializationParameters {
    allow_duplicate_names: bool,
    byte_order_mark: bool,
    cdata_section_elements: Vec<OwnedName>,
    doctype_public: Option<String>,
    doctype_system: Option<String>,
    encoding: String,
    escape_uri_attributes: bool,
    html_version: Decimal,
    include_content_type: bool,
    indent: bool,
    item_separator: String,
    json_node_output_method: QNameOrString,
    media_type: Option<String>,
    method: QNameOrString,
    normalization_form: Option<String>,
    omit_xml_declaration: bool,
    standalone: Option<bool>,
    suppress_indentation: Vec<OwnedName>,
    undeclare_prefixes: bool,
    use_character_maps: HashMap<char, String>,
    version: String,
}

// this would be prettier with some fancy derive macro, but we don't
// have that many functions that need this and this is easier to make.
macro_rules! option_parameter_conversion_option_with_default {
    ($map:ident, $xpath_name:literal, $atomic:expr, $ty:ty, $default:expr, $static_context:ident, $xot:ident) => {{
        let name: atomic::Atomic = $xpath_name.to_string().into();
        let value = $map.get_as_type(
            &name,
            ast::Occurrence::Option,
            $atomic,
            $static_context,
            $xot,
        )?;
        let value = if let Some(value) = value {
            value.items()?.option()?
        } else {
            None
        };
        let value: $ty = if let Some(value) = value {
            value.to_atomic()?.try_into()?
        } else {
            $default
        };
        value
    }};
}

macro_rules! option_parameter_conversion_option {
    ($map:ident, $xpath_name:literal, $atomic:expr, $ty:ty, $static_context:ident, $xot:ident) => {{
        let name: atomic::Atomic = $xpath_name.to_string().into();
        let value = $map.get_as_type(
            &name,
            ast::Occurrence::Option,
            $atomic,
            $static_context,
            $xot,
        )?;
        let value = if let Some(value) = value {
            value.items()?.option()?
        } else {
            None
        };
        let value: Option<$ty> = if let Some(value) = value {
            Some(value.to_atomic()?.try_into()?)
        } else {
            None
        };
        value
    }};
}

impl SerializationParameters {
    fn from_map(
        map: Map,
        static_context: &context::StaticContext,
        xot: &Xot,
    ) -> error::Result<Self> {
        let allow_duplicate_names = option_parameter_conversion_option_with_default!(
            map,
            "allow-duplicate-names",
            Xs::Boolean,
            bool,
            false,
            static_context,
            xot
        );
        let byte_order_mark = option_parameter_conversion_option_with_default!(
            map,
            "byte-order-mark",
            Xs::Boolean,
            bool,
            false,
            static_context,
            xot
        );

        // cdata-section-elements

        let doctype_public = option_parameter_conversion_option!(
            map,
            "doctype-public",
            Xs::String,
            String,
            static_context,
            xot
        );

        let doctype_system = option_parameter_conversion_option!(
            map,
            "doctype-system",
            Xs::String,
            String,
            static_context,
            xot
        );

        let encoding = option_parameter_conversion_option_with_default!(
            map,
            "encoding",
            Xs::String,
            String,
            "utf-8".to_string(),
            static_context,
            xot
        );

        let escape_uri_attributes = option_parameter_conversion_option_with_default!(
            map,
            "escape-uri-attributes",
            Xs::Boolean,
            bool,
            true,
            static_context,
            xot
        );

        let html_version = option_parameter_conversion_option_with_default!(
            map,
            "html-version",
            Xs::Decimal,
            Decimal,
            Decimal::from_str_exact("5.0").unwrap(),
            static_context,
            xot
        );

        let include_content_type = option_parameter_conversion_option_with_default!(
            map,
            "include-content-type",
            Xs::Boolean,
            bool,
            true,
            static_context,
            xot
        );

        let indent = option_parameter_conversion_option_with_default!(
            map,
            "indent",
            Xs::Boolean,
            bool,
            false,
            static_context,
            xot
        );

        let item_separator = option_parameter_conversion_option_with_default!(
            map,
            "item-separator",
            Xs::String,
            String,
            " ".to_string(),
            static_context,
            xot
        );

        // json-node-output-method

        let media_type = option_parameter_conversion_option!(
            map,
            "media-type",
            Xs::String,
            String,
            static_context,
            xot
        );

        // method

        let normalization_form = option_parameter_conversion_option!(
            map,
            "normalization-form",
            Xs::String,
            String,
            static_context,
            xot
        );

        let omit_xml_declaration = option_parameter_conversion_option_with_default!(
            map,
            "omit-xml-declaration",
            Xs::Boolean,
            bool,
            true,
            static_context,
            xot
        );

        let standalone = option_parameter_conversion_option!(
            map,
            "standalone",
            Xs::Boolean,
            bool,
            static_context,
            xot
        );

        // suppress-indentation

        let undeclare_prefixes = option_parameter_conversion_option_with_default!(
            map,
            "undeclare-prefixes",
            Xs::Boolean,
            bool,
            false,
            static_context,
            xot
        );

        // use-character-maps

        let version = option_parameter_conversion_option_with_default!(
            map,
            "version",
            Xs::String,
            String,
            "1.0".to_string(),
            static_context,
            xot
        );

        Ok(Self {
            allow_duplicate_names,
            byte_order_mark,
            cdata_section_elements: Vec::new(),
            doctype_public,
            doctype_system,
            encoding,
            escape_uri_attributes,
            html_version,
            include_content_type,
            indent,
            item_separator,
            json_node_output_method: QNameOrString::String("xml".to_string()),
            media_type,
            method: QNameOrString::String("xml".to_string()),
            normalization_form,
            omit_xml_declaration,
            standalone,
            suppress_indentation: Vec::new(),
            undeclare_prefixes,
            use_character_maps: HashMap::default(),
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::sequence;

    use super::*;

    #[test]
    fn test_allow_duplicate_names_true() {
        let map = Map::new(vec![(
            "allow-duplicate-names".to_string().into(),
            sequence::Sequence::from(vec![atomic::Atomic::Boolean(true)]),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert!(params.allow_duplicate_names);
    }

    #[test]
    fn test_allow_duplicate_names_false() {
        let map = Map::new(vec![(
            "allow-duplicate-names".to_string().into(),
            sequence::Sequence::from(vec![atomic::Atomic::Boolean(false)]),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert!(!params.allow_duplicate_names);
    }

    #[test]
    fn test_allow_duplicate_names_default() {
        let map = Map::new(vec![(
            "allow-duplicate-names".to_string().into(),
            sequence::Sequence::default(),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert!(!params.allow_duplicate_names);
    }

    #[test]
    fn test_allow_duplicate_names_empty_sequence() {
        let empty_vec: Vec<atomic::Atomic> = Vec::new();
        let map = Map::new(vec![(
            "allow-duplicate-names".to_string().into(),
            sequence::Sequence::from(empty_vec),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert!(!params.allow_duplicate_names);
    }
}

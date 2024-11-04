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
    escape_uri_attribtue: bool,
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
macro_rules! option_parameter_conversion_option {
    ($map:ident, $xpath_name:literal, $occurrence:expr, $atomic:expr, $ty:ty, $default:literal, $static_context:ident, $xot:ident) => {{
        let name: atomic::Atomic = $xpath_name.to_string().into();
        let value = $map.get_as_type(&name, $occurrence, $atomic, $static_context, $xot)?;
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

impl SerializationParameters {
    fn from_map(
        map: Map,
        static_context: &context::StaticContext,
        xot: &Xot,
    ) -> error::Result<Self> {
        let allow_duplicate_names = option_parameter_conversion_option!(
            map,
            "allow-duplicate-names",
            ast::Occurrence::Option,
            Xs::Boolean,
            bool,
            false,
            static_context,
            xot
        );

        // let allow_duplicate_names: atomic::Atomic = "allow-duplicate-names".to_string().into();
        // let allow_duplicate_names = map.get_as_type(
        //     &allow_duplicate_names,
        //     ast::Occurrence::Option,
        //     Xs::Boolean,
        //     static_context,
        //     xot,
        // )?;
        // let allow_duplicate_names = if let Some(allow_duplicate_names) = allow_duplicate_names {
        //     allow_duplicate_names.items()?.option()?
        // } else {
        //     None
        // };
        // let allow_duplicate_names: bool = if let Some(allow_duplicate_names) = allow_duplicate_names
        // {
        //     allow_duplicate_names.to_atomic()?.try_into()?
        // } else {
        //     false
        // };
        Ok(Self {
            allow_duplicate_names,
            byte_order_mark: false,
            cdata_section_elements: Vec::new(),
            doctype_public: None,
            doctype_system: None,
            encoding: "UTF-8".to_string(),
            escape_uri_attribtue: false,
            html_version: Decimal::ZERO,
            include_content_type: false,
            indent: false,
            item_separator: " ".to_string(),
            json_node_output_method: QNameOrString::String("xml".to_string()),
            media_type: None,
            method: QNameOrString::String("xml".to_string()),
            normalization_form: None,
            omit_xml_declaration: false,
            standalone: None,
            suppress_indentation: Vec::new(),
            undeclare_prefixes: false,
            use_character_maps: HashMap::default(),
            version: "1.0".to_string(),
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

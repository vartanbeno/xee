use ahash::HashMap;
use rust_decimal::Decimal;
use xot::{xmlname::OwnedName, Xot};

use xee_schema_type::Xs;

use crate::{context, error, function::Map};

use super::opc::{OptionParameterConverter, QNameOrString};

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

impl SerializationParameters {
    fn from_map(
        map: Map,
        static_context: &context::StaticContext,
        xot: &Xot,
    ) -> error::Result<Self> {
        let c = OptionParameterConverter::new(&map, static_context, xot);
        let allow_duplicate_names =
            c.option_with_default("allow-duplicate-names", Xs::Boolean, false)?;

        let byte_order_mark = c.option_with_default("byte-order-mark", Xs::Boolean, false)?;

        let cdata_section_elements = c.many("cdata-section-elements", Xs::QName)?;

        let doctype_public = c.option("doctype-public", Xs::String)?;

        let doctype_system = c.option("doctype-system", Xs::String)?;

        let encoding = c.option_with_default("encoding", Xs::String, "utf-8".to_string())?;

        let escape_uri_attributes =
            c.option_with_default("escape-uri-attributes", Xs::Boolean, true)?;

        let html_version = c.option_with_default(
            "html-version",
            Xs::Decimal,
            Decimal::from_str_exact("5.0").unwrap(),
        )?;

        let include_content_type =
            c.option_with_default("include-content-type", Xs::Boolean, true)?;

        let indent = c.option_with_default("indent", Xs::Boolean, false)?;

        let item_separator =
            c.option_with_default("item-separator", Xs::String, " ".to_string())?;

        let json_node_output_method = c.qname_or_string(
            "json-node-output-method",
            QNameOrString::String("xml".to_string()),
        )?;

        let media_type = c.option("media-type", Xs::String)?;

        let method = c.qname_or_string("method", QNameOrString::String("xml".to_string()))?;

        let normalization_form = c.option("normalization-form", Xs::String)?;

        let omit_xml_declaration =
            c.option_with_default("omit-xml-declaration", Xs::Boolean, true)?;

        let standalone = c.option("standalone", Xs::Boolean)?;

        let suppress_indentation = c.many("suppress-indentation", Xs::QName)?;

        let undeclare_prefixes = c.option_with_default("undeclare-prefixes", Xs::Boolean, false)?;

        // use-character-maps

        let version = c.option_with_default("version", Xs::String, "1.0".to_string())?;

        Ok(Self {
            allow_duplicate_names,
            byte_order_mark,
            cdata_section_elements,
            doctype_public,
            doctype_system,
            encoding,
            escape_uri_attributes,
            html_version,
            include_content_type,
            indent,
            item_separator,
            json_node_output_method,
            media_type,
            method,
            normalization_form,
            omit_xml_declaration,
            standalone,
            suppress_indentation,
            undeclare_prefixes,
            use_character_maps: HashMap::default(),
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{atomic, sequence};

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
    fn test_allow_duplicate_names_default_empty_sequence() {
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
    fn test_allow_duplicate_names_missing() {
        let map = Map::new(vec![]).unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert!(!params.allow_duplicate_names);
    }

    #[test]
    fn test_cdata_section_elements() {
        let html = OwnedName::new("html".to_string(), "".to_string(), "".to_string());
        let script = OwnedName::new("script".to_string(), "".to_string(), "".to_string());
        let map = Map::new(vec![(
            "cdata-section-elements".to_string().into(),
            sequence::Sequence::from(vec![
                atomic::Atomic::QName(html.clone().into()),
                atomic::Atomic::QName(script.clone().into()),
            ]),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert_eq!(params.cdata_section_elements.len(), 2);
        assert_eq!(params.cdata_section_elements[0], html);
        assert_eq!(params.cdata_section_elements[1], script);
    }

    #[test]
    fn test_qname_or_string_string() {
        let json: atomic::Atomic = "json".to_string().into();
        let map = Map::new(vec![(
            "json-node-output-method".to_string().into(),
            sequence::Sequence::from(vec![json]),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert_eq!(
            params.json_node_output_method,
            QNameOrString::String("json".to_string())
        );
    }

    #[test]
    fn test_qname_or_string_qname() {
        let owned_name = OwnedName::new("json".to_string(), "".to_string(), "".to_string());
        let json: atomic::Atomic = owned_name.clone().into();
        let map = Map::new(vec![(
            "json-node-output-method".to_string().into(),
            sequence::Sequence::from(vec![json]),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert_eq!(
            params.json_node_output_method,
            QNameOrString::QName(owned_name)
        );
    }

    #[test]
    fn test_qname_or_string_default_empty_sequence() {
        let map = Map::new(vec![(
            "json-node-output-method".to_string().into(),
            sequence::Sequence::default(),
        )])
        .unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert_eq!(
            params.json_node_output_method,
            QNameOrString::String("xml".to_string())
        );
    }

    #[test]
    fn test_qname_or_string_default_missing() {
        let map = Map::new(vec![]).unwrap();
        let static_context = context::StaticContext::default();
        let xot = Xot::new();
        let params = SerializationParameters::from_map(map, &static_context, &xot).unwrap();
        assert_eq!(
            params.json_node_output_method,
            QNameOrString::String("xml".to_string())
        );
    }
}

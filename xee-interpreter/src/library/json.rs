use xee_xpath_macros::xpath_fn;

use crate::{atomic, error, function, sequence, wrap_xpath_fn};

use super::StaticFunctionDescription;

#[xpath_fn("fn:parse-json($json_text as xs:string?) as item()?")]
fn parse_json1(json_text: Option<&str>) -> error::Result<Option<sequence::Item>> {
    if let Some(json_text) = json_text {
        let value = json::parse(json_text).map_err(|_| error::Error::FOJS0001)?;
        Ok(parse_json_value(&value)?)
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:parse-json($json_text as xs:string?, $options as map(*)) as item()?")]
fn parse_json2(
    json_text: Option<&str>,
    _options: function::Map,
) -> error::Result<Option<sequence::Item>> {
    // TODO: ignore parameters for now
    if let Some(json_text) = json_text {
        let value = json::parse(json_text).map_err(|_| error::Error::FOJS0001)?;
        Ok(parse_json_value(&value)?)
    } else {
        Ok(None)
    }
}

fn parse_json_value(value: &json::JsonValue) -> error::Result<Option<sequence::Item>> {
    match value {
        json::JsonValue::Null => Ok(None),
        json::JsonValue::Short(s) => {
            let atomic: atomic::Atomic = s.to_string().into();
            Ok(Some(atomic.into()))
        }
        json::JsonValue::String(s) => {
            let atomic: atomic::Atomic = s.to_string().into();
            Ok(Some(atomic.into()))
        }
        json::JsonValue::Number(n) => {
            let f: f64 = (*n).into();
            let atomic: atomic::Atomic = f.into();
            Ok(Some(atomic.into()))
        }
        json::JsonValue::Boolean(b) => {
            let atomic = atomic::Atomic::Boolean(*b);
            Ok(Some(atomic.into()))
        }
        json::JsonValue::Array(a) => {
            let mut entries = Vec::with_capacity(a.len());
            for value in a.iter() {
                let value = parse_json_value(value)?;
                let sequence: sequence::Sequence = value.into();
                entries.push(sequence);
            }
            let array = function::Array::new(entries);
            let function = function::Function::Array(array);
            Ok(Some(function.into()))
        }
        json::JsonValue::Object(o) => {
            let mut entries = Vec::with_capacity(o.len());
            for (key, value) in o.iter() {
                let key: atomic::Atomic = key.to_string().into();
                let value = parse_json_value(value)?;
                let sequence: sequence::Sequence = value.into();
                entries.push((key.clone(), sequence));
            }
            let map = function::Map::new(entries)?;
            let function = function::Function::Map(map);
            Ok(Some(function.into()))
        }
    }
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![wrap_xpath_fn!(parse_json1), wrap_xpath_fn!(parse_json2)]
}

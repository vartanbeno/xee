// https://www.w3.org/TR/xpath-functions-31/#parsing-and-serializing

use xee_xpath_macros::xpath_fn;
use xot::Xot;

use crate::{
    context, error, function,
    interpreter::Interpreter,
    sequence::{self, SerializationParameters},
    wrap_xpath_fn,
};

use super::StaticFunctionDescription;

#[xpath_fn("fn:parse-xml($arg as xs:string?) as document-node(element(*))?")]
fn parse_xml(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    arg: Option<&str>,
) -> error::Result<Option<xot::Node>> {
    if let Some(arg) = arg {
        let documents = context.documents();
        let handle = documents
            .borrow_mut()
            .add_string(interpreter.xot_mut(), None, arg)
            .map_err(|_| error::Error::FODC0006)?;
        let doc = documents
            .borrow()
            .get_node_by_handle(handle)
            .ok_or(error::Error::FODC0006)?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:parse-xml-fragment($arg as xs:string?) as document-node()?")]
fn parse_xml_fragment(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    arg: Option<&str>,
) -> error::Result<Option<xot::Node>> {
    if let Some(arg) = arg {
        let documents = context.documents();
        let handle = documents
            .borrow_mut()
            .add_fragment_string(interpreter.xot_mut(), arg)
            .map_err(|_| error::Error::FODC0006)?;
        let doc = documents
            .borrow()
            .get_node_by_handle(handle)
            .ok_or(error::Error::FODC0006)?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

#[xpath_fn("fn:serialize($arg as item()*) as xs:string")]
fn serialize1(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    arg: &sequence::Sequence,
) -> error::Result<String> {
    let map = function::Map::new(vec![])?;
    let serialization_parameters =
        SerializationParameters::from_map(map, context.static_context(), interpreter.xot_mut())?;
    serialize_helper(arg, serialization_parameters, interpreter.xot_mut())
}

#[xpath_fn("fn:serialize($arg as item()*, $params as item()?) as xs:string")]
fn serialize2(
    context: &context::DynamicContext,
    interpreter: &mut Interpreter,
    arg: &sequence::Sequence,
    params: Option<sequence::Item>,
) -> error::Result<String> {
    let map = if let Some(params) = params {
        if let sequence::Item::Function(function) = params {
            if let function::Function::Map(map) = function.as_ref() {
                map.clone()
            } else {
                return Err(error::Error::XPTY0004);
            }
        } else {
            // TODO: handle element(output::serialization-parameters)
            return Err(error::Error::XPTY0004);
        }
    } else {
        function::Map::new(vec![])?
    };
    let serialization_parameters =
        SerializationParameters::from_map(map, context.static_context(), interpreter.xot_mut())?;
    serialize_helper(arg, serialization_parameters, interpreter.xot_mut())
}

fn serialize_helper(
    arg: &sequence::Sequence,
    parameters: SerializationParameters,
    xot: &mut Xot,
) -> error::Result<String> {
    let node = arg.normalize(&parameters.item_separator, xot)?;

    let suppress = parameters
        .suppress_indentation
        .iter()
        .map(|owned_name| owned_name.to_ref(xot).name_id())
        .collect::<Vec<_>>();
    let cdata_section_elements = parameters
        .cdata_section_elements
        .iter()
        .map(|owned_name| owned_name.to_ref(xot).name_id())
        .collect::<Vec<_>>();
    let declaration = if !parameters.omit_xml_declaration {
        Some(xot::output::xml::Declaration {
            encoding: Some(parameters.encoding.to_string()),
            standalone: parameters.standalone,
        })
    } else {
        None
    };
    let doctype = match (parameters.doctype_public, parameters.doctype_system) {
        (Some(public), Some(system)) => Some(xot::output::xml::DocType::Public { public, system }),
        (None, Some(system)) => Some(xot::output::xml::DocType::System { system }),
        // TODO: this should really not happen?
        (Some(public), None) => Some(xot::output::xml::DocType::Public {
            public,
            system: "".to_string(),
        }),
        (None, None) => None,
    };
    let output_parameters = xot::output::xml::Parameters {
        indentation: Some(xot::output::Indentation { suppress }),
        cdata_section_elements,
        declaration,
        doctype,
        ..Default::default()
    };
    Ok(xot.serialize_xml_string(output_parameters, node)?)
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(parse_xml),
        wrap_xpath_fn!(parse_xml_fragment),
        wrap_xpath_fn!(serialize1),
        wrap_xpath_fn!(serialize2),
    ]
}

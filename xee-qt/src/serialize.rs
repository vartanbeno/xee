use xee_interpreter::{sequence::Sequence, xml::Node};
use xot::Xot;

use crate::error::{Error, Result};

// represent items as XML, if possible, wrapped
// in a sequence tag
pub(crate) fn serialize(xot: &Xot, sequence: &Sequence) -> Result<String> {
    let mut xmls = Vec::with_capacity(sequence.len());
    for item in sequence.items() {
        if let Ok(Node::Xot(node)) = item?.to_node() {
            let xml_value = xot.to_string(node);
            if let Ok(xml_value) = xml_value {
                xmls.push(xml_value);
            } else {
                return Err(Error::CannotRepresentAsXml);
            }
        } else {
            return Err(Error::CannotRepresentAsXml);
        }
    }
    Ok(format!("<sequence>{}</sequence>", xmls.join("")))
}

use miette::{miette, Result};
use xee_xpath::{Atomic, Item, Node, StackValue};
use xot::Xot;

// represent a stack value as XML, if possible, wrapped
// in a sequence tag
pub(crate) fn serialize(xot: &Xot, value: &StackValue) -> Result<String> {
    let xmls = match value {
        StackValue::Atomic(Atomic::Empty) => vec![],
        StackValue::Node(Node::Xot(node)) => {
            let xml_value = xot.to_string(*node);
            if let Ok(xml_value) = xml_value {
                vec![xml_value]
            } else {
                return Err(miette!("cannot be represented as XML"));
            }
        }
        StackValue::Sequence(seq) => {
            let seq = seq.borrow();
            let mut xmls = Vec::with_capacity(seq.len());
            for item in seq.as_slice().iter() {
                if let Item::Node(Node::Xot(node)) = item {
                    let xml_value = xot.to_string(*node);
                    if let Ok(xml_value) = xml_value {
                        xmls.push(xml_value);
                    } else {
                        return Err(miette!("cannot be represented as XML"));
                    }
                } else {
                    return Err(miette!("cannot be represented as XML"));
                }
            }
            xmls
        }
        _ => {
            // cannot be represented as XML
            return Err(miette!("cannot be represented as XML"));
        }
    };
    Ok(format!("<sequence>{}</sequence>", xmls.join("")))
}

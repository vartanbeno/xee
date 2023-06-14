use miette::{miette, Result};
use xee_xpath::{Node, OutputItem as Item, OutputSequence as Sequence};
use xot::Xot;

// represent items as XML, if possible, wrapped
// in a sequence tag
pub(crate) fn serialize(xot: &Xot, sequence: &Sequence) -> Result<String> {
    let items = sequence.items();
    let mut xmls = Vec::with_capacity(items.len());
    for item in items {
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
    Ok(format!("<sequence>{}</sequence>", xmls.join("")))
}

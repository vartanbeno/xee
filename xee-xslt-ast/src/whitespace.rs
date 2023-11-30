use xot::{Node, NodeEdge, Value, Xot};

use crate::names::Names;

pub(crate) fn strip_whitespace(xot: &mut Xot, names: &Names, node: Node) {
    // all comments and processing instructions are removed
    // any text nodes that are now adjacent to each other are merged
    let mut to_remove = vec![];
    for edge in xot.traverse(node) {
        match edge {
            NodeEdge::Start(node) => match xot.value(node) {
                Value::Root => {}
                Value::Comment(..) => {
                    to_remove.push(node);
                }
                Value::ProcessingInstruction(..) => {
                    to_remove.push(node);
                }
                Value::Text(text) => {
                    if is_xml_whitespace(text.get()) {
                        to_remove.push(node);
                    }
                }
                Value::Element(..) => {}
            },
            NodeEdge::End(node) => {}
        }
    }
    for node in to_remove {
        let _ = xot.remove(node);
    }
}

fn is_xml_whitespace_char(c: char) -> bool {
    matches!(c, '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}')
}

fn is_xml_whitespace(s: &str) -> bool {
    s.chars().all(is_xml_whitespace_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_comments() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><!--comment--></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc/>");
    }

    #[test]
    fn test_remove_processing_instructions() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><p>A<?pi?>B</p></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc><p>AB</p></doc>");
    }

    #[test]
    fn test_remove_whitespace() {
        let mut xot = Xot::new();
        let names = Names::new(&mut xot);
        let root = xot.parse(r#"<doc><p>   </p></doc>"#).unwrap();
        strip_whitespace(&mut xot, &names, root);
        assert_eq!(xot.to_string(root).unwrap(), "<doc><p/></doc>");
    }
}

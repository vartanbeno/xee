use xot::{Element, Node, NodeEdge, Xot};

#[derive(Debug)]
enum Error {
    Xot(xot::Error),
}

impl From<xot::Error> for Error {
    fn from(error: xot::Error) -> Self {
        Self::Xot(error)
    }
}

// conditional element inclusion and pre-processing shadow-attributes
fn preprocess(xot: &mut Xot, top: Node) -> Result<(), Error> {
    let xsl_ns = xot.add_namespace("http://www.w3.org/1999/XSL/Transform");
    let instruction_use_when = xot.add_name("use-when");
    let xsl_use_when = xot.add_name_ns("use-when", xsl_ns);
    let in_xsl_ns = |element: &Element| {
        let name = element.name();
        let namespace = xot.namespace_for_name(name);
        namespace == xsl_ns
    };

    let mut to_remove = Vec::new();
    let mut to_remove_attribute = Vec::new();

    let mut stack = Vec::new();

    for edge in xot.traverse(top) {
        match edge {
            NodeEdge::Start(node) => {
                // if we're in a removed element, we don't need to
                // do anything as it's all going to disappear
                if let Some(in_removed) = stack.last() {
                    if *in_removed {
                        continue;
                    }
                }
                if let Some(element) = xot.element(node) {
                    let use_when = if in_xsl_ns(element) {
                        instruction_use_when
                    } else {
                        xsl_use_when
                    };
                    if let Some(value) = element.attributes().get(&use_when) {
                        // TODO: replace with real xpath evaluation
                        if value == "false()" {
                            to_remove.push(node);
                            stack.push(true);
                        } else {
                            to_remove_attribute.push((node, use_when));
                            stack.push(false);
                        }
                    } else {
                        stack.push(false);
                    }
                }
            }
            NodeEdge::End(_) => {
                stack.pop();
            }
        }
    }

    for node in to_remove {
        xot.remove(node)?;
    }

    for (node, name) in to_remove_attribute {
        if let Some(element) = xot.element_mut(node) {
            element.remove_attribute(name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_when_without_content() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if use-when="false()"></xsl:if></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#
        );
    }

    #[test]
    fn test_use_when_with_content() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if use-when="false()"><p/></xsl:if></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#
        );
    }

    #[test]
    fn test_nested_use_when() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if use-when="false()"><xsl:if use-when="false()"><p/></xsl:if></xsl:if></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#
        );
    }

    #[test]
    fn test_xsl_use_when_on_literal_element() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><foo xsl:use-when="false()"/></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#
        );
    }

    #[test]
    fn test_xsl_use_when_on_instruction_is_ignored() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if xsl:use-when="false()"/></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if xsl:use-when="false()"/></xsl:transform>"#
        );
    }

    #[test]
    fn test_unprefixed_use_when_on_literal_element_is_ignored() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><p use-when="false()"/></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><p use-when="false()"/></xsl:transform>"#
        );
    }

    #[test]
    fn test_use_when_is_stripped_when_true() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if use-when="true()"/></xsl:transform>"#).unwrap();
        preprocess(&mut xot, doc).unwrap();
        assert_eq!(
            xot.to_string(doc).unwrap(),
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:if/></xsl:transform>"#
        );
    }
}

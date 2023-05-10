use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xee_xpath::{DynamicContext, Item, Namespaces, Node, StaticContext, XPath};
use xot::Xot;

use crate::qt;

const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

fn load_from_file(xot: &mut Xot, path: &Path) -> Result<Vec<qt::TestCase>> {
    let xml_file = File::open(path)
        .into_diagnostic()
        .wrap_err("Cannot open XML file")?;
    let mut buf_reader = BufReader::new(xml_file);
    let mut xml = String::new();
    buf_reader
        .read_to_string(&mut xml)
        .into_diagnostic()
        .wrap_err("Cannot read XML file")?;
    load_from_xml(xot, &xml)
}

fn load_from_xml(xot: &mut Xot, xml: &str) -> Result<Vec<qt::TestCase>> {
    let root = xot
        .parse(xml)
        .into_diagnostic()
        .wrap_err("Cannot parse XML")?;
    let root = Node::Xot(root);
    let namespaces = Namespaces::new(Some(NS), None);
    let loader = Loader::new(&namespaces);
    let xpaths = XPaths::new(&loader.static_context)?;

    loader.test_cases(xot, &xpaths, root)
}

struct Loader<'a> {
    static_context: StaticContext<'a>,
}

impl<'a> Loader<'a> {
    fn new(namespaces: &'a Namespaces<'a>) -> Self {
        let static_context = StaticContext::new(namespaces);
        Self { static_context }
    }

    fn test_cases(
        &self,
        xot: &Xot,
        xpaths: &'a XPaths<'a>,
        node: Node,
    ) -> Result<Vec<qt::TestCase>> {
        let dynamic_context = DynamicContext::new(xot, &self.static_context);
        xpaths
            .test_cases
            .many(&dynamic_context, &Item::Node(node))?
            .iter()
            .map(|n| {
                Ok(qt::TestCase {
                    name: xpaths
                        .name
                        .one(&dynamic_context, n)?
                        .as_atomic()?
                        .as_string()?,
                    description: "".to_string(),
                    created: qt::Attribution {
                        by: "".to_string(),
                        on: "".to_string(),
                    },
                    modified: Vec::new(),
                    environments: Vec::new(),
                    dependencies: Vec::new(),
                    modules: Vec::new(),
                    test: "".to_string(),
                    result: qt::TestCaseResult::AssertTrue,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

struct XPaths<'a> {
    test_cases: XPath<'a>,
    name: XPath<'a>,
}

impl<'a> XPaths<'a> {
    fn new(static_context: &'a StaticContext<'a>) -> Result<Self> {
        Ok(XPaths {
            test_cases: XPath::new(static_context, "/test-set/test-case")?,
            name: XPath::new(static_context, "@name/string()")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_debug_snapshot;

    const ROOT_FIXTURE: &str = include_str!("fixtures/root.xml");

    #[test]
    fn test_load() {
        let mut xot = Xot::new();
        assert_debug_snapshot!(load_from_xml(&mut xot, ROOT_FIXTURE));
    }
}

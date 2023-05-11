use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xee_xpath::{
    ConvertError, DynamicContext, Item, ManyQuery, Namespaces, Node, OneQuery, StaticContext,
};
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
    let namespaces = Namespaces::with_default_element_namespace(NS);
    let loader = Loader::new(&namespaces);

    let dynamic_context = DynamicContext::new(xot, &loader.static_context);
    loader.test_cases(&dynamic_context, root)
}

struct Loader<'a> {
    static_context: StaticContext<'a>,
}

fn convert_string(item: &Item) -> Result<String, ConvertError> {
    Ok(item.as_atomic()?.as_string()?)
}

impl<'a> Loader<'a> {
    fn new(namespaces: &'a Namespaces<'a>) -> Self {
        let static_context = StaticContext::new(namespaces);
        Self { static_context }
    }

    fn test_cases(
        &self,
        dynamic_context: &'a DynamicContext<'a>,
        node: Node,
    ) -> Result<Vec<qt::TestCase>> {
        let name_query = OneQuery::new(&self.static_context, "@name/string()", convert_string)?;
        let description_query =
            OneQuery::new(&self.static_context, "description/string()", convert_string)?;
        let test_query = OneQuery::new(&self.static_context, "test/string()", convert_string)?;
        let test_cases_query =
            ManyQuery::new(&self.static_context, "/test-set/test-case", |item| {
                Ok(qt::TestCase {
                    name: name_query.execute(dynamic_context, item)?,
                    description: description_query.execute(dynamic_context, item)?,
                    created: qt::Attribution {
                        by: "".to_string(),
                        on: "".to_string(),
                    },
                    modified: Vec::new(),
                    environments: Vec::new(),
                    dependencies: Vec::new(),
                    modules: Vec::new(),
                    test: test_query.execute(dynamic_context, item)?,
                    result: qt::TestCaseResult::AssertTrue,
                })
            })?;
        Ok(test_cases_query.execute(dynamic_context, &Item::Node(node))?)
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
        assert_debug_snapshot!(load_from_xml(&mut xot, ROOT_FIXTURE).unwrap());
    }
}

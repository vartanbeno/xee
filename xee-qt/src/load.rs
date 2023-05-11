use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xee_xpath::{
    Convert, ConvertError, DynamicContext, Item, ManyQuery, Namespaces, Node, OneQuery,
    StaticContext,
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

    let static_context = StaticContext::new(&namespaces);

    // let loader = Loader::new(query)?;

    let query = test_cases_query(&static_context)?;
    let dynamic_context = DynamicContext::new(xot, &static_context);
    // the query has a lifetime for the dynamic context, and a lifetime
    // for the static context
    let r = query.execute(&dynamic_context, &Item::Node(root))?;
    Ok(r)
    // loader.test_cases(&dynamic_context, root)
}

fn convert_string(_: &DynamicContext, item: &Item) -> Result<String, ConvertError> {
    Ok(item.as_atomic()?.as_string()?)
}

fn test_cases_query<'s>(
    static_context: &'s StaticContext<'s>,
) -> Result<ManyQuery<'s, qt::TestCase, impl Convert<qt::TestCase> + 's>> {
    let name_query = OneQuery::new(static_context, "@name/string()", convert_string)?;
    let description_query = OneQuery::new(static_context, "description/string()", convert_string)?;
    let test_query = OneQuery::new(static_context, "test/string()", convert_string)?;
    let created_query = OneQuery::new(static_context, "created", |dynamic_context, item| {
        let by_query = OneQuery::new(static_context, "@by/string()", convert_string)?;
        let on_query = OneQuery::new(static_context, "@on/string()", convert_string)?;
        Ok(qt::Attribution {
            by: by_query.execute(dynamic_context, item)?,
            on: on_query.execute(dynamic_context, item)?,
        })
    })?;
    let change_query = OneQuery::new(static_context, "@change/string()", convert_string)?;
    // XXX this duplication is required to support the move, which is required
    // to make the lifetimes work, but it's still a pain. it's better than
    // creating (and thus compiling) the query inside the closure, as
    // that would have a compile per convert!
    let by_query = OneQuery::new(static_context, "@by/string()", convert_string)?;
    let on_query = OneQuery::new(static_context, "@on/string()", convert_string)?;
    let modified_query =
        ManyQuery::new(static_context, "modified", move |dynamic_context, item| {
            let attribution = qt::Attribution {
                by: by_query.execute(dynamic_context, item)?,
                on: on_query.execute(dynamic_context, item)?,
            };
            let description = change_query.execute(dynamic_context, item)?;
            Ok(qt::Modification {
                attribution,
                description,
            })
        })?;

    let type_query = OneQuery::new(static_context, "@type/string()", convert_string)?;
    let value_query = OneQuery::new(static_context, "@value/string()", convert_string)?;
    let dependency_query = ManyQuery::new(
        static_context,
        "dependency",
        move |dynamic_context, item| {
            Ok(qt::Dependency {
                type_: type_query.execute(dynamic_context, item)?,
                value: value_query.execute(dynamic_context, item)?,
            })
        },
    )?;

    Ok(ManyQuery::new(
        static_context,
        "/test-set/test-case",
        move |dynamic_context, item| {
            Ok(qt::TestCase {
                name: name_query.execute(dynamic_context, item)?,
                description: description_query.execute(dynamic_context, item)?,
                created: created_query.execute(dynamic_context, item)?,
                modified: modified_query.execute(dynamic_context, item)?,
                environments: Vec::new(),
                dependencies: dependency_query.execute(dynamic_context, item)?,
                modules: Vec::new(),
                test: test_query.execute(dynamic_context, item)?,
                result: qt::TestCaseResult::AssertTrue,
            })
        },
    )?)
}

// struct Loader<'a, F>
// where
//     F: Convert<'a, 'a, qt::TestCase>,
// {
//     test_cases_query: ManyQuery<'a, qt::TestCase, F>,
// }

// impl<'a, F> Loader<'a, F>
// where
//     F: Convert<'a, 'a, qt::TestCase>,
// {
//     fn new(test_cases_query: ManyQuery<'a, qt::TestCase, F>) -> Result<Self> {
//         Ok(Self { test_cases_query })
//     }

//     fn test_cases(
//         &self,
//         dynamic_context: &'a DynamicContext<'a>,
//         node: Node,
//     ) -> Result<Vec<qt::TestCase>> {
//         Ok(self
//             .test_cases_query
//             .execute(dynamic_context, &Item::Node(node))?)
//     }
// }

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

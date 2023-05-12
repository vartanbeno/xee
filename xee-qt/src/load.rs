use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xee_xpath::Session;
use xee_xpath::{
    Convert, ConvertError, DynamicContext, Item, ManyQuery, Namespaces, Node, OneQuery,
    OptionQuery, Queries, StaticContext,
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

    let queries = Queries::new(&static_context);

    let (queries, query) = test_cases_query(queries)?;

    let dynamic_context = DynamicContext::new(xot, &static_context);
    let session = queries.session(&dynamic_context);
    // the query has a lifetime for the dynamic context, and a lifetime
    // for the static context
    let r = query.execute(&session, &Item::Node(root))?;
    Ok(r)
    // loader.test_cases(&dynamic_context, root)
}

fn convert_string(_: &Session, item: &Item) -> Result<String, ConvertError> {
    Ok(item.as_atomic()?.as_string()?)
}

fn test_cases_query(
    mut queries: Queries<'_>,
) -> Result<(
    Queries<'_>,
    ManyQuery<qt::TestCase, impl Convert<qt::TestCase> + '_>,
)> {
    let name_query = queries.one("@name/string()", convert_string)?;
    let description_query = queries.one("description/string()", convert_string)?;

    let test_query = queries.one("test/string()", convert_string)?;
    let by_query = queries.one("@by/string()", convert_string)?;
    let on_query = queries.one("@on/string()", convert_string)?;
    // println!("on_query: {:?}", on_query);
    let by_query2 = by_query.clone();
    let on_query2 = on_query.clone();
    let created_query = queries.one("created", move |session, item| {
        {
            {
                Ok(qt::Attribution {
                    by: by_query.execute(session, item)?,
                    on: on_query.execute(session, item)?,
                })
            }
        }
    })?;
    let change_query = queries.one("@change/string()", convert_string)?;
    let modified_query = queries.many("modified", move |session, item| {
        let attribution = qt::Attribution {
            by: by_query2.execute(session, item)?,
            on: on_query2.execute(session, item)?,
        };
        let description = change_query.execute(session, item)?;
        Ok(qt::Modification {
            attribution,
            description,
        })
    })?;

    let type_query = queries.one("@type/string()", convert_string)?;
    let value_query = queries.one("@value/string()", convert_string)?;
    let dependency_query = queries.many("dependency", move |session, item| {
        Ok(qt::Dependency {
            type_: type_query.execute(session, item)?,
            value: value_query.execute(session, item)?,
        })
    })?;

    let code_query = queries.one("@code/string()", convert_string)?;
    let error_query = queries.option("error", move |session, item| {
        Ok(qt::TestCaseResult::Error(
            code_query.execute(session, item)?,
        ))
    })?;
    let assert_true_query =
        queries.option("assert-true", |_, _| Ok(qt::TestCaseResult::AssertTrue))?;

    // let any_of_query_ref = Rc::new(OneQueryRef::new());

    // let any_of_query_ref2 = any_of_query_ref.clone();

    let f = move |session: &Session, item: &Item| {
        // let any_of = any_of_query_ref2.clone().execute(dynamic_context, item)?;
        let error = error_query.execute(session, item)?;
        if let Some(error) = error {
            return Ok(error);
        }
        let assert_true = assert_true_query.execute(session, item)?;
        if let Some(assert_true) = assert_true {
            return Ok(assert_true);
        };
        Ok(qt::TestCaseResult::AssertFalse)
    };

    // any_of_query_ref.fulfill(&static_context, "any-of", f);

    let result_query = queries.one("result", f)?;
    // unreachable!("unknown result type")
    // let all_of_query = OptionQuery::new(static_context, "all-of", |dynamic_context, item| {});
    // })?;

    let test_query = queries.many("/test-set/test-case", move |session, item| {
        Ok(qt::TestCase {
            name: name_query.execute(session, item)?,
            description: description_query.execute(session, item)?,
            created: created_query.execute(session, item)?,
            modified: modified_query.execute(session, item)?,
            environments: Vec::new(),
            dependencies: dependency_query.execute(session, item)?,
            modules: Vec::new(),
            test: test_query.execute(session, item)?,
            result: result_query.execute(session, item)?,
        })
    })?;

    Ok((queries, test_query))
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

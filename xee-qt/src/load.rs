use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use xee_xpath::Recurse;
use xee_xpath::Session;
use xee_xpath::{
    Convert, ConvertError, DynamicContext, Item, ManyQuery, Namespaces, Node, Queries,
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

    let queries = Queries::new(&static_context);

    let (queries, query) = test_cases_query(xot, queries)?;

    let dynamic_context = DynamicContext::new(xot, &static_context);
    let session = queries.session(&dynamic_context);
    // the query has a lifetime for the dynamic context, and a lifetime
    // for the static context
    let r = query.execute(&session, &Item::Node(root))?;
    Ok(r)
}

fn convert_string(_: &Session, item: &Item) -> Result<String, ConvertError> {
    Ok(item.as_atomic()?.as_string()?)
}

fn test_cases_query<'a>(
    _xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(
    Queries<'a>,
    ManyQuery<qt::TestCase, impl Convert<qt::TestCase> + 'a>,
)> {
    let name_query = queries.one("@name/string()", convert_string)?;
    let description_query = queries.one("description/string()", convert_string)?;

    let test_query = queries.one("test/string()", convert_string)?;
    let by_query = queries.one("@by/string()", convert_string)?;
    let on_query = queries.one("@on/string()", convert_string)?;
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
    let satisfied_query = queries.option("@satisfied/string()", convert_string)?;
    let dependency_query = queries.many("dependency", move |session, item| {
        let satisfied = satisfied_query.execute(session, item)?;
        let satisfied = if let Some(satisfied) = satisfied {
            if satisfied == "true" {
                true
            } else if satisfied == "false" {
                false
            } else {
                panic!("Unexpected satisfied value: {:?}", satisfied);
            }
        } else {
            true
        };
        Ok(qt::Dependency {
            type_: type_query.execute(session, item)?,
            value: value_query.execute(session, item)?,
            satisfied,
        })
    })?;

    let code_query = queries.one("@code/string()", convert_string)?;
    let error_query = queries.one(".", move |session, item| {
        Ok(qt::TestCaseResult::Error(
            code_query.execute(session, item)?,
        ))
    })?;
    let assert_count_query = queries.one("string()", |_, item| {
        let count = item.as_atomic()?.as_string()?;
        // XXX unwrap is a hack
        let count = count.parse::<usize>().unwrap();
        Ok(qt::TestCaseResult::AssertCount(count))
    })?;

    let assert_xml_query = queries.one("string()", |_, item| {
        let xml = item.as_atomic()?.as_string()?;
        Ok(qt::TestCaseResult::AssertXml(xml))
    })?;

    let assert_eq_query = queries.one("string()", |_, item| {
        let eq = item.as_atomic()?.as_string()?;
        Ok(qt::TestCaseResult::AssertEq(eq))
    })?;

    let assert_string_value_query = queries.one("string()", |_, item| {
        let string_value = item.as_atomic()?.as_string()?;
        Ok(qt::TestCaseResult::AssertStringValue(string_value))
    })?;

    let any_of_recurse = queries.many_recurse("*")?;

    // we use a local-name query here as it's the easiest way support this:
    // there is a single entry in the "result" element, but this may be
    // "any-of" and this contains a list of entries Using a relative path with
    // `query.option()` to detect entries (like "error", "assert-true", etc)
    // doesn't work for "any-of", as it contains a list of entries.
    let local_name_query = queries.one("local-name()", convert_string)?;
    let result_query = queries.one("result/*", move |session: &Session, item: &Item| {
        let f = |session: &Session, item: &Item, recurse: &Recurse<qt::TestCaseResult>| {
            let local_name = local_name_query.execute(session, item)?;
            let r = if local_name == "any-of" {
                let contents = any_of_recurse.execute(session, item, recurse)?;
                qt::TestCaseResult::AnyOf(contents)
            } else if local_name == "error" {
                error_query.execute(session, item)?
            } else if local_name == "assert-true" {
                qt::TestCaseResult::AssertTrue
            } else if local_name == "assert-false" {
                qt::TestCaseResult::AssertFalse
            } else if local_name == "assert-count" {
                assert_count_query.execute(session, item)?
            } else if local_name == "assert-xml" {
                assert_xml_query.execute(session, item)?
            } else if local_name == "assert-eq" {
                assert_eq_query.execute(session, item)?
            } else if local_name == "assert-string-value" {
                assert_string_value_query.execute(session, item)?
            } else {
                // qt::TestCaseResult::AssertFalse
                panic!("unknown local name: {}", local_name);
            };
            Ok(r)
        };
        let recurse = Recurse::new(&f);
        Ok(recurse.execute(session, item)?)
    })?;

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

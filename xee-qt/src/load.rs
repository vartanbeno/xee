use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use xee_xpath::Recurse;
use xee_xpath::Session;
use xee_xpath::{
    ConvertError, DynamicContext, Item, Namespaces, Node, Queries, Query, StaticContext,
};
use xot::Xot;

use crate::qt;

const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

impl qt::TestSet {
    pub(crate) fn load_from_file(xot: &mut Xot, path: &Path) -> Result<Self> {
        let xml_file = File::open(path)
            .into_diagnostic()
            .wrap_err("Cannot open XML file")?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader
            .read_to_string(&mut xml)
            .into_diagnostic()
            .wrap_err("Cannot read XML file")?;
        Self::load_from_xml(xot, &xml)
    }

    pub(crate) fn load_from_xml(xot: &mut Xot, xml: &str) -> Result<Self> {
        let xot_root = xot
            .parse(xml)
            .into_diagnostic()
            .wrap_err("Cannot parse XML")?;
        let root = Node::Xot(xot_root);
        let namespaces = Namespaces::with_default_element_namespace(NS);

        let static_context = StaticContext::new(&namespaces);
        let r = {
            let queries = Queries::new(&static_context);

            let (queries, query) = test_set_query(xot, queries)?;

            let dynamic_context = DynamicContext::new(xot, &static_context);
            let session = queries.session(&dynamic_context);
            // the query has a lifetime for the dynamic context, and a lifetime
            // for the static context
            query.execute(&session, &Item::Node(root))?
        };
        xot.remove(xot_root).unwrap();
        Ok(r)
    }
}

impl qt::Catalog {
    // XXX some duplication here with qt::TestSet
    pub(crate) fn load_from_file(xot: &mut Xot, path: &Path) -> Result<Self> {
        let xml_file = File::open(path)
            .into_diagnostic()
            .wrap_err("Cannot open XML file")?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader
            .read_to_string(&mut xml)
            .into_diagnostic()
            .wrap_err("Cannot read XML file")?;
        Self::load_from_xml(xot, &xml)
    }

    pub(crate) fn load_from_xml(xot: &mut Xot, xml: &str) -> Result<Self> {
        let xot_root = xot
            .parse(xml)
            .into_diagnostic()
            .wrap_err("Cannot parse XML")?;
        let root = Node::Xot(xot_root);
        let namespaces = Namespaces::with_default_element_namespace(NS);

        let static_context = StaticContext::new(&namespaces);

        let r = {
            let queries = Queries::new(&static_context);

            let (queries, query) = catalog_query(xot, queries)?;

            let dynamic_context = DynamicContext::new(xot, &static_context);
            let session = queries.session(&dynamic_context);
            query.execute(&session, &Item::Node(root))?
        };
        xot.remove(xot_root).unwrap();
        Ok(r)
    }
}

fn test_set_query<'a>(
    xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<qt::TestSet> + 'a)> {
    let name_query = queries.one("@name/string()", convert_string)?;
    let descriptions_query = queries.many("description/string()", convert_string)?;
    let (queries, shared_environments_query) = shared_environments_query(xot, queries)?;
    let (mut queries, test_cases_query) = test_cases_query(xot, queries)?;
    let test_set_query = queries.one("/test-set", move |session, item| {
        let name = name_query.execute(session, item)?;
        let descriptions = descriptions_query.execute(session, item)?;
        let shared_environments = shared_environments_query.execute(session, item)?;
        let test_cases = test_cases_query.execute(session, item)?;
        Ok(qt::TestSet {
            name,
            descriptions,
            dependencies: Vec::new(),
            shared_environments,
            test_cases,
        })
    })?;
    Ok((queries, test_set_query))
}

fn convert_string(_: &Session, item: &Item) -> Result<String, ConvertError> {
    Ok(item.to_atomic()?.to_string()?)
}

fn metadata_query<'a>(
    _xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<qt::Metadata> + 'a)> {
    let description_query = queries.option("description/string()", convert_string)?;
    let by_query = queries.one("@by/string()", convert_string)?;
    let on_query = queries.one("@on/string()", convert_string)?;
    let by_query2 = by_query.clone();
    let on_query2 = on_query.clone();
    let created_query = queries.option("created", move |session, item| {
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

    let metadata_query = queries.one(".", move |session, item| {
        let description = description_query.execute(session, item)?;
        let created = created_query.execute(session, item)?;
        let modified = modified_query.execute(session, item)?;
        Ok(qt::Metadata {
            description,
            created,
            modified,
        })
    })?;

    Ok((queries, metadata_query))
}

fn test_cases_query<'a>(
    xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<Vec<qt::TestCase>> + 'a)> {
    let name_query = queries.one("@name/string()", convert_string)?;
    let (mut queries, metadata_query) = metadata_query(xot, queries)?;
    let test_query = queries.one("test/string()", convert_string)?;

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
            spec: qt::DependencySpec {
                type_: type_query.execute(session, item)?,
                value: value_query.execute(session, item)?,
            },
            satisfied,
        })
    })?;
    let ref_query = queries.option("@ref/string()", convert_string)?;
    let (mut queries, environment_query) = environment_spec_query(xot, queries)?;
    let local_environment_query = queries.many("environment", move |session, item| {
        let ref_ = ref_query.execute(session, item)?;
        if let Some(ref_) = ref_ {
            Ok(qt::TestCaseEnvironment::Ref(qt::EnvironmentRef { ref_ }))
        } else {
            Ok(qt::TestCaseEnvironment::Local(Box::new(
                environment_query.execute(session, item)?,
            )))
        }
    })?;

    let code_query = queries.one("@code/string()", convert_string)?;
    let error_query = queries.one(".", move |session, item| {
        Ok(qt::TestCaseResult::Error(
            code_query.execute(session, item)?,
        ))
    })?;
    let assert_count_query = queries.one("string()", |_, item| {
        let count = item.to_atomic()?.to_string()?;
        // XXX unwrap is a hack
        let count = count.parse::<usize>().unwrap();
        Ok(qt::TestCaseResult::AssertCount(count))
    })?;

    let assert_xml_query = queries.one("string()", |_, item| {
        let xml = item.to_atomic()?.to_string()?;
        Ok(qt::TestCaseResult::AssertXml(xml))
    })?;

    let assert_eq_query = queries.one("string()", |_, item| {
        let eq = item.to_atomic()?.to_string()?;
        Ok(qt::TestCaseResult::AssertEq(qt::XPathExpr(eq)))
    })?;

    let assert_string_value_query = queries.one("string()", |_, item| {
        let string_value = item.to_atomic()?.to_string()?;
        Ok(qt::TestCaseResult::AssertStringValue(string_value))
    })?;

    let any_all_recurse = queries.many_recurse("*")?;

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
                let contents = any_all_recurse.execute(session, item, recurse)?;
                qt::TestCaseResult::AnyOf(contents)
            } else if local_name == "all-of" {
                let contents = any_all_recurse.execute(session, item, recurse)?;
                qt::TestCaseResult::AllOf(contents)
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

    let test_query = queries.many("test-case", move |session, item| {
        Ok(qt::TestCase {
            name: name_query.execute(session, item)?,
            metadata: metadata_query.execute(session, item)?,
            environments: local_environment_query.execute(session, item)?,
            dependencies: dependency_query.execute(session, item)?,
            modules: Vec::new(),
            test: test_query.execute(session, item)?,
            result: result_query.execute(session, item)?,
        })
    })?;

    Ok((queries, test_query))
}

fn environment_spec_query<'a>(
    xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<qt::EnvironmentSpec> + 'a)> {
    let file_query = queries.one("@file/string()", convert_string)?;
    let role_query = queries.option("@role/string()", convert_string)?;
    let uri_query = queries.option("@uri/string()", convert_string)?;
    let (mut queries, metadata_query) = metadata_query(xot, queries)?;

    let sources_query = queries.many("source", move |session, item| {
        let file = PathBuf::from(file_query.execute(session, item)?);
        let role = role_query.execute(session, item)?;
        let uri = uri_query.execute(session, item)?;
        let metadata = metadata_query.execute(session, item)?;
        // we can return multiple sources if both role and uri are set
        // we flatten it later
        let mut sources = Vec::new();
        if let Some(role) = role {
            if role == "." {
                sources.push(qt::Source {
                    metadata: metadata.clone(),
                    role: qt::SourceRole::Context,
                    file: file.clone(),
                })
            } else {
                sources.push(qt::Source {
                    metadata: metadata.clone(),
                    role: qt::SourceRole::Var(role),
                    file: file.clone(),
                });
            }
        };

        if let Some(uri) = uri {
            sources.push(qt::Source {
                metadata,
                role: qt::SourceRole::Doc(uri),
                file,
            });
        }

        Ok(sources)
    })?;

    let environment_query = queries.one(".", move |session, item| {
        let sources = sources_query.execute(session, item)?;
        // we need to flatten sources
        let sources = sources.into_iter().flatten().collect::<Vec<qt::Source>>();
        let environment_spec = qt::EnvironmentSpec {
            sources,
            ..Default::default()
        };
        Ok(environment_spec)
    })?;

    Ok((queries, environment_query))
}

fn shared_environments_query<'a>(
    xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<qt::SharedEnvironments> + 'a)> {
    let name_query = queries.one("@name/string()", convert_string)?;
    let (mut queries, environment_spec_query) = environment_spec_query(xot, queries)?;
    let environments_query = queries.many("environment", move |session, item| {
        let name = name_query.execute(session, item)?;
        let environment_spec = environment_spec_query.execute(session, item)?;
        Ok((name, environment_spec))
    })?;
    let shared_environments_query = queries.one(".", move |session, item| {
        let environments = environments_query.execute(session, item)?;
        Ok(qt::SharedEnvironments::new(
            environments.into_iter().collect(),
        ))
    })?;
    Ok((queries, shared_environments_query))
}

fn catalog_query<'a>(
    xot: &'a Xot,
    mut queries: Queries<'a>,
) -> Result<(Queries<'a>, impl Query<qt::Catalog> + 'a)> {
    let test_suite_query = queries.one("@test-suite/string()", convert_string)?;
    let version_query = queries.one("@version/string()", convert_string)?;

    let (mut queries, shared_environments_query) = shared_environments_query(xot, queries)?;

    let test_set_name_query = queries.one("@name/string()", convert_string)?;
    let test_set_file_query = queries.one("@file/string()", convert_string)?;
    let test_set_query = queries.many("test-set", move |session, item| {
        let name = test_set_name_query.execute(session, item)?;
        let file = PathBuf::from(test_set_file_query.execute(session, item)?);
        Ok(qt::TestSetRef { name, file })
    })?;
    let catalog_query = queries.one("catalog", move |session, item| {
        let test_suite = test_suite_query.execute(session, item)?;
        let version = version_query.execute(session, item)?;
        let shared_environments = shared_environments_query.execute(session, item)?;
        let test_sets = test_set_query.execute(session, item)?;
        let file_paths = test_sets.iter().map(|t| t.file.clone()).collect();
        Ok(qt::Catalog {
            test_suite,
            version,
            shared_environments,
            test_sets,
            file_paths,
        })
    })?;
    Ok((queries, catalog_query))
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_debug_snapshot;

    const ROOT_FIXTURE: &str = include_str!("fixtures/root.xml");
    const CATALOG_FIXTURE: &str = include_str!("fixtures/catalog.xml");

    #[test]
    fn test_load() {
        let mut xot = Xot::new();
        assert_debug_snapshot!(qt::TestSet::load_from_xml(&mut xot, ROOT_FIXTURE).unwrap());
    }

    #[test]
    fn test_load_catalog() {
        let mut xot = Xot::new();
        assert_debug_snapshot!(qt::Catalog::load_from_xml(&mut xot, CATALOG_FIXTURE).unwrap());
    }
}

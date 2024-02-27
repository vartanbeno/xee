use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use xee_xpath::{
    context::{DynamicContext, StaticContext, Variables},
    parse, sequence,
    xml::Documents,
    Name, Queries, Query,
};
use xot::Xot;

use crate::{error::Result, load::convert_string, metadata::Metadata};

use super::{
    collation::Collation,
    collection::Collection,
    resource::Resource,
    source::{Source, SourceRole},
};

// the abstract environment. Can be an XPath or XSLT environment.
pub(crate) trait Environment {
    // create an empty environment
    fn empty() -> Self;

    // get the underlying environment spec
    fn environment_spec(&self) -> &EnvironmentSpec;
}

// In a test case we can include an environment directly, or refer to an environment
#[derive(Debug)]
pub(crate) enum TestCaseEnvironment<E: Environment> {
    Local(Box<E>),
    Ref(EnvironmentRef),
}

// a way to reference to other environments
#[derive(Debug, Clone)]
pub struct EnvironmentRef {
    pub(crate) ref_: String,
}

impl Display for EnvironmentRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ref_)
    }
}

// environment information shared by XPath and XSLT
#[derive(Debug, Default, Clone)]
pub(crate) struct EnvironmentSpec {
    pub(crate) base_dir: PathBuf,

    pub(crate) sources: Vec<Source>,
    pub(crate) params: Vec<Param>,
    // TODO
    pub(crate) collations: Vec<Collation>,
    // TODO: needs to wait until the interpreter has a resource abstraction
    pub(crate) resources: Vec<Resource>,
    // TODO: needs to wait until the interpreter has a collection abstraction
    pub(crate) collections: Vec<Collection>,
    // not supported as Xee doesn't support XML schema
    pub(crate) schemas: Vec<Schema>,
    // Not in use at all?
    // pub(crate) function_libraries: Vec<FunctionLibrary>,
}

// Not supported yet: schema support not implemented in Xee
#[derive(Debug, Clone)]
pub(crate) struct Schema {}

#[derive(Debug, Clone)]
pub(crate) struct Param {
    pub(crate) name: Name,
    pub(crate) select: Option<String>,
    // TODO: not supported yet
    pub(crate) as_: Option<String>,
    // TODO: not supported yet
    pub(crate) static_: bool,
    // XQuery related, not supported
    pub(crate) declared: bool,
    // Doesn't appear to be in use, even though it's in the schema
    pub(crate) source: Option<String>,
}

impl EnvironmentSpec {
    pub(crate) fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn context_item(
        &self,
        xot: &mut Xot,
        documents: &mut Documents,
    ) -> Result<Option<sequence::Item>> {
        for source in &self.sources {
            if let SourceRole::Context = source.role {
                let node = source.node(xot, &self.base_dir, documents)?;
                return Ok(Some(sequence::Item::from(node)));
            }
        }
        Ok(None)
    }

    pub(crate) fn variables(&self, xot: &mut Xot, documents: &mut Documents) -> Result<Variables> {
        let mut variables = Variables::new();
        for source in &self.sources {
            if let SourceRole::Var(name) = &source.role {
                let name = &name[1..]; // without $
                let node = source.node(xot, &self.base_dir, documents)?;
                variables.insert(Name::name(name), sequence::Item::from(node).into());
            }
        }
        for param in &self.params {
            let static_context = StaticContext::default();
            let select = (param.select.as_ref()).expect("param: missing select not supported");
            let program = parse(&static_context, select);
            if program.is_err() {
                println!("param: select xpath parse failed: {}", select);
                continue;
            }
            let program = program.unwrap();
            let dynamic_context = DynamicContext::empty(&static_context);
            let runnable = program.runnable(&dynamic_context);
            let result = runnable.many(None, xot).map_err(|e| e.error)?;
            variables.insert(param.name.clone(), result);
        }
        Ok(variables)
    }

    pub(crate) fn query<'a>(
        path: &'a Path,
        queries: Queries<'a>,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)> {
        let (mut queries, sources_query) = Source::query(queries)?;

        let name_query = queries.one("@name/string()", convert_string)?;
        let select_query = queries.option("@select/string()", convert_string)?;
        let as_query = queries.option("@as/string()", convert_string)?;
        let source_query = queries.option("@source/string()", convert_string)?;
        let declared_query = queries.option("@declared/string()", convert_string)?;

        let params_query = queries.many("param", move |session, item| {
            let name = name_query.execute(session, item)?;
            let select = select_query.execute(session, item)?;
            let as_ = as_query.execute(session, item)?;
            let source = source_query.execute(session, item)?;
            let declared = declared_query.execute(session, item)?;

            let declared = declared.map(|declared| declared == "true").unwrap_or(false);

            // TODO: do not handle prefixes yet
            let name = Name::name(&name);

            Ok(Param {
                name,
                select,
                as_,
                source,
                declared,
                // TODO
                static_: false,
            })
        })?;

        // the environment base_dir is the same as the catalog/test set path,
        // but without the file name
        let path = path.parent().unwrap();
        let environment_query = queries.one(".", move |session, item| {
            let sources = sources_query.execute(session, item)?;
            // we need to flatten sources
            let sources = sources.into_iter().flatten().collect::<Vec<Source>>();
            let params = params_query.execute(session, item)?;

            let environment_spec = EnvironmentSpec {
                base_dir: path.to_path_buf(),
                sources,
                params,
                ..Default::default()
            };

            Ok(environment_spec)
        })?;

        Ok((queries, environment_query))
    }
}

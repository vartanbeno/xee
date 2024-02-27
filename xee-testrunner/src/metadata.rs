use xee_xpath::{Queries, Query};
use xot::Xot;

use crate::error::Result;
use crate::load::convert_string;

#[derive(Debug, Clone)]
pub(crate) struct Metadata {
    pub(crate) description: Option<String>,
    pub(crate) created: Option<Attribution>,
    pub(crate) modified: Vec<Modification>,
}

#[derive(Debug, Clone)]
pub(crate) struct Modification {
    pub(crate) attribution: Attribution,
    pub(crate) description: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Attribution {
    pub(crate) by: String,
    pub(crate) on: String, // should be a date
}

impl Metadata {
    pub(crate) fn query<'a>(
        _xot: &Xot,
        mut queries: Queries<'a>,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)> {
        let description_query = queries.option("description/string()", convert_string)?;
        let by_query = queries.one("@by/string()", convert_string)?;
        let on_query = queries.one("@on/string()", convert_string)?;
        let by_query2 = by_query.clone();
        let on_query2 = on_query.clone();
        let created_query = queries.option("created", move |session, item| {
            {
                {
                    Ok(Attribution {
                        by: by_query.execute(session, item)?,
                        on: on_query.execute(session, item)?,
                    })
                }
            }
        })?;

        let change_query = queries.option("@change/string()", convert_string)?;
        let modified_query = queries.many("modified", move |session, item| {
            let attribution = Attribution {
                by: by_query2.execute(session, item)?,
                on: on_query2.execute(session, item)?,
            };
            let description = change_query.execute(session, item)?;
            Ok(Modification {
                attribution,
                description: description.unwrap_or("".to_string()),
            })
        })?;

        let metadata_query = queries.one(".", move |session, item| {
            let description = description_query.execute(session, item)?;
            let created = created_query.execute(session, item)?;
            let modified = modified_query.execute(session, item)?;
            Ok(Metadata {
                description,
                created,
                modified,
            })
        })?;

        Ok((queries, metadata_query))
    }
}

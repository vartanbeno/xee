use ibig::{ibig, IBig};
use xee_interpreter::sequence::Sequence;
use xee_xpath::{
    error, item, query::RecurseQuery, Documents, Queries, Query, Recurse, Session, Uri,
};

#[test]
fn test_duplicate_document_uri() -> error::Result<()> {
    let mut documents = Documents::new();
    let _doc1 = documents
        .add_string(
            &Uri::new("doc1"),
            r#"<doc><result><any-of><value>A</value></any-of></result></doc>"#,
        )
        .unwrap();
    // try to load doc with the same URI
    let doc2_err = documents
        .add_string(
            &Uri::new("doc1"),
            r#"<doc><result><value>A</value></result></doc>"#,
        )
        .unwrap_err();
    assert_eq!(doc2_err.to_string(), "Duplicate URI: doc1");
    Ok(())
}

#[test]
fn test_simple_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .add_string(&Uri::new("http://example.com"), "<root>foo</root>")
        .unwrap();

    let mut queries = Queries::default();
    let q = queries.one("/root/string()", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, doc)?;
    assert_eq!(r, "foo");
    Ok(())
}

#[test]
fn test_sequence_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .add_string(&Uri::new("http://example.com"), "<root>foo</root>")
        .unwrap();

    let mut queries = Queries::default();
    let q = queries.sequence("/root/string()")?;

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, doc)?;
    let sequence: Sequence = "foo".into();
    assert_eq!(r, sequence);
    Ok(())
}

#[test]
fn test_option_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc_with_value = documents
        .add_string(
            &Uri::new("http://example.com/with_value"),
            "<root><value>Foo</value></root>",
        )
        .unwrap();
    let doc_without_value = documents
        .add_string(
            &Uri::new("http://example.com/without_value"),
            "<root></root>",
        )
        .unwrap();

    let mut queries = Queries::default();
    let q = queries.option("/root/value/string()", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, doc_with_value)?;
    assert_eq!(r, Some("Foo".to_string()));
    let r = q.execute(&mut session, doc_without_value)?;
    assert_eq!(r, None);
    Ok(())
}

#[test]
fn test_nested_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .add_string(
            &Uri::new("http://example.com"),
            "<root><a>1</a><a>2</a></root>",
        )
        .unwrap();

    let mut queries = Queries::default();
    let f_query = queries.one("./number()", |_, item| Ok(item.try_into_value::<f64>()?))?;
    let q = queries.many("/root/a", |session, item| {
        Ok(f_query.execute(session, item)?)
    })?;

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, doc)?;
    assert_eq!(r, vec![1.0, 2.0]);
    Ok(())
}

#[test]
fn test_wrong_queries() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .add_string(&Uri::new("http://example.com"), "<root>foo</root>")
        .unwrap();

    let queries = Queries::default();

    let mut second_queries = Queries::default();
    let second_q = second_queries.one("/root/string()", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;
    let mut session = queries.session(documents);
    let r = second_q.execute(&mut session, doc).unwrap_err();
    assert_eq!(r, error::ErrorValue::UsedQueryWithWrongQueries.into());
    Ok(())
}

#[test]
fn test_option_query_recurse() -> error::Result<()> {
    let mut queries = Queries::default();

    #[derive(Debug, PartialEq, Eq)]
    enum Expr {
        AnyOf(Box<Expr>),
        Value(String),
        Empty,
    }

    // if we find the "any-of" element, we want to use a recursive
    // call to the query we pass it
    let any_of_recurse = queries.option_recurse("any-of")?;
    // the "value" element is simply a string
    let value_query = queries.option("value/string()", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;

    // a result is either a "value" or an "any-of" element
    let result_query = queries.one("/doc/result", |session, item| {
        let f = |session: &mut Session, item: &item::Item, recurse: &Recurse<_>| {
            // we either call the any of query, which recursively
            // calls this function
            if let Some(any_of) = any_of_recurse.execute(session, item, recurse)? {
                return Ok(Expr::AnyOf(Box::new(any_of)));
            }
            // or use the value query
            if let Some(value) = value_query.execute(session, item)? {
                return Ok(Expr::Value(value));
            }
            Ok(Expr::Empty)
        };
        // we want to recursively call this function
        let recurse = Recurse::new(&f);
        recurse.execute(session, item)
    })?;

    let mut documents = Documents::new();
    let doc1 = documents
        .add_string(
            &Uri::new("doc1"),
            r#"<doc><result><any-of><value>A</value></any-of></result></doc>"#,
        )
        .unwrap();
    let doc2 = documents
        .add_string(
            &Uri::new("doc2"),
            r#"<doc><result><value>A</value></result></doc>"#,
        )
        .unwrap();

    let mut session = queries.session(documents);
    let r = result_query.execute(&mut session, doc1)?;
    assert_eq!(r, Expr::AnyOf(Box::new(Expr::Value("A".to_string()))));

    let r = result_query.execute(&mut session, doc2)?;
    assert_eq!(r, Expr::Value("A".to_string()));
    Ok(())
}

#[test]
fn test_many_query_recurse() -> error::Result<()> {
    let mut queries = Queries::default();

    #[derive(Debug, PartialEq, Eq)]
    enum Expr {
        AnyOf(Vec<Expr>),
        Value(String),
        Empty,
    }

    // if we find any "any-of" element, we want to use a recursive
    // call to the query we pass it
    let any_of_recurse = queries.many_recurse("any-of")?;
    // the "value" element is simply a string
    let value_query = queries.option("value/string()", |_, item| {
        Ok(item.try_into_value::<String>()?)
    })?;

    // a result is either a "value" or an "any-of" element
    let result_query = queries.one("/doc/result", |session, item| {
        let f = |session: &mut Session, item: &item::Item, recurse: &Recurse<_>| {
            // we either call the any of query, which recursively
            // calls this function
            let elements = any_of_recurse.execute(session, item, recurse)?;
            if !elements.is_empty() {
                return Ok(Expr::AnyOf(elements));
            }
            // or use the value query
            if let Some(value) = value_query.execute(session, item)? {
                return Ok(Expr::Value(value));
            }
            Ok(Expr::Empty)
        };
        // we want to recursively call this function
        let recurse = Recurse::new(&f);
        recurse.execute(session, item)
    })?;

    let mut documents = Documents::new();
    let doc1 = documents
        .add_string(
            &Uri::new("doc1"),
            r#"<doc><result><any-of><value>A</value></any-of><any-of><value>B</value></any-of></result></doc>"#,
        )
        .unwrap();
    let doc2 = documents
        .add_string(
            &Uri::new("doc2"),
            r#"<doc><result><value>A</value></result></doc>"#,
        )
        .unwrap();

    let mut session = queries.session(documents);
    let r = result_query.execute(&mut session, doc1)?;
    assert_eq!(
        r,
        Expr::AnyOf(vec![
            Expr::Value("A".to_string()),
            Expr::Value("B".to_string())
        ])
    );

    let r = result_query.execute(&mut session, doc2)?;
    assert_eq!(r, Expr::Value("A".to_string()));
    Ok(())
}

#[test]
fn test_map_query() -> error::Result<()> {
    let mut queries = Queries::default();
    let q = queries
        .one("1 + 2", |_, item| {
            let v: IBig = item.to_atomic()?.try_into()?;
            Ok(v)
        })?
        .map(|v, _, _| Ok(v + ibig!(1)));

    let documents = Documents::new();

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, &1i64.into())?;
    assert_eq!(r, ibig!(4));
    Ok(())
}

#[test]
fn test_map_query_clone() -> error::Result<()> {
    let mut queries = Queries::default();
    let q = queries
        .one("1 + 2", |_, item| {
            let v: IBig = item.to_atomic()?.try_into()?;
            Ok(v)
        })?
        .map(|v, _, _| Ok(v + ibig!(1)));
    let q = q.clone();
    let documents = Documents::new();

    let mut session = queries.session(documents);
    let r = q.execute(&mut session, &1i64.into())?;
    assert_eq!(r, ibig!(4));
    Ok(())
}

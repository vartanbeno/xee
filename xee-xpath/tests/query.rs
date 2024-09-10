use xee_xpath::{error, Documents, Queries};

#[test]
fn test_simple_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .load_string("http://example.com", "<root>foo</root>")
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
fn test_nested_query() -> error::Result<()> {
    let mut documents = Documents::new();
    let doc = documents
        .load_string("http://example.com", "<root><a>1</a><a>2</a></root>")
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

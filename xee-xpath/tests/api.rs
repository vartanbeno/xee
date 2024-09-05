use xee_xpath::{Documents, Engine, Item, Result, SpannedResult, XPaths};

#[test]
fn test_basic() -> SpannedResult<()> {
    let mut xpaths = XPaths::default();
    let e = xpaths.compile("//root/string()")?;

    let mut documents = Documents::new();
    let doc = documents
        .load_string("http://example.com", "<root>text</root>")
        .unwrap();

    let mut engine = Engine::new(&xpaths, documents);
    let result = engine.evaluate(e, doc)?;
    // we can collect this and we either have an error or we get an item
    let result = result.into_iter().collect::<Result<Vec<Item>>>().unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].to_atomic().unwrap().to_string().unwrap(), "text");
    Ok(())
}

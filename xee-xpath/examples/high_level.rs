use xee_xpath::high_level::{Documents, Engine, XPath};

fn main() {
    let mut xpath = XPath::default();
    let e = xpath.compile("//root/text()").unwrap();

    let mut documents = Documents::new();
    let doc = documents
        .load_string("http://example.com", "<root>text</root>")
        .unwrap();

    let mut engine = Engine::new(&xpath);
    let result = engine.evaluate(e, documents, doc).unwrap();

    let mut documents2 = Documents::new();
    let doc2 = documents2
        .load_string("http://example.com", "<root>text2</root>")
        .unwrap();

    let mut engine = Engine::new(&xpath);
    let result2 = engine.evaluate(e, documents2, doc2).unwrap();
}

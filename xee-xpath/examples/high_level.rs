use xee_xpath::high_level::{Documents, Engine, XPath};

fn main() {
    let mut documents = Documents::new();
    let doc = documents
        .load_string("http://example.com", "<root>text</root>")
        .unwrap();

    let mut xpath = XPath::default();
    let e = xpath.compile("//root/text()").unwrap();

    let mut engine = Engine::new(xpath, documents);
    let result = engine.evaluate(e, doc).unwrap();
}

// use xee_xpath::{Documents, Engine, XPaths};

// fn main() {
//     let mut xpaths = XPaths::default();
//     let e = xpaths.compile("//root/text()").unwrap();

//     let mut documents = Documents::new();
//     let doc = documents
//         .load_string("http://example.com", "<root>text</root>")
//         .unwrap();

//     let mut engine = Engine::new(&xpaths, documents);
//     let result = engine.evaluate(e, doc).unwrap();

//     let mut documents2 = Documents::new();
//     let doc2 = documents2
//         .load_string("http://example.com", "<root>text2</root>")
//         .unwrap();

//     let mut engine = Engine::new(&xpaths, documents2);
//     let result2 = engine.evaluate(e, doc2).unwrap();
// }

fn main() {}

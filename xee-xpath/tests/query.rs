// use xee_xpath::{Documents, Engine, Queries, XPaths};

// fn main() {
//     let mut queries = Queries::default();
//     let value_query = queries
//         .one("./string()", |_, item| item.to_string().unwrap())
//         .unwrap();

//     let values_query = queries
//         .many("/root/value", |session, item| {
//             value_query.execute(session, item)
//         })
//         .unwrap();

//     // let converters = Converters::default();

//     // let value_converter = converters.one(value_query, |item| item.to_string().unwrap());
//     // let values_converter = converters.many(values_query, value_converter);

//     let mut documents = Documents::new();
//     let doc = documents
//         .load_string(
//             "http://example.com",
//             "<root><value>A</value><value>B</value></root>",
//         )
//         .unwrap();

//     let mut session = Session::new(&queries, documents);
//     // a document can be used as it is nodable to the root node?
//     let results = session.execute(values_query, doc).unwrap();
// }

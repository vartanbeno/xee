// use ahash::{HashMap, HashMapExt};
// use xot::{NameId, NamespaceId, Node, Xot};

// use crate::qt;

// const NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

// struct TestCaseNames {
//     test_case: NameId,
//     name: NameId,
//     description: NameId,
//     created: NameId,
//     created_by: NameId,
//     created_on: NameId,
//     environment: NameId,
//     test: NameId,
//     result: NameId,
// }

// impl TestCaseNames {
//     fn new(xot: &mut Xot, namespace_id: NamespaceId) -> Self {
//         TestCaseNames {
//             test_case: xot.add_name_ns("test-case", namespace_id),
//             name: xot.add_name_ns("name", namespace_id),
//             description: xot.add_name_ns("description", namespace_id),
//             created: xot.add_name_ns("created", namespace_id),
//             created_by: xot.add_name_ns("by", namespace_id),
//             created_on: xot.add_name_ns("on", namespace_id),
//             environment: xot.add_name_ns("environment", namespace_id),
//             test: xot.add_name_ns("test", namespace_id),
//             result: xot.add_name_ns("result", namespace_id),
//         }
//     }
// }

// enum DeserializerValueType {
//     Attribute,
//     TextContent,
// }

// enum Deserializer
// struct DeserializerEntry {
//     name_id: NameId,
//     type_id: DeserializerValueType,
// }

// impl DeserializerEntry {
//     fn deserialize(&self, xot: &Xot, node: Node) -> String {
//         let element = xot.element(node).unwrap();
//         match self.type_id {
//             DeserializerValueType::Attribute => {
//                 let value = element
//                     .get_attribute(self.name_id)
//                     .expect("Expected attribute but doesn't exist");
//                 value.to_string()
//             }
//             DeserializerValueType::TextContent => xot
//                 .text_content_str(node)
//                 .expect("Expected text content but doesn't exist")
//                 .to_string(),
//         }
//     }
// }

// struct Deserializer {
//     entries: HashMap<String, DeserializerEntry>,
// }

// impl Deserializer {
//     fn new() -> Self {
//         Deserializer {
//             entries: HashMap::new(),
//         }
//     }

//     fn deserialize(&self, node: Node) {}
// }

// struct Loader<'a> {
//     xot: &'a Xot,
//     namespace_id: NamespaceId,
//     test_case_names: TestCaseNames,
// }

// impl<'a> Loader<'a> {
//     fn new(xot: &'a mut Xot) -> Self {
//         let namespace_id = xot.add_namespace(NS);
//         let test_case_names = TestCaseNames::new(xot, namespace_id);
//         Self {
//             xot,
//             namespace_id,
//             test_case_names,
//         }
//     }

//     fn load_test_case(&self, node: Node) -> qt::TestCase {
//         let element = self.xot.element(node).unwrap();

//         let name = element.get_attribute(self.test_case_names.name).unwrap();
//         for child in self.xot.children(node) {
//             let element = self.xot.element(child);
//             if let Some(element) = element {
//                 if element.name() == self.test_case_names.description {
//                     let description = self.xot.text_content_str(child).unwrap();
//                 }
//             }
//         }
//         todo!();
//     }
// }

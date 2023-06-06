// use ahash::AHashMap as HashMap;

// use crate::ast::SequenceType;
// use crate::context::DynamicContext;
// use crate::data::{Atomic, ContextTryInto, Value, ValueError};

// impl SequenceType {
//     fn convert<T>(&self, value: Value, context: &DynamicContext) -> Result<T, ValueError>
//     where
//         T: std::convert::TryFrom<Atomic, Error = ValueError>,
//     {
//         let atomic: Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

// fn foo(a: i64) {}

// fn bar() {
//     let value = Value::Atomic(Atomic::Integer(1));
//     // let context = DynamicContext::new();
//     foo(sequence_type.convert(value, &context)?)
// }
// #[xpath_fn("foo($a as xs:integer, $b as xs:string) as xs:string")]
// fn foo(a: i64, s: &str) -> String {}

// wrapper should generate:
// Value -> i64
// Value -> &str
// String -> Value

// use crate::context::{DynamicContext, FN_NAMESPACE, XS_NAMESPACE};
// use crate::types::types_core::{Item, ItemType, Occurrence, SequenceType};
// use crate::value::Node;
// use crate::value::ValueError;
// use crate::{ast, Atomic, StackValue};

// #[derive(Debug)]
// struct Error {}

// type Result<T> = std::result::Result<T, Error>;

// #[derive(Debug)]
// struct FunctionDescription {
//     name: ast::Name,
//     arguments: Vec<Argument>,
//     return_type: Option<SequenceType>,
// }

// #[derive(Debug)]
// struct Argument {
//     name: ast::Name,
//     type_: SequenceType,
// }

// trait TypeConverter<T> {
//     fn to_value(
//         context: &DynamicContext,
//         stack_value: &StackValue,
//     ) -> std::result::Result<T, ValueError>;

//     fn to_stack_value(context: &DynamicContext, value: T) -> StackValue;
// }

// struct IntTypeConverter {}

// impl TypeConverter<i64> for IntTypeConverter {
//     fn to_value(
//         context: &DynamicContext,
//         stack_value: &StackValue,
//     ) -> std::result::Result<i64, ValueError> {
//         let atomic = stack_value.to_atomic(context)?;
//         atomic.to_integer()
//     }

//     fn to_stack_value(_context: &DynamicContext, value: i64) -> StackValue {
//         StackValue::Atomic(Atomic::Integer(value))
//     }
// }

// struct AtomicTypes<'a> {
//     item_types: HashMap<&'a str, ast::Name>,
// }

// impl<'a> AtomicTypes<'a> {
//     fn new() -> Self {
//         let item_types = HashMap::from([(
//             "i64",
//             ast::Name::new("integer".to_string(), Some(XS_NAMESPACE.to_string())),
//         )]);
//         Self { item_types }
//     }

//     fn get(&self, name: &str) -> Option<&ast::Name> {
//         self.item_types.get(name)
//     }
// }

// struct FnConverter {
//     atomic_types: AtomicTypes<'static>,
// }

// impl FnConverter {
//     fn new() -> Self {
//         Self {
//             atomic_types: AtomicTypes::new(),
//         }
//     }

//     fn convert_fn(&self, item_fn: syn::ItemFn) -> Result<FunctionDescription> {
//         let namespace = FN_NAMESPACE;
//         let name = ast::Name::new(item_fn.sig.ident.to_string(), Some(namespace.to_string()));
//         let arguments = self.convert_arguments(item_fn.sig.inputs)?;
//         let return_type = self.convert_return_type(&item_fn.sig.output)?;
//         Ok(FunctionDescription {
//             name,
//             arguments,
//             return_type,
//         })
//     }

//     fn convert_arguments(
//         &self,
//         inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
//     ) -> Result<Vec<Argument>> {
//         let mut arguments = Vec::new();
//         for input in inputs {
//             match input {
//                 syn::FnArg::Receiver(_) => {
//                     return Err(Error {});
//                 }
//                 syn::FnArg::Typed(pat_type) => {
//                     let name = self.convert_argument_name(&pat_type.pat)?;
//                     let type_ = self.convert_type(&pat_type.ty)?;
//                     arguments.push(Argument { name, type_ });
//                 }
//             }
//         }
//         Ok(arguments)
//     }

//     fn convert_return_type(&self, output: &syn::ReturnType) -> Result<Option<SequenceType>> {
//         match output {
//             syn::ReturnType::Default => Ok(None),
//             syn::ReturnType::Type(_, ty) => {
//                 let type_ = self.convert_type(ty)?;
//                 Ok(Some(type_))
//             }
//         }
//     }

//     fn convert_argument_name(&self, pat: &syn::Pat) -> Result<ast::Name> {
//         match pat {
//             syn::Pat::Ident(pat_ident) => Ok(ast::Name::new(pat_ident.ident.to_string(), None)),
//             _ => Err(Error {}),
//         }
//     }

//     fn convert_type(&self, ty: &syn::Type) -> Result<SequenceType> {
//         match ty {
//             syn::Type::Path(type_path) => {
//                 let ident = type_path.path.get_ident().ok_or(Error {})?;
//                 let type_ = self
//                     .atomic_types
//                     .get(ident.to_string().as_str())
//                     .ok_or(Error {})?;
//                 Ok(SequenceType::Item(Item {
//                     item_type: ItemType::AtomicOrUnionType(type_.clone()),
//                     occurrence: Occurrence::One,
//                 }))
//             }
//             _ => Err(Error {}),
//         }
//     }
// }

// pub struct Empty {}

// pub struct TextNode(Node);

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use insta::assert_debug_snapshot;
//     use nonempty::NonEmpty;

//     fn convert(s: &str) -> Result<FunctionDescription> {
//         let item_fn = syn::parse_str::<syn::ItemFn>(s).unwrap();
//         let converter = FnConverter::new();
//         converter.convert_fn(item_fn)
//     }

//     // #[test]
//     // fn test_parse() {
//     //     let item_fn = syn::parse_str::<syn::ItemFn>("fn foo(a: i64) -> i64 { a + 1 }").unwrap();
//     //     println!("{:#?}", item_fn);
//     // }

//     #[test]
//     fn test_item_one_integer() {
//         fn foo(a: i64) -> i64 {
//             a + 1
//         }
//         assert_debug_snapshot!(convert("fn foo(a: i64) -> i64 { a + 1 }"));
//     }

//     #[test]
//     fn test_item_optional_integer() {
//         fn foo(a: Option<i64>) -> Option<i64> {
//             a.map(|a| a + 1)
//         }
//     }

//     #[test]
//     fn test_item_many_integer() {
//         fn foo(a: &[i64]) -> Vec<i64> {
//             a.iter().map(|a| a + 1).collect()
//         }
//     }

//     #[test]
//     fn test_item_nonempty_integer() {
//         fn foo(a: &NonEmpty<i64>) -> Option<NonEmpty<i64>> {
//             NonEmpty::collect(a.iter().map(|a| a + 1))
//         }
//     }

//     #[test]
//     fn test_item_empty_sequence() {
//         fn foo(a: Empty) -> Empty {
//             a
//         }
//     }

//     #[test]
//     fn test_text_kind_test() {
//         fn foo(a: TextNode) -> bool {
//             true
//         }
//     }
// }

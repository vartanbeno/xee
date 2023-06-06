// use xee_xpath_ast::ast::SequenceType;

// use crate::context::DynamicContext;
// use crate::data::{Atomic, ContextTryInto, Item, Sequence, Value, ValueError};

// fn many_items(value: &Value) -> Result<Sequence, ValueError> {
//     value.try_into()
// }

// trait Convert<T> {
//     fn convert_one(&self, value: Value, context: &DynamicContext) -> Result<T, ValueError>;
//     fn convert_option(
//         &self,
//         value: Value,
//         context: &DynamicContext,
//     ) -> Result<Option<T>, ValueError> {
//         let option = value.to_option()?;
//         match option {
//             Some(item) => Ok(Some(self.convert_one(Value::from_item(item), context)?)),
//             None => Ok(None),
//         }
//     }
//     fn convert_many(&self, value: Value, context: &DynamicContext) -> Result<Vec<T>, ValueError> {
//         let sequence = value.to_many()?;
//         let mut result = Vec::with_capacity(sequence.borrow().len());
//         for item in sequence.borrow().as_slice() {
//             result.push(self.convert_one(Value::from_item(item.clone()), context)?);
//         }
//         Ok(result)
//     }
// }

// // impl Convert<Item> for SequenceType {
// //     fn convert_one(&self, value: Value, _context: &DynamicContext) -> Result<Item, ValueError> {
// //         value.to_one()
// //     }
// //     fn convert_option(
// //         &self,
// //         value: Value,
// //         _context: &DynamicContext,
// //     ) -> Result<Option<Item>, ValueError> {
// //         value.to_option()
// //     }
// //     fn convert_many(
// //         &self,
// //         value: Value,
// //         _context: &DynamicContext,
// //     ) -> Result<Vec<Item>, ValueError> {
// //         // XXX argh
// //         Ok(value.to_many()?.borrow().items.clone())
// //     }
// // }

// impl Convert<String> for SequenceType {
//     fn convert_one(&self, value: Value, context: &DynamicContext) -> Result<String, ValueError> {
//         let atomic: Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

// // impl Convert<Sequence> for SequenceType {
// //     fn convert(&self, value: Value, context: &DynamicContext) -> Result<Sequence, ValueError> {
// //         let atomic: Atomic = value.context_try_into(context)?;
// //         atomic.try_into()
// //     }
// // }

// use xot::Xot;

// use xee_interpreter::error;
// use xee_interpreter::pattern::PatternLookup;
// use xee_ir::{ir, Bindings};
// use xee_xslt_ast::ast;

// struct IrConverter {}

// impl IrConverter {
//     fn convert_transform(&mut self, transform: &ast::Transform) -> error::SpannedResult<ir::ExprS> {
//         let bindings = self.transform(transform)?;
//         Ok(bindings.expr())
//     }

//     fn transform(&mut self, transform: &ast::Transform) -> error::SpannedResult<Bindings> {
//         transform
//             .declarations
//             .iter()
//             .fold(Ok(Bindings::empty()), |bindings, declaration| {
//                 let declaration_bindings = self.declaration(declaration)?;
//                 Ok(bindings?.bind(declaration_binding))
//             })
//     }

//     fn declaration(&mut self, declaration: &ast::Declaration) -> error::SpannedResult<Bindings> {
//         use ast::Declaration::*;
//         match declaration {
//             Template(template) => {
//                 if let Some(pattern) = &template.match_ {
//                     let expr = ir::Expr::Rule(ir::Rule {
//                         pattern: pattern.pattern.clone(),
//                         function_definition:
//                     });
//                     todo!();
//                 } else {
//                     todo!();
//                 }
//             }
//             _ => {
//                 todo!("Unsupported declaration")
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    // fn convert_expr_single(s: &str) -> error::SpannedResult<ir::ExprS> {
    //     let ast = ast::ExprSingle::parse(s)?;
    //     let static_context = context::StaticContext::default();
    //     let mut converter = IrConverter::new(&static_context);
    //     converter.convert_expr_single(&ast)
    // }

    // pub(crate) fn convert_xpath(s: &str) -> error::SpannedResult<ir::ExprS> {
    //     let static_context = context::StaticContext::default();
    //     let ast = static_context.parse_xpath(s)?;
    //     let mut converter = IrConverter::new(&static_context);
    //     converter.convert_xpath(&ast)
    // }

    // #[test]
    // fn test_integer() {
    //     assert_debug_snapshot!(convert_expr_single("1"));
    // }
}

use xot::Xot;

use xee_interpreter::pattern::PatternLookup;
use xee_interpreter::{error, function, interpreter};
use xee_ir::{ir, Bindings};
use xee_xslt_ast::ast;

struct IrConverter {
    program: interpreter::Program,
}

impl IrConverter {
    fn convert_transform(
        mut self,
        transform: &ast::Transform,
    ) -> error::SpannedResult<interpreter::Program> {
        let bindings = (&mut self).transform(transform)?;
        Ok(self.program)
    }

    fn transform(&mut self, transform: &ast::Transform) -> error::SpannedResult<()> {
        for declaration in &transform.declarations {
            self.declaration(declaration)?;
        }
        Ok(())
    }

    fn declaration(&mut self, declaration: &ast::Declaration) -> error::SpannedResult<()> {
        use ast::Declaration::*;
        match declaration {
            Template(template) => {
                if let Some(pattern) = &template.match_ {
                    // self.program.declarations.pattern_lookup.add(
                    //     &pattern.pattern,
                    //     self.sequence_constructor(&template.sequence_constructor)?,
                    // );
                    Ok(())
                } else {
                    todo!();
                }
            }
            _ => {
                todo!("Unsupported declaration")
            }
        }
    }

    // fn sequence_constructor(
    //     &self,
    //     sequence_constructor: &ast::SequenceConstructor,
    // ) -> error::SpannedResult<function::InlineFunctionId> {
    //     let mut items = sequence_constructor.iter();
    //     let left = items.next().unwrap();
    //     let left_bindings = Ok(self.sequence_constructor_item(left)?);
    //     // items.fold(left, |left, right| {
    //     //     let left = self.sequence_constructor_item(left)?;
    //     //     let right = self.sequence_constructor_item(right)?;
    //     //     Ok(ir::Binary {
    //     //         left: ,
    //     //         op: ir::BinaryOperator::Comma,
    //     //         right
    //     //     )
    //     // })
    // }

    // fn sequence_constructor_item(
    //     &self,
    //     item: &ast::SequenceConstructorItem,
    // ) -> error::SpannedResult<ir::ExprS> {
    //     match item {
    //         ast::SequenceConstructorItem::ElementNode(element_node) => {
    //             let name_atom = self.element_name(&element_node.name)?;
    //             ir::Expr::Element(ir::XmlElement { name: name_atom })}
    //         }
    //         _ => todo!(),
    //     }
    // }
}

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

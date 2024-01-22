use xee_name::Name;
use xee_xpath_ast::ast::Span;
use xot::Xot;

use xee_interpreter::pattern::PatternLookup;
use xee_interpreter::{error, function, interpreter};
use xee_ir::{ir, Binding, Bindings};
use xee_xpath_ast::span::Spanned;
use xee_xslt_ast::ast;

struct IrConverter {
    program: interpreter::Program,
    counter: usize,
}

impl IrConverter {
    fn new_name(&mut self) -> ir::Name {
        let name = format!("x{}", self.counter);
        self.counter += 1;
        ir::Name::new(name)
    }

    fn new_binding(&mut self, expr: ir::Expr, span: Span) -> Binding {
        let name = self.new_name();
        Binding::new(name, expr, span)
    }

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
                    self.program.declarations.pattern_lookup.add(
                        &pattern.pattern,
                        self.sequence_constructor(&template.sequence_constructor)?,
                    );
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

    fn sequence_constructor(
        &self,
        sequence_constructor: &ast::SequenceConstructor,
    ) -> error::SpannedResult<function::InlineFunctionId> {
        let mut items = sequence_constructor.iter();
        let left = items.next().unwrap();
        // let left_bindings = Ok(self.sequence_constructor_item(left)?);
        // TODO: compile bindings into function for program, then
        // return function id
        todo!();
        // items.fold(left, |left, right| {
        //     let left = self.sequence_constructor_item(left)?;
        //     let right = self.sequence_constructor_item(right)?;
        //     Ok(ir::Binary {
        //         left: ,
        //         op: ir::BinaryOperator::Comma,
        //         right
        //     )
        // })
    }

    fn sequence_constructor_item(
        &mut self,
        item: &ast::SequenceConstructorItem,
    ) -> error::SpannedResult<Bindings> {
        match item {
            ast::SequenceConstructorItem::ElementNode(element_node) => {
                let mut name_bindings = self.element_name(&element_node.name)?;
                let name_atom = name_bindings.atom();
                let expr = ir::Expr::Element(ir::XmlElement { name: name_atom });
                let binding = self.new_binding(expr, (0..0).into());
                Ok(Bindings::new(binding))
            }
            _ => todo!(),
        }
    }

    fn element_name(&mut self, name: &ast::Name) -> error::SpannedResult<Bindings> {
        let local_name = Spanned::new(
            ir::Atom::Const(ir::Const::String(name.local.clone())),
            (0..0).into(),
        );
        let namespace = Spanned::new(
            ir::Atom::Const(ir::Const::String(name.namespace.clone())),
            (0..0).into(),
        );
        let binding = self.new_binding(
            ir::Expr::XmlName(ir::XmlName {
                local_name,
                namespace,
            }),
            (0..0).into(),
        );
        Ok(Bindings::new(binding))
    }
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

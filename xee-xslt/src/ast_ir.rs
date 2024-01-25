use ahash::HashSetExt;
use xee_name::Namespaces;
use xee_xpath_ast::ast::Span;

use xee_interpreter::{context::StaticContext, error, interpreter};
use xee_ir::{ir, Binding, Bindings, FunctionBuilder, InterpreterCompiler, Scopes, Variables};
use xee_xpath_ast::span::Spanned;
use xee_xslt_ast::{ast, parse_transform};

struct IrConverter<'a> {
    variables: Variables,
    static_context: &'a StaticContext<'a>,
}

pub fn compile(
    transform: ast::Transform,
    static_context: &StaticContext,
) -> error::SpannedResult<interpreter::Program> {
    let mut ir_converter = IrConverter::new(static_context);
    let declarations = ir_converter.transform(&transform)?;
    let mut program = interpreter::Program::new((0..0).into());
    let mut scopes = Scopes::new();
    let builder = FunctionBuilder::new(&mut program);
    let mut compiler = InterpreterCompiler::new(builder, &mut scopes, static_context);
    compiler.compile_declarations(&declarations)?;
    Ok(program)
}

pub(crate) fn parse(
    static_context: &StaticContext,
    xslt: &str,
) -> error::SpannedResult<interpreter::Program> {
    let transform = parse_transform(xslt).unwrap(); // TODO get rid of error definitely wrong
    compile(transform, static_context)
}

impl<'a> IrConverter<'a> {
    fn new(static_context: &'a StaticContext<'a>) -> Self {
        IrConverter {
            variables: Variables::new(),
            static_context,
        }
    }

    fn new_binding(&mut self, expr: ir::Expr, span: Span) -> Binding {
        let name = self.variables.new_name();
        Binding::new(name, expr, span)
    }

    fn main_sequence_constructor(&mut self) -> ast::SequenceConstructor {
        vec![ast::SequenceConstructorItem::Instruction(
            ast::SequenceConstructorInstruction::ApplyTemplates(Box::new(ast::ApplyTemplates {
                mode: None,
                select: Some(ast::Expression {
                    xpath: xee_xpath_ast::ast::XPath::parse(
                        "/",
                        &Namespaces::default(),
                        &xee_name::VariableNames::new(),
                    )
                    .unwrap(),
                    span: xee_xslt_ast::ast::Span::new(0, 0),
                }),
                content: vec![],
                span: xee_xslt_ast::ast::Span::new(0, 0),
            })),
        )]
    }

    fn transform(&mut self, transform: &ast::Transform) -> error::SpannedResult<ir::Declarations> {
        let main_sequence_constructor = self.main_sequence_constructor();
        let main = self.sequence_constructor_function(&main_sequence_constructor)?;
        let mut declarations = ir::Declarations::new(main);
        for declaration in &transform.declarations {
            self.declaration(&mut declarations, declaration)?;
        }
        Ok(declarations)
    }

    fn declaration(
        &mut self,
        declarations: &mut ir::Declarations,
        declaration: &ast::Declaration,
    ) -> error::SpannedResult<()> {
        use ast::Declaration::*;
        match declaration {
            Template(template) => {
                if let Some(pattern) = &template.match_ {
                    let function_definition =
                        self.sequence_constructor_function(&template.sequence_constructor)?;
                    declarations.rules.push(ir::Rule {
                        pattern: pattern.pattern.clone(),
                        function_definition,
                    });
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

    fn sequence_constructor_function(
        &mut self,
        sequence_constructor: &ast::SequenceConstructor,
    ) -> error::SpannedResult<ir::FunctionDefinition> {
        let context_names = self.variables.push_context();
        let bindings = self.sequence_constructor(sequence_constructor)?;
        self.variables.pop_context();
        let params = vec![
            ir::Param {
                name: context_names.item,
                type_: None,
            },
            ir::Param {
                name: context_names.position,
                type_: None,
            },
            ir::Param {
                name: context_names.last,
                type_: None,
            },
        ];
        Ok(ir::FunctionDefinition {
            params,
            return_type: None,
            body: Box::new(bindings.expr()),
        })
    }

    fn sequence_constructor(
        &mut self,
        sequence_constructor: &ast::SequenceConstructor,
    ) -> error::SpannedResult<Bindings> {
        let mut items = sequence_constructor.iter();
        let left = items.next();
        if let Some(left) = left {
            let span_start = 0; // TODO
            let left_bindings = Ok(self.sequence_constructor_item(left)?);
            items.fold(left_bindings, |left, right| {
                let mut left_bindings = left?;
                let mut right_bindings = self.sequence_constructor_item(right)?;
                let expr = ir::Expr::Binary(ir::Binary {
                    left: left_bindings.atom(),
                    op: ir::BinaryOperator::Comma,
                    right: right_bindings.atom(),
                });
                let span_end = 0; // TODO
                let span = (span_start..span_end).into();
                let binding = self.new_binding(expr, span);
                Ok(left_bindings.concat(right_bindings).bind(binding))
            })
        } else {
            Ok(Bindings::empty())
        }
    }

    fn sequence_constructor_item(
        &mut self,
        item: &ast::SequenceConstructorItem,
    ) -> error::SpannedResult<Bindings> {
        match item {
            ast::SequenceConstructorItem::ElementNode(element_node) => {
                let mut bindings = self.element_name(&element_node.name)?;
                let name_atom = bindings.atom();
                let expr = ir::Expr::Element(ir::XmlElement { name: name_atom });
                let element_binding = self.new_binding(expr, (0..0).into());
                let mut bindings = bindings.bind(element_binding);
                let bindings = if !element_node.sequence_constructor.is_empty() {
                    let element_atom = bindings.atom();
                    let mut content_bindings =
                        self.sequence_constructor(&element_node.sequence_constructor)?;
                    let content_atom = content_bindings.atom();
                    let bindings = bindings.concat(content_bindings);
                    let append = ir::Expr::XmlAppend(ir::XmlAppend {
                        parent: element_atom,
                        child: content_atom,
                    });
                    let append_binding = self.new_binding(append, (0..0).into());
                    bindings.bind(append_binding)
                } else {
                    bindings
                };
                Ok(bindings)
            }
            ast::SequenceConstructorItem::Instruction(instruction) => {
                self.sequence_constructor_instruction(instruction)
            }
            _ => todo!(),
        }
    }

    fn sequence_constructor_instruction(
        &mut self,
        instruction: &ast::SequenceConstructorInstruction,
    ) -> error::SpannedResult<Bindings> {
        use ast::SequenceConstructorInstruction::*;
        match instruction {
            ApplyTemplates(apply_templates) => self.apply_templates(apply_templates),
            _ => todo!(),
        }
    }

    fn apply_templates(
        &mut self,
        apply_templates: &ast::ApplyTemplates,
    ) -> error::SpannedResult<Bindings> {
        // TODO: default for select should be child::node()
        let mut bindings = self.expression(apply_templates.select.as_ref().unwrap())?;
        let select_atom = bindings.atom();
        let expr = ir::Expr::ApplyTemplates(ir::ApplyTemplates {
            select: select_atom,
        });
        let binding = self.new_binding(expr, (0..0).into());
        Ok(bindings.bind(binding))
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

    fn expression(&mut self, expression: &ast::Expression) -> error::SpannedResult<Bindings> {
        let mut ir_converter =
            xee_xpath::IrConverter::new(&mut self.variables, self.static_context);
        ir_converter.expr(&expression.xpath.0)
    }
}

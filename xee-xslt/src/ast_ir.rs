use ahash::{HashMap, HashMapExt};
use xee_xpath_ast::ast::Span;

use xee_interpreter::{context::StaticContext, error, function, interpreter};
use xee_ir::{ir, Binding, Bindings, FunctionBuilder, InterpreterCompiler, Scopes};
use xee_xpath_ast::span::Spanned;
use xee_xslt_ast::{ast, parse_transform};

struct IrConverter<'a> {
    program: interpreter::Program,
    static_context: &'a StaticContext<'a>,
    counter: usize,
    variables: HashMap<ast::Name, ir::Name>,
}

pub(crate) fn compile(
    static_context: &StaticContext,
    transform: ast::Transform,
) -> error::SpannedResult<interpreter::Program> {
    let ir_converter = IrConverter::new(static_context);
    ir_converter.convert_transform(&transform)
}

pub(crate) fn parse(
    static_context: &StaticContext,
    xslt: &str,
) -> error::SpannedResult<interpreter::Program> {
    let transform = parse_transform(xslt).unwrap(); // TODO get rid of error definitely wrong
    compile(static_context, transform)
}

impl<'a> IrConverter<'a> {
    fn new(static_context: &'a StaticContext<'a>) -> Self {
        IrConverter {
            program: interpreter::Program::new((0..0).into()),
            static_context,
            counter: 0,
            variables: HashMap::new(),
        }
    }

    fn program(self) -> interpreter::Program {
        self.program
    }

    fn new_name(&mut self) -> ir::Name {
        let name = format!("x{}", self.counter);
        self.counter += 1;
        ir::Name::new(name)
    }

    fn new_var_name(&mut self, name: &ast::Name) -> ir::Name {
        self.variables.get(name).cloned().unwrap_or_else(|| {
            let new_name = self.new_name();
            self.variables.insert(name.clone(), new_name.clone());
            new_name
        })
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
                    let function_id =
                        self.sequence_constructor_function_id(&template.sequence_constructor)?;
                    self.program
                        .declarations
                        .pattern_lookup
                        .add(&pattern.pattern, function_id);
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

    fn sequence_constructor_function_id(
        &mut self,
        sequence_constructor: &ast::SequenceConstructor,
    ) -> error::SpannedResult<function::InlineFunctionId> {
        let bindings = self.sequence_constructor(sequence_constructor)?;
        let params = vec![
            ir::Param {
                name: ir::Name::new("item".to_string()),
                type_: None,
            },
            ir::Param {
                name: ir::Name::new("position".to_string()),
                type_: None,
            },
            ir::Param {
                name: ir::Name::new("last".to_string()),
                type_: None,
            },
        ];
        let function_definition = ir::FunctionDefinition {
            params,
            return_type: None,
            body: Box::new(bindings.expr()),
        };
        let mut scopes = Scopes::new();
        let builder = FunctionBuilder::new(&mut self.program);
        let mut compiler = InterpreterCompiler::new(builder, &mut scopes, self.static_context);
        compiler.compile_function_id(&function_definition, (0..0).into())
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

use ahash::HashSetExt;
use xee_name::{Name, Namespaces, FN_NAMESPACE};

use xee_interpreter::{context::StaticContext, error, interpreter};
use xee_ir::{compile_xslt, ir, Bindings, Variables};
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
    compile_xslt(declarations, static_context)
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

    fn simple_content_atom(&mut self) -> ir::Atom {
        self.static_function_atom("simple-content", Some(FN_NAMESPACE), 2)
    }

    fn concat_atom(&mut self, arity: u8) -> ir::Atom {
        self.static_function_atom("concat", Some(FN_NAMESPACE), arity)
    }

    fn static_function_atom(&mut self, name: &str, namespace: Option<&str>, arity: u8) -> ir::Atom {
        ir::Atom::Const(ir::Const::StaticFunctionReference(
            self.static_context
                .functions
                .get_by_name(
                    &Name::new(name.to_string(), namespace.map(|ns| ns.to_string()), None),
                    arity,
                )
                .unwrap(),
            None,
        ))
    }

    fn simple_content_expr(
        &mut self,
        select_atom: ir::AtomS,
        separator_atom: ir::AtomS,
    ) -> ir::Expr {
        ir::Expr::FunctionCall(ir::FunctionCall {
            atom: Spanned::new(self.simple_content_atom(), (0..0).into()),
            args: vec![select_atom, separator_atom],
        })
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
        sequence_constructor: &[ast::SequenceConstructorItem],
    ) -> error::SpannedResult<Bindings> {
        let mut items = sequence_constructor.iter();
        let left = items.next();
        if let Some(left) = left {
            if let Some((name, var_bindings)) = self.variable(left)? {
                let expr = ir::Expr::Let(ir::Let {
                    name,
                    var_expr: Box::new(var_bindings.expr()),
                    return_expr: Box::new(self.sequence_constructor(items.as_slice())?.expr()),
                });
                return Ok(Bindings::new(
                    self.variables.new_binding(expr, (0..0).into()),
                ));
            }
            self.sequence_constructor_concat(left, items)
        } else {
            let empty_sequence = self.empty_sequence();
            Ok(Bindings::new(
                self.variables
                    .new_binding(empty_sequence.value, empty_sequence.span),
            ))
        }
    }

    fn sequence_constructor_concat<'b>(
        &mut self,
        left: &ast::SequenceConstructorItem,
        items: impl Iterator<Item = &'b ast::SequenceConstructorItem>,
    ) -> error::SpannedResult<Bindings> {
        let left_bindings = Ok(self.sequence_constructor_item(left)?);
        items.fold(left_bindings, |left, right| {
            let mut left_bindings = left?;
            let mut right_bindings = self.sequence_constructor_item(right)?;
            let expr = ir::Expr::Binary(ir::Binary {
                left: left_bindings.atom(),
                op: ir::BinaryOperator::Comma,
                right: right_bindings.atom(),
            });
            let binding = self.variables.new_binding_no_span(expr);
            Ok(left_bindings.concat(right_bindings).bind(binding))
        })
    }

    fn sequence_constructor_item(
        &mut self,
        item: &ast::SequenceConstructorItem,
    ) -> error::SpannedResult<Bindings> {
        match item {
            ast::SequenceConstructorItem::ElementNode(element_node) => {
                let mut bindings = self.element_name(&element_node.name)?;
                let name_atom = bindings.atom();
                let expr = ir::Expr::XmlElement(ir::XmlElement { name: name_atom });
                let element_binding = self.variables.new_binding_no_span(expr);
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
                    let append_binding = self.variables.new_binding_no_span(append);
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
            ValueOf(value_of) => self.value_of(value_of),
            If(if_) => self.if_(if_),
            Choose(choose) => self.choose(choose),
            // a bunch of language-like instructions are supported earlier
            Variable(_variable) => unreachable!(),
            _ => todo!(),
        }
    }

    fn apply_templates(
        &mut self,
        apply_templates: &ast::ApplyTemplates,
    ) -> error::SpannedResult<Bindings> {
        // TODO: default for select should be child::node()
        let (select_atom, bindings) = self
            .expression(apply_templates.select.as_ref().unwrap())?
            .atom_bindings();
        // let select_atom = bindings.atom();
        Ok(bindings.bind_expr_no_span(
            &mut self.variables,
            ir::Expr::ApplyTemplates(ir::ApplyTemplates {
                select: select_atom,
            }),
        ))
    }

    fn value_of(&mut self, value_of: &ast::ValueOf) -> error::SpannedResult<Bindings> {
        if let Some(select) = &value_of.select {
            let (select_atom, select_bindings) = self.expression(select)?.atom_bindings();
            let (separator_atom, separator_bindings) = if let Some(separator) = &value_of.separator
            {
                self.attribute_value_template(separator)?
            } else {
                Bindings::new(
                    self.variables
                        .new_binding_no_span(ir::Expr::Atom(Spanned::new(
                            ir::Atom::Const(ir::Const::String(" ".to_string())),
                            (0..0).into(),
                        ))),
                )
            }
            .atom_bindings();
            let bindings = select_bindings.concat(separator_bindings);
            let expr = self.simple_content_expr(select_atom, separator_atom);
            let (text_atom, bindings) = bindings
                .bind_expr_no_span(&mut self.variables, expr)
                .atom_bindings();
            Ok(bindings.bind_expr_no_span(
                &mut self.variables,
                ir::Expr::XmlText(ir::XmlText { value: text_atom }),
            ))
        } else {
            todo!()
        }
    }

    fn attribute_value_template(
        &mut self,
        value_template: &ast::ValueTemplate<String>,
    ) -> error::SpannedResult<Bindings> {
        let mut all_bindings = Vec::new();
        for item in &value_template.template {
            let bindings = match item {
                ast::ValueTemplateItem::String { text, span: _span } => {
                    let text_atom = Spanned::new(
                        ir::Atom::Const(ir::Const::String(text.clone())),
                        (0..0).into(),
                    );
                    let bindings = Bindings::empty();
                    bindings.bind_expr_no_span(&mut self.variables, ir::Expr::Atom(text_atom))
                }
                ast::ValueTemplateItem::Curly { c } => {
                    let text_atom = Spanned::new(
                        ir::Atom::Const(ir::Const::String(c.to_string())),
                        (0..0).into(),
                    );
                    let bindings = Bindings::empty();
                    bindings.bind_expr_no_span(&mut self.variables, ir::Expr::Atom(text_atom))
                }
                ast::ValueTemplateItem::Value { xpath, span: _ } => {
                    let (atom, bindings) = self.xpath(&xpath.0)?.atom_bindings();
                    let expr = self.simple_content_expr(
                        atom,
                        Spanned::new(
                            ir::Atom::Const(ir::Const::String(" ".to_string())),
                            (0..0).into(),
                        ),
                    );
                    bindings.bind_expr_no_span(&mut self.variables, expr)
                }
            };
            all_bindings.push(bindings);
        }
        Ok(if all_bindings.is_empty() {
            // empty attribute value template is a string
            let bindings = Bindings::empty();
            bindings.bind_expr_no_span(
                &mut self.variables,
                ir::Expr::Atom(Spanned::new(
                    ir::Atom::Const(ir::Const::String("".to_string())),
                    (0..0).into(),
                )),
            )
        } else if all_bindings.len() == 1 {
            // a single binding is just that binding
            all_bindings.pop().unwrap()
        } else {
            // TODO: speculative code, needs tests
            // if we have multiple bindings, concatenate each result into
            // a single string
            let mut combined_bindings = Bindings::empty();
            let mut atoms = Vec::new();
            for binding in all_bindings {
                let (atom, binding) = binding.atom_bindings();
                combined_bindings = combined_bindings.concat(binding);
                atoms.push(atom);
            }
            // concatenate all the pieces of content into a single string
            // TODO: this may create more than we have arities for, so we may want to use more
            // generic concat function that takes a sequence at some point
            let concat_atom = self.concat_atom(atoms.len() as u8);
            let expr = ir::Expr::FunctionCall(ir::FunctionCall {
                atom: Spanned::new(concat_atom, (0..0).into()),
                args: atoms,
            });
            combined_bindings.bind_expr_no_span(&mut self.variables, expr)
        })
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
        let binding = self
            .variables
            .new_binding_no_span(ir::Expr::XmlName(ir::XmlName {
                local_name,
                namespace,
            }));
        Ok(Bindings::new(binding))
    }

    fn variable(
        &mut self,
        item: &ast::SequenceConstructorItem,
    ) -> error::SpannedResult<Option<(ir::Name, Bindings)>> {
        if let ast::SequenceConstructorItem::Instruction(
            ast::SequenceConstructorInstruction::Variable(variable),
        ) = item
        {
            let name = self.variables.new_var_name(&variable.name);
            if let Some(select) = &variable.select {
                let var_bindings = self.expression(select)?;
                Ok(Some((name, var_bindings)))
            } else {
                todo!()
            }
        } else {
            Ok(None)
        }
    }

    fn empty_sequence(&mut self) -> ir::ExprS {
        Spanned::new(
            ir::Expr::Atom(Spanned::new(
                ir::Atom::Const(ir::Const::EmptySequence),
                (0..0).into(),
            )),
            (0..0).into(),
        )
    }

    fn if_(&mut self, if_: &ast::If) -> error::SpannedResult<Bindings> {
        let (condition, bindings) = self.expression(&if_.test)?.atom_bindings();
        let expr = ir::Expr::If(ir::If {
            condition,
            then: Box::new(self.sequence_constructor(&if_.sequence_constructor)?.expr()),
            else_: Box::new(self.empty_sequence()),
        });
        Ok(bindings.bind_expr_no_span(&mut self.variables, expr))
    }

    fn choose(&mut self, choose: &ast::Choose) -> error::SpannedResult<Bindings> {
        self.choose_when_otherwise(&choose.when, choose.otherwise.as_ref())
        // let first = &choose.when.first().unwrap();
        // let rest = &choose.when[1..];

        // let (condition, bindings) = self.expression(&choose.when[0].test)?.atom_bindings();
        // if let Some(otherwise) = &choose.otherwise {
        //     let expr = ir::Expr::If(ir::If {
        //         condition,
        //         then: Box::new(
        //             self.sequence_constructor(&choose.when[0].sequence_constructor)?
        //                 .expr(),
        //         ),
        //         else_: Box::new(
        //             self.sequence_constructor(&otherwise.sequence_constructor)?
        //                 .expr(),
        //         ),
        //     });
        //     Ok(bindings.bind_expr_no_span(&mut self.variables, expr))
        // } else {
        //     let expr = ir::Expr::If(ir::If {
        //         condition,
        //         then: Box::new(
        //             self.sequence_constructor(&choose.when[0].sequence_constructor)?
        //                 .expr(),
        //         ),
        //         else_: Box::new(self.empty_sequence()),
        //     });
        //     Ok(bindings.bind_expr_no_span(&mut self.variables, expr))
        // }
    }

    fn choose_when_otherwise(
        &mut self,
        when: &[ast::When],
        otherwise: Option<&ast::Otherwise>,
    ) -> error::SpannedResult<Bindings> {
        let first = &when.first().unwrap();
        let rest = &when[1..];

        let (condition, bindings) = self.expression(&first.test)?.atom_bindings();
        let else_expr = if !rest.is_empty() {
            self.choose_when_otherwise(rest, otherwise)?.expr()
        } else if let Some(otherwise) = otherwise {
            self.sequence_constructor(&otherwise.sequence_constructor)?
                .expr()
        } else {
            self.empty_sequence()
        };

        let expr = ir::Expr::If(ir::If {
            condition,
            then: Box::new(
                self.sequence_constructor(&first.sequence_constructor)?
                    .expr(),
            ),
            else_: Box::new(else_expr),
        });
        Ok(bindings.bind_expr_no_span(&mut self.variables, expr))
    }

    fn expression(&mut self, expression: &ast::Expression) -> error::SpannedResult<Bindings> {
        self.xpath(&expression.xpath.0)
    }

    fn xpath(&mut self, xpath: &xee_xpath_ast::ast::ExprS) -> error::SpannedResult<Bindings> {
        let mut ir_converter =
            xee_xpath::IrConverter::new(&mut self.variables, self.static_context);
        ir_converter.expr(xpath)
    }
}

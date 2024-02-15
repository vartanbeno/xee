use ahash::HashSetExt;
use xee_name::{Name, Namespaces, FN_NAMESPACE};

use xee_interpreter::{context::StaticContext, error, interpreter};
use xee_ir::{compile_xslt, ir, Bindings, Variables};
use xee_xpath_ast::{ast as xpath_ast, pattern::transform_pattern, span::Spanned};
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

    // fn error_atom(&mut self) -> ir::Atom {
    //     self.static_function_atom("error", Some(FN_NAMESPACE), 0)
    // }

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
                        pattern: transform_pattern(&pattern.pattern, |expr| {
                            self.pattern_predicate(expr)
                        })?,
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
            ast::SequenceConstructorItem::Instruction(instruction) => {
                self.sequence_constructor_instruction(instruction)
            }
            ast::SequenceConstructorItem::Content(content) => {
                self.sequence_constructor_content(content)
            }
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
            ForEach(for_each) => self.for_each(for_each),
            Copy(copy) => self.copy(copy),
            CopyOf(copy_of) => self.copy_of(copy_of),
            Sequence(sequence) => self.sequence(sequence),
            Element(element) => self.element(element),
            Text(text) => self.text(text),
            // xsl:variable does not produce content and is handled earlier already
            Variable(_variable) => unreachable!(),
            _ => todo!(),
        }
    }

    fn sequence_constructor_content(
        &mut self,
        content: &ast::Content,
    ) -> error::SpannedResult<Bindings> {
        match content {
            ast::Content::Element(element_node) => {
                self.sequence_constructor_content_element(element_node)
            }
            ast::Content::Text(text) => {
                let text_atom = Spanned::new(
                    ir::Atom::Const(ir::Const::String(text.clone())),
                    (0..0).into(),
                );
                let bindings = Bindings::empty();
                Ok(bindings.bind_expr_no_span(
                    &mut self.variables,
                    ir::Expr::XmlText(ir::XmlText { value: text_atom }),
                ))
            }
            ast::Content::Value(expression) => {
                let (atom, bindings) = self.expression(expression)?.atom_bindings();
                let expr = self.simple_content_expr(atom, self.space_separator_atom());
                let (text_atom, bindings) = bindings
                    .bind_expr_no_span(&mut self.variables, expr)
                    .atom_bindings();
                Ok(bindings.bind_expr_no_span(
                    &mut self.variables,
                    ir::Expr::XmlText(ir::XmlText { value: text_atom }),
                ))
            }
        }
    }

    fn sequence_constructor_content_element(
        &mut self,
        element_node: &ast::ElementNode,
    ) -> error::SpannedResult<Bindings> {
        let (name_atom, bindings) = self.xml_name(&element_node.name)?.atom_bindings();
        let name_expr = ir::Expr::XmlElement(ir::XmlElement { name: name_atom });
        let (element_atom, mut bindings) = bindings
            .bind_expr_no_span(&mut self.variables, name_expr)
            .atom_bindings();
        for (name, value) in &element_node.attributes {
            let (value_atom, value_bindings) =
                self.attribute_value_template(value)?.atom_bindings();
            let (attribute_name_atom, attribute_bindings) = self.xml_name(name)?.atom_bindings();
            let value_bindings = value_bindings.concat(attribute_bindings);
            let attribute_expr = ir::Expr::XmlAttribute(ir::XmlAttribute {
                name: attribute_name_atom,
                value: value_atom,
            });
            let (attribute_atom, attribute_bindings) = value_bindings
                .bind_expr_no_span(&mut self.variables, attribute_expr)
                .atom_bindings();
            let append_expr = ir::Expr::XmlAppend(ir::XmlAppend {
                parent: element_atom.clone(),
                child: attribute_atom,
            });
            let append_bindings =
                attribute_bindings.bind_expr_no_span(&mut self.variables, append_expr);
            bindings = bindings.concat(append_bindings);
        }
        let sequence_constructor_bindings = self.sequence_constructor_append(
            element_atom.clone(),
            &element_node.sequence_constructor,
        )?;
        let bindings = bindings.concat(sequence_constructor_bindings);
        Ok(bindings)
    }

    fn sequence_constructor_append(
        &mut self,
        element_atom: ir::AtomS,
        sequence_constructor: &ast::SequenceConstructor,
    ) -> error::SpannedResult<Bindings> {
        if !sequence_constructor.is_empty() {
            let (atom, bindings) = self
                .sequence_constructor(sequence_constructor)?
                .atom_bindings();
            let append = ir::Expr::XmlAppend(ir::XmlAppend {
                parent: element_atom,
                child: atom,
            });
            let bindings = bindings.bind_expr_no_span(&mut self.variables, append);
            Ok(bindings)
        } else {
            Ok(Bindings::empty())
        }
    }

    fn space_separator_atom(&self) -> ir::AtomS {
        Spanned::new(
            ir::Atom::Const(ir::Const::String(" ".to_string())),
            (0..0).into(),
        )
    }

    fn apply_templates(
        &mut self,
        apply_templates: &ast::ApplyTemplates,
    ) -> error::SpannedResult<Bindings> {
        // TODO: default for select should be child::node()
        let (select_atom, bindings) = self
            .expression(apply_templates.select.as_ref().unwrap())?
            .atom_bindings();
        Ok(bindings.bind_expr_no_span(
            &mut self.variables,
            ir::Expr::ApplyTemplates(ir::ApplyTemplates {
                select: select_atom,
            }),
        ))
    }

    fn select_or_sequence_constructor(
        &mut self,
        instruction: &impl ast::SelectOrSequenceConstructor,
    ) -> error::SpannedResult<Bindings> {
        if let Some(select) = instruction.select() {
            self.expression(select)
        } else {
            self.sequence_constructor(instruction.sequence_constructor())
        }
    }

    fn value_of(&mut self, value_of: &ast::ValueOf) -> error::SpannedResult<Bindings> {
        let (select_atom, select_bindings) = self
            .select_or_sequence_constructor(value_of)?
            .atom_bindings();

        let (separator_atom, separator_bindings) = if let Some(separator) = &value_of.separator {
            self.attribute_value_template(separator)?
        } else {
            Bindings::new(
                self.variables
                    .new_binding_no_span(ir::Expr::Atom(self.space_separator_atom())),
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
                    let expr = self.simple_content_expr(atom, self.space_separator_atom());
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

    fn xml_name(&mut self, name: &ast::Name) -> error::SpannedResult<Bindings> {
        let local_name = Spanned::new(
            ir::Atom::Const(ir::Const::String(name.local_name().to_string())),
            (0..0).into(),
        );
        let namespace = Spanned::new(
            ir::Atom::Const(ir::Const::String(
                name.namespace().unwrap_or("").to_string(),
            )),
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
            let var_bindings = if let Some(select) = &variable.select {
                self.expression(select)?
            } else {
                self.sequence_constructor(&variable.sequence_constructor)?
            };
            Ok(Some((name, var_bindings)))
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

    fn for_each(&mut self, for_each: &ast::ForEach) -> error::SpannedResult<Bindings> {
        let (var_atom, bindings) = self.expression(&for_each.select)?.atom_bindings();

        let context_names = self.variables.push_context();
        let return_bindings = self.sequence_constructor(&for_each.sequence_constructor)?;
        self.variables.pop_context();
        let expr = ir::Expr::Map(ir::Map {
            context_names,
            var_atom,
            return_expr: Box::new(return_bindings.expr()),
        });

        Ok(bindings.bind_expr_no_span(&mut self.variables, expr))
    }

    fn copy(&mut self, copy: &ast::Copy) -> error::SpannedResult<Bindings> {
        let (context_atom, bindings) = if let Some(select) = &copy.select {
            self.expression(select)?.atom_bindings()
        } else {
            self.variables.context_item((0..0).into())?.atom_bindings()
        };
        // copy shallow this item
        let expr = ir::Expr::CopyShallow(ir::CopyShallow {
            select: context_atom,
        });
        let (copy_atom, bindings) = bindings
            .bind_expr_no_span(&mut self.variables, expr)
            .atom_bindings();

        // if it is an element or document,
        // execute sequence constructor
        // TODO: work on document check
        // let _is_document_expr = self.is_document_expr(context_atom.clone());
        let is_element_expr = self.is_element_expr(copy_atom.clone());
        let (is_element_atom, bindings) = bindings
            .bind_expr_no_span(&mut self.variables, is_element_expr)
            .atom_bindings();

        let copy_expr = ir::Expr::Atom(copy_atom.clone());

        let (sequence_constructor_atom, sequence_constructor_bindings) = self
            .sequence_constructor(&copy.sequence_constructor)?
            .atom_bindings();

        let bindings = bindings.concat(sequence_constructor_bindings);

        let append = ir::Expr::XmlAppend(ir::XmlAppend {
            parent: copy_atom,
            child: sequence_constructor_atom,
        });

        let if_expr = ir::Expr::If(ir::If {
            condition: is_element_atom,
            then: Box::new(Spanned::new(append, (0..0).into())),
            else_: Box::new(Spanned::new(copy_expr, (0..0).into())),
        });

        Ok(bindings.bind_expr_no_span(&mut self.variables, if_expr))
    }

    // fn is_document_expr(&self, atom: ir::AtomS) -> ir::Expr {
    //     ir::Expr::InstanceOf(ir::InstanceOf {
    //         atom,
    //         sequence_type: xpath_ast::SequenceType::Item(xpath_ast::Item {
    //             item_type: xpath_ast::ItemType::KindTest(xpath_ast::KindTest::Document(None)),
    //             occurrence: xpath_ast::Occurrence::One,
    //         }),
    //     })
    // }

    fn is_element_expr(&self, atom: ir::AtomS) -> ir::Expr {
        ir::Expr::InstanceOf(ir::InstanceOf {
            atom,
            sequence_type: xpath_ast::SequenceType::Item(xpath_ast::Item {
                item_type: xpath_ast::ItemType::KindTest(xpath_ast::KindTest::Element(None)),
                occurrence: xpath_ast::Occurrence::One,
            }),
        })
    }

    fn copy_of(&mut self, copy_of: &ast::CopyOf) -> error::SpannedResult<Bindings> {
        let (atom, bindings) = self.expression(&copy_of.select)?.atom_bindings();
        let copy_deep_expr = ir::Expr::CopyDeep(ir::CopyDeep { select: atom });
        Ok(bindings.bind_expr_no_span(&mut self.variables, copy_deep_expr))
    }

    fn sequence(&mut self, sequence: &ast::Sequence) -> error::SpannedResult<Bindings> {
        self.select_or_sequence_constructor(sequence)
    }

    // the xsl:element instruction
    fn element(&mut self, element: &ast::Element) -> error::SpannedResult<Bindings> {
        let (localname_atom, bindings) = self
            .attribute_value_template(&element.name)?
            .atom_bindings();
        let (namespace_atom, namespace_bindings) = if let Some(namespace) = &element.namespace {
            self.attribute_value_template(namespace)?.atom_bindings()
        } else {
            let namespace_atom = Spanned::new(
                ir::Atom::Const(ir::Const::String("".to_string())),
                (0..0).into(),
            );
            (namespace_atom, Bindings::empty())
        };
        let bindings = bindings.concat(namespace_bindings);
        let name = ir::Expr::XmlName(ir::XmlName {
            local_name: localname_atom,
            namespace: namespace_atom,
        });
        let (name_atom, bindings) = bindings
            .bind_expr_no_span(&mut self.variables, name)
            .atom_bindings();

        let expr = ir::Expr::XmlElement(ir::XmlElement { name: name_atom });
        let (element_atom, bindings) = bindings
            .bind_expr_no_span(&mut self.variables, expr)
            .atom_bindings();
        let sequence_constructor_bindings =
            self.sequence_constructor_append(element_atom, &element.sequence_constructor)?;
        Ok(bindings.concat(sequence_constructor_bindings))
    }

    fn text(&mut self, text: &ast::Text) -> error::SpannedResult<Bindings> {
        let (atom, bindings) = self
            .attribute_value_template(&text.content)?
            .atom_bindings();
        Ok(bindings.bind_expr_no_span(
            &mut self.variables,
            ir::Expr::XmlText(ir::XmlText { value: atom }),
        ))
    }

    // fn throw_error(&mut self) -> error::SpannedResult<Bindings> {
    //     let error_atom = self.error_atom();
    //     let expr = ir::Expr::FunctionCall(ir::FunctionCall {
    //         atom: Spanned::new(error_atom, (0..0).into()),
    //         args: vec![],
    //     });
    //     Ok(Bindings::new(self.variables.new_binding_no_span(expr)))
    // }

    fn expression(&mut self, expression: &ast::Expression) -> error::SpannedResult<Bindings> {
        self.xpath(&expression.xpath.0)
    }

    fn xpath(&mut self, xpath: &xee_xpath_ast::ast::ExprS) -> error::SpannedResult<Bindings> {
        let mut ir_converter =
            xee_xpath::IrConverter::new(&mut self.variables, self.static_context);
        ir_converter.expr(xpath)
    }

    fn pattern_predicate(
        &mut self,
        expr: &xpath_ast::ExprS,
    ) -> error::SpannedResult<ir::FunctionDefinition> {
        let context_names = self.variables.push_context();
        let bindings = self.xpath(expr)?;
        self.variables.pop_context();
        // a predicate is a function that takes a sequence as an argument and returns
        // a boolean that is true if the sequence matches the predicate
        let name = self.variables.new_name();
        let var_atom = Spanned::new(ir::Atom::Variable(name.clone()), (0..0).into());
        let filter = ir::Expr::PatternPredicate(ir::PatternPredicate {
            context_names: context_names.clone(),
            var_atom,
            expr: Box::new(bindings.expr()),
        });
        let bindings = bindings.bind_expr(&mut self.variables, Spanned::new(filter, (0..0).into()));

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
}

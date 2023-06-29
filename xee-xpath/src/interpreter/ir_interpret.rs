use miette::SourceSpan;

use crate::context::{ContextRule, StaticContext};
use crate::error::{Error, Result};
use crate::ir;
use crate::stack;

use super::builder::{BackwardJumpRef, ForwardJumpRef, FunctionBuilder, JumpCondition};
use super::instruction::Instruction;

pub(crate) type Scopes = crate::interpreter::scope::Scopes<ir::Name>;

pub(crate) struct InterpreterCompiler<'a> {
    pub(crate) scopes: &'a mut Scopes,
    pub(crate) static_context: &'a StaticContext<'a>,
    pub(crate) builder: FunctionBuilder<'a>,
}

impl<'a> InterpreterCompiler<'a> {
    pub(crate) fn compile_expr(&mut self, expr: &ir::ExprS) -> Result<()> {
        let span = expr.span;
        match &expr.value {
            ir::Expr::Atom(atom) => self.compile_atom(atom),
            ir::Expr::Let(let_) => self.compile_let(let_, span),
            ir::Expr::Binary(binary) => self.compile_binary(binary, span),
            ir::Expr::Unary(unary) => self.compile_unary(unary, span),
            ir::Expr::FunctionDefinition(function_definition) => {
                self.compile_function_definition(function_definition, span)
            }
            ir::Expr::FunctionCall(function_call) => {
                self.compile_function_call(function_call, span)
            }
            ir::Expr::Step(step) => self.compile_step(step, span),
            ir::Expr::If(if_) => self.compile_if(if_, span),
            ir::Expr::Map(map) => self.compile_map(map, span),
            ir::Expr::Filter(filter) => self.compile_filter(filter, span),
            ir::Expr::Quantified(quantified) => self.compile_quantified(quantified, span),
        }
    }

    fn compile_atom(&mut self, atom: &ir::AtomS) -> Result<()> {
        match &atom.value {
            ir::Atom::Const(c) => {
                match c {
                    ir::Const::Integer(i) => {
                        self.builder.emit_constant((*i).into(), atom.span);
                    }
                    ir::Const::String(s) => {
                        self.builder.emit_constant((s).into(), atom.span);
                    }
                    ir::Const::Double(d) => {
                        self.builder.emit_constant((*d).into(), atom.span);
                    }
                    ir::Const::Decimal(d) => {
                        self.builder.emit_constant((*d).into(), atom.span);
                    }
                    ir::Const::EmptySequence => {
                        self.builder.emit_constant(stack::Value::Empty, atom.span)
                    }
                    ir::Const::StaticFunctionReference(static_function_id, context_names) => {
                        self.compile_static_function_reference(
                            *static_function_id,
                            context_names.as_ref(),
                            atom.span,
                        )?;
                    }
                };
                Ok(())
            }
            ir::Atom::Variable(name) => self.compile_variable(name, atom.span),
        }
    }

    fn compile_variable(&mut self, name: &ir::Name, span: SourceSpan) -> Result<()> {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                return Err(Error::XPDY0130);
            }
            self.builder.emit(Instruction::Var(index as u16), span);
            Ok(())
        } else {
            // if value is in any outer scopes
            if self.scopes.is_closed_over_name(name) {
                let index = self.builder.add_closure_name(name);
                if index > u16::MAX as usize {
                    return Err(Error::XPDY0130);
                }
                self.builder
                    .emit(Instruction::ClosureVar(index as u16), span);
                Ok(())
            } else {
                unreachable!("variable not found: {:?}", name);
            }
        }
    }

    fn compile_variable_set(&mut self, name: &ir::Name, span: SourceSpan) -> Result<()> {
        if let Some(index) = self.scopes.get(name) {
            if index > u16::MAX as usize {
                return Err(Error::XPDY0130);
            }
            self.builder.emit(Instruction::Set(index as u16), span);
        } else {
            panic!("can only set locals: {:?}", name);
        }
        Ok(())
    }

    fn compile_let(&mut self, let_: &ir::Let, span: SourceSpan) -> Result<()> {
        self.compile_expr(&let_.var_expr)?;
        self.scopes.push_name(&let_.name);
        self.compile_expr(&let_.return_expr)?;
        self.builder.emit(Instruction::LetDone, span);
        self.scopes.pop_name();
        Ok(())
    }

    fn compile_if(&mut self, if_: &ir::If, span: SourceSpan) -> Result<()> {
        self.compile_atom(&if_.condition)?;
        let jump_else = self.builder.emit_jump_forward(JumpCondition::False, span);
        self.compile_expr(&if_.then)?;
        let jump_end = self.builder.emit_jump_forward(JumpCondition::Always, span);
        self.builder.patch_jump(jump_else);
        self.compile_expr(&if_.else_)?;
        self.builder.patch_jump(jump_end);
        Ok(())
    }

    fn compile_binary(&mut self, binary: &ir::Binary, span: SourceSpan) -> Result<()> {
        self.compile_atom(&binary.left)?;
        self.compile_atom(&binary.right)?;
        match &binary.op {
            ir::BinaryOperator::Add => {
                self.builder.emit(Instruction::Add, span);
            }
            ir::BinaryOperator::Sub => {
                self.builder.emit(Instruction::Sub, span);
            }
            ir::BinaryOperator::Mul => {
                self.builder.emit(Instruction::Mul, span);
            }
            ir::BinaryOperator::Div => {
                self.builder.emit(Instruction::Div, span);
            }
            ir::BinaryOperator::IntDiv => {
                self.builder.emit(Instruction::IntDiv, span);
            }
            ir::BinaryOperator::Mod => {
                self.builder.emit(Instruction::Mod, span);
            }
            ir::BinaryOperator::ValueEq => {
                self.builder.emit(Instruction::Eq, span);
            }
            ir::BinaryOperator::ValueNe => {
                self.builder.emit(Instruction::Ne, span);
            }
            ir::BinaryOperator::ValueLt => {
                self.builder.emit(Instruction::Lt, span);
            }
            ir::BinaryOperator::ValueLe => {
                self.builder.emit(Instruction::Le, span);
            }
            ir::BinaryOperator::ValueGt => {
                self.builder.emit(Instruction::Gt, span);
            }
            ir::BinaryOperator::ValueGe => {
                self.builder.emit(Instruction::Ge, span);
            }
            ir::BinaryOperator::GenEq => {
                self.builder.emit(Instruction::GenEq, span);
            }
            ir::BinaryOperator::GenNe => {
                self.builder.emit(Instruction::GenNe, span);
            }
            ir::BinaryOperator::GenLt => {
                self.builder.emit(Instruction::GenLt, span);
            }
            ir::BinaryOperator::GenLe => {
                self.builder.emit(Instruction::GenLe, span);
            }
            ir::BinaryOperator::GenGt => {
                self.builder.emit(Instruction::GenGt, span);
            }
            ir::BinaryOperator::GenGe => {
                self.builder.emit(Instruction::GenGe, span);
            }
            ir::BinaryOperator::Comma => {
                self.builder.emit(Instruction::Comma, span);
            }
            ir::BinaryOperator::Union => {
                self.builder.emit(Instruction::Union, span);
            }
            ir::BinaryOperator::Range => {
                self.builder.emit(Instruction::Range, span);
            }
            ir::BinaryOperator::Concat => {
                self.builder.emit(Instruction::Concat, span);
            }
            ir::BinaryOperator::And => {
                // XXX we don't do any short-circuiting of evaluation yet
                let first_false = self.builder.emit_jump_forward(JumpCondition::False, span);
                let second_true = self.builder.emit_jump_forward(JumpCondition::True, span);
                self.builder.patch_jump(first_false);
                // pop the second item on the stack
                self.builder.emit(Instruction::Pop, span);
                self.builder.emit_constant(false.into(), span);
                let end = self.builder.emit_jump_forward(JumpCondition::Always, span);
                self.builder.patch_jump(second_true);
                self.builder.emit_constant(true.into(), span);
                self.builder.patch_jump(end);
            }
            ir::BinaryOperator::Or => {
                // XXX we don't do any short-circuiting of evaluation yet
                let first_true = self.builder.emit_jump_forward(JumpCondition::True, span);
                let second_true = self.builder.emit_jump_forward(JumpCondition::True, span);
                // neither first nor second were true, so we return false
                self.builder.emit_constant(false.into(), span);
                let end = self.builder.emit_jump_forward(JumpCondition::Always, span);
                self.builder.patch_jump(first_true);
                self.builder.patch_jump(second_true);
                self.builder.emit_constant(true.into(), span);
                self.builder.patch_jump(end);
            }
            _ => todo!("operator not supported yet: {:?}", binary.op),
        }
        Ok(())
    }

    fn compile_unary(&mut self, unary: &ir::Unary, span: SourceSpan) -> Result<()> {
        self.compile_atom(&unary.atom)?;
        match unary.op {
            ir::UnaryOperator::Plus => {
                self.builder.emit(Instruction::Plus, span);
            }
            ir::UnaryOperator::Minus => {
                self.builder.emit(Instruction::Minus, span);
            }
        }
        Ok(())
    }

    fn compile_function_definition(
        &mut self,
        function_definition: &ir::FunctionDefinition,
        span: SourceSpan,
    ) -> Result<()> {
        let nested_builder = self.builder.builder();
        self.scopes.push_scope();

        let mut compiler = InterpreterCompiler {
            builder: nested_builder,
            scopes: self.scopes,
            static_context: self.static_context,
        };

        for param in &function_definition.params {
            compiler.scopes.push_name(&param.0);
        }
        compiler.compile_expr(&function_definition.body)?;
        for _ in &function_definition.params {
            compiler.scopes.pop_name();
        }

        compiler.scopes.pop_scope();

        let function =
            compiler
                .builder
                .finish("inline".to_string(), function_definition.params.len(), span);
        // now place all captured names on stack, to ensure we have the
        // closure
        // in reverse order so we can pop them off in the right order
        for name in function.closure_names.iter().rev() {
            self.compile_variable(name, span)?;
        }
        let function_id = self.builder.add_function(function);
        self.builder
            .emit(Instruction::Closure(function_id.as_u16()), span);
        Ok(())
    }

    fn compile_static_function_reference(
        &mut self,
        static_function_id: stack::StaticFunctionId,
        context_names: Option<&ir::ContextNames>,
        span: SourceSpan,
    ) -> Result<()> {
        let static_function = self
            .static_context
            .functions
            .get_by_index(static_function_id);
        match static_function.context_rule {
            Some(ContextRule::ItemFirst) => {
                // XXX optional context names; what if context is absent?
                let context_names = context_names.unwrap();
                self.compile_variable(&context_names.item, span)?
            }
            Some(ContextRule::ItemLast) => {
                let context_names = context_names.unwrap();
                self.compile_variable(&context_names.item, span)?
            }
            Some(ContextRule::PositionFirst) => self.compile_variable(
                {
                    let context_names = context_names.unwrap();
                    &context_names.position
                },
                span,
            )?,
            Some(ContextRule::SizeFirst) => {
                let context_names = context_names.unwrap();
                self.compile_variable(&context_names.last, span)?
            }
            None => {}
        }
        self.builder.emit(
            Instruction::StaticClosure(static_function_id.as_u16()),
            span,
        );
        Ok(())
    }

    fn compile_function_call(
        &mut self,
        function_call: &ir::FunctionCall,
        span: SourceSpan,
    ) -> Result<()> {
        self.compile_atom(&function_call.atom)?;
        for arg in &function_call.args {
            self.compile_atom(arg)?;
        }
        self.builder
            .emit(Instruction::Call(function_call.args.len() as u8), span);
        Ok(())
    }

    fn compile_step(&mut self, step: &ir::Step, span: SourceSpan) -> Result<()> {
        self.compile_atom(&step.context)?;
        let step_id = self.builder.add_step(step.step.clone());
        self.builder.emit(Instruction::Step(step_id as u16), span);
        Ok(())
    }

    fn compile_map(&mut self, map: &ir::Map, span: SourceSpan) -> Result<()> {
        // place the resulting sequence on the stack
        let new_sequence = ir::Name("xee_new_sequence".to_string());
        self.scopes.push_name(&new_sequence);
        self.builder.emit(Instruction::BuildNew, span);

        let (loop_start, loop_end) =
            self.compile_sequence_loop_init(&map.var_atom, &map.context_names, span)?;

        self.compile_sequence_get_item(&map.var_atom, &map.context_names, span)?;
        // name it
        self.scopes.push_name(&map.context_names.item);
        // execute the map expression, placing result on stack
        self.compile_expr(&map.return_expr)?;
        self.scopes.pop_name();

        // push result to new sequence
        self.compile_variable(&new_sequence, span)?;
        self.builder.emit(Instruction::BuildPush, span);

        // clean up the var_name item
        self.builder.emit(Instruction::Pop, span);

        self.compile_sequence_loop_iterate(loop_start, &map.context_names, span)?;

        self.builder.patch_jump(loop_end);
        self.compile_sequence_loop_end(span);

        self.builder.emit(Instruction::BuildComplete, span);
        // pop new sequence name & sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
        Ok(())
    }

    fn compile_filter(&mut self, filter: &ir::Filter, span: SourceSpan) -> Result<()> {
        // place the resulting sequence on the stack
        let new_sequence = ir::Name("xee_new_sequence".to_string());
        self.scopes.push_name(&new_sequence);
        self.builder.emit(Instruction::BuildNew, span);

        let (loop_start, loop_end) =
            self.compile_sequence_loop_init(&filter.var_atom, &filter.context_names, span)?;

        // place item to filter on stack
        self.compile_sequence_get_item(&filter.var_atom, &filter.context_names, span)?;
        // name it
        self.scopes.push_name(&filter.context_names.item);
        // execute the filter expression, placing result on stack
        self.compile_expr(&filter.return_expr)?;
        self.scopes.pop_name();
        // duplicate result so we can do the IsNumeric check
        self.builder.emit(Instruction::Dup, span);
        // the resulting value can be a numeric value
        self.builder.emit(Instruction::IsNumeric, span);
        // if it's not a numeric expression we're going to interpret it as boolean,
        // a normal filter
        let is_not_numeric = self.builder.emit_jump_forward(JumpCondition::False, span);
        // It was numeric, we have on the stack a position to compare with
        self.compile_variable(&filter.context_names.position, span)?;
        self.builder.emit(Instruction::Eq, span);
        // Now we have a boolean on the stack: a normal filter

        // We take the effective boolean value of the result
        // if filter is false, we skip this item
        self.builder.patch_jump(is_not_numeric);
        let is_included = self.builder.emit_jump_forward(JumpCondition::True, span);
        // we need to clean up the stack after this
        self.builder.emit(Instruction::Pop, span);
        // and iterate the loop
        let iterate = self.builder.emit_jump_forward(JumpCondition::Always, span);

        self.builder.patch_jump(is_included);
        // push item to new sequence
        self.compile_variable(&new_sequence, span)?;
        self.builder.emit(Instruction::BuildPush, span);

        self.builder.patch_jump(iterate);
        // no need to clean up the stack, as filter get is pushed onto sequence
        self.compile_sequence_loop_iterate(loop_start, &filter.context_names, span)?;

        self.builder.patch_jump(loop_end);
        self.compile_sequence_loop_end(span);

        self.builder.emit(Instruction::BuildComplete, span);
        // pop new sequence name & sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        self.scopes.pop_name();
        Ok(())
    }

    fn compile_quantified(&mut self, quantified: &ir::Quantified, span: SourceSpan) -> Result<()> {
        let (loop_start, loop_end) =
            self.compile_sequence_loop_init(&quantified.var_atom, &quantified.context_names, span)?;

        self.compile_sequence_get_item(&quantified.var_atom, &quantified.context_names, span)?;
        // name it
        self.scopes.push_name(&quantified.context_names.item);
        // execute the satisfies expression, placing result in on stack
        self.compile_expr(&quantified.satisifies_expr)?;
        self.scopes.pop_name();

        let jump_out_end = match quantified.quantifier {
            ir::Quantifier::Some => self.builder.emit_jump_forward(JumpCondition::True, span),
            ir::Quantifier::Every => self.builder.emit_jump_forward(JumpCondition::False, span),
        };
        // we didn't jump out, clean up quantifier variable
        self.builder.emit(Instruction::Pop, span);

        self.compile_sequence_loop_iterate(loop_start, &quantified.context_names, span)?;

        self.builder.patch_jump(loop_end);

        // if we reached the end, without jumping out
        self.compile_sequence_loop_end(span);

        let reached_end_value = match quantified.quantifier {
            ir::Quantifier::Some => false.into(),
            ir::Quantifier::Every => true.into(),
        };
        self.builder.emit_constant(reached_end_value, span);
        let end = self.builder.emit_jump_forward(JumpCondition::Always, span);

        // we jumped out
        self.builder.patch_jump(jump_out_end);
        // clean up quantifier variable
        self.builder.emit(Instruction::Pop, span);
        self.compile_sequence_loop_end(span);

        let jumped_out_value = match quantified.quantifier {
            ir::Quantifier::Some => true.into(),
            ir::Quantifier::Every => false.into(),
        };
        // if we jumped out, we set satisfies to true
        self.builder.emit_constant(jumped_out_value, span);

        self.builder.patch_jump(end);
        // pop sequence length name & index
        self.scopes.pop_name();
        self.scopes.pop_name();
        Ok(())
    }

    fn compile_sequence_loop_init(
        &mut self,
        atom: &ir::AtomS,
        context_names: &ir::ContextNames,
        span: SourceSpan,
    ) -> Result<(BackwardJumpRef, ForwardJumpRef)> {
        //  sequence length
        self.compile_atom(atom)?;
        self.scopes.push_name(&context_names.last);
        self.builder.emit(Instruction::SequenceLen, span);

        // place index on stack
        self.builder.emit_constant(1i64.into(), span);
        self.scopes.push_name(&context_names.position);

        let loop_start_ref = self.builder.loop_start();

        // compare with sequence length, if index is gt length, we're done with the loop
        self.compile_variable(&context_names.position, span)?;
        self.compile_variable(&context_names.last, span)?;
        self.builder.emit(Instruction::Gt, span);
        // check whether index is gt length, if so, we're done with the loop
        let loop_end_ref = self.builder.emit_jump_forward(JumpCondition::True, span);

        Ok((loop_start_ref, loop_end_ref))
    }

    fn compile_sequence_get_item(
        &mut self,
        atom: &ir::AtomS,
        context_names: &ir::ContextNames,
        span: SourceSpan,
    ) -> Result<()> {
        // get item at the index
        self.compile_variable(&context_names.position, span)?;
        self.compile_atom(atom)?;
        self.builder.emit(Instruction::SequenceGet, span);
        Ok(())
    }

    fn compile_sequence_loop_iterate(
        &mut self,
        loop_start: BackwardJumpRef,
        context_names: &ir::ContextNames,
        span: SourceSpan,
    ) -> Result<()> {
        // update index with 1
        self.compile_variable(&context_names.position, span)?;
        self.builder.emit_constant(1i64.into(), span);
        self.builder.emit(Instruction::Add, span);
        self.compile_variable_set(&context_names.position, span)?;
        self.builder
            .emit_jump_backward(loop_start, JumpCondition::Always, span);
        Ok(())
    }

    fn compile_sequence_loop_end(&mut self, span: SourceSpan) {
        // pop length and index
        self.builder.emit(Instruction::Pop, span);
        self.builder.emit(Instruction::Pop, span);
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_debug_snapshot;
    use xee_xpath_ast::{ast, Namespaces};
    use xot::Xot;

    use crate::atomic;
    use crate::context::{DynamicContext, StaticContext};
    use crate::error::Result;
    use crate::output;
    use crate::run::evaluate;
    use crate::sequence;
    use crate::stack;
    use crate::xml;
    use crate::xpath::XPath;

    fn xot_nodes_to_items(node: &[xot::Node]) -> output::Sequence {
        output::Sequence::from(
            node.iter()
                .map(|&node| sequence::Item::from(xml::Node::Xot(node)))
                .collect::<Vec<_>>(),
        )
    }

    fn run(s: &str) -> Result<stack::Value> {
        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let xpath = XPath::new(context.static_context, s)?;
        xpath.run_value(&context, None)
    }

    fn run_with_variables(
        s: &str,
        variables: &[(ast::Name, Vec<sequence::Item>)],
    ) -> Result<stack::Value> {
        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let variable_names = variables
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let static_context = StaticContext::with_variable_names(&namespaces, &variable_names);
        let context = DynamicContext::with_variables(&xot, &static_context, variables);
        let xpath = XPath::new(context.static_context, s)?;
        xpath.run_value(&context, None)
    }

    fn run_debug(s: &str) -> Result<stack::Value> {
        let xot = Xot::new();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::new(&xot, &static_context);
        let xpath = XPath::new(context.static_context, s)?;
        dbg!(&xpath.program.get_function(0).decoded());
        xpath.run_value(&context, None)
    }

    fn run_xml(xml: &str, xpath: &str) -> Result<output::Sequence> {
        evaluate(xml, xpath, None)
    }

    fn run_xml_default_ns(xml: &str, xpath: &str, ns: &str) -> Result<output::Sequence> {
        evaluate(xml, xpath, Some(ns))
    }

    fn assert_nodes<S>(xml: &str, xpath: &str, get_nodes: S) -> Result<()>
    where
        S: Fn(&Xot, &xml::Document) -> Vec<xot::Node>,
    {
        let mut xot = Xot::new();
        let uri = xml::Uri("http://example.com".to_string());
        let mut documents = xml::Documents::new();
        documents.add(&mut xot, &uri, xml).unwrap();
        let namespaces = Namespaces::new(None, None);
        let static_context = StaticContext::new(&namespaces);
        let context = DynamicContext::with_documents(&xot, &static_context, &documents);
        let document = documents.get(&uri).unwrap();
        let nodes = get_nodes(&xot, document);

        let xpath = XPath::new(context.static_context, xpath)?;
        let result = xpath.many_xot_node(&context, document.root)?;
        assert_eq!(result, xot_nodes_to_items(&nodes));
        Ok(())
    }

    #[test]
    fn test_compile_add() {
        assert_debug_snapshot!(run("1 + 2"));
    }

    #[test]
    fn test_nested() {
        assert_debug_snapshot!(run("1 + (8 - 2)"));
    }

    #[test]
    fn test_comma() {
        assert_debug_snapshot!(run("1, 2"));
    }

    #[test]
    fn test_empty_sequence() {
        assert_debug_snapshot!(run("()"));
    }

    #[test]
    fn test_comma_squences() {
        assert_debug_snapshot!(run("(1, 2), (3, 4)"));
    }

    #[test]
    fn test_let() {
        assert_debug_snapshot!(run("let $x := 1 return $x + 2"));
    }

    #[test]
    fn test_let_nested() {
        assert_debug_snapshot!(run("let $x := 1, $y := $x + 3 return $y + 5"));
    }

    #[test]
    fn test_let_on_right_side() {
        assert_debug_snapshot!(run("1 + (let $x := 2 return $x + 10)"));
    }

    #[test]
    fn test_if() {
        assert_debug_snapshot!(run("if (1) then 2 else 3"));
    }

    #[test]
    fn test_if_false() {
        assert_debug_snapshot!(run("if (0) then 2 else 3"));
    }

    #[test]
    fn test_if_with_let_true() {
        assert_debug_snapshot!(run(
            "if (1) then (let $x := 2 return $x) else (let $x := 3 return $x)"
        ));
    }

    #[test]
    fn test_if_with_let_false() {
        assert_debug_snapshot!(run(
            "if (0) then (let $x := 2 return $x) else (let $x := 3 return $x)"
        ));
    }

    #[test]
    fn test_value_eq_true() {
        assert_debug_snapshot!(run("1 eq 1"));
    }

    #[test]
    fn test_value_eq_false() {
        assert_debug_snapshot!(run("1 eq 2"));
    }

    #[test]
    fn test_value_ne_true() {
        assert_debug_snapshot!(run("1 ne 2"));
    }

    #[test]
    fn test_value_ne_false() {
        assert_debug_snapshot!(run("1 ne 1"));
    }

    #[test]
    fn test_value_lt_true() {
        assert_debug_snapshot!(run("1 lt 2"));
    }

    #[test]
    fn test_value_lt_false() {
        assert_debug_snapshot!(run("2 lt 1"));
    }

    #[test]
    fn test_inline_function_without_args() {
        assert_debug_snapshot!(run("function() { 5 } ()"));
    }

    #[test]
    fn test_inline_function_with_single_arg() {
        assert_debug_snapshot!(run("function($x) { $x + 5 } (3)"));
    }

    #[test]
    fn test_inline_function_with_multiple_args() {
        assert_debug_snapshot!(run("function($x, $y) { $x + $y } (3, 5)"));
    }

    #[test]
    fn test_function_nested() {
        assert_debug_snapshot!(run("function($x) { function($y) { $y + 2 }($x + 1) } (5)"));
    }

    #[test]
    fn test_function_closure() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function() { $x + 2 } }()()"
        ));
    }

    #[test]
    fn test_function_closure_with_multiple_variables() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3, $y := 1 return function() { $x - $y } }()()"
        ));
    }

    #[test]
    fn test_function_closure_with_multiple_variables_arguments() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function($y) { $x - $y } }()(1)"
        ));
    }

    #[test]
    fn test_function_closure_nested() {
        assert_debug_snapshot!(run(
            "function() { let $x := 3 return function() { let $y := 4 return function() { $x + $y }} }()()()"
        ));
    }

    #[test]
    fn test_static_function_call() {
        assert_debug_snapshot!(run("my_function(5, 2)"));
    }

    #[test]
    fn test_named_function_ref_call() {
        assert_debug_snapshot!(run("my_function#2(5, 2)"));
    }

    #[test]
    fn test_static_call_with_placeholders() {
        assert_debug_snapshot!(run("my_function(?, 2)(5)"));
    }

    #[test]
    fn test_inline_function_with_args_placeholdered() {
        assert_debug_snapshot!(run("function($x, $y) { $x - $y } ( ?, 3 ) (5)"));
    }

    #[test]
    fn test_inline_function_with_args_placeholdered2() {
        assert_debug_snapshot!(run("function($x, $y) { $x - $y } ( ?, 3 ) (?) (5)"));
    }

    #[test]
    fn test_inline_function_call_with_let() {
        assert_debug_snapshot!(run(
            "function($x, $y) { $x + $y }(let $a := 1 return $a, let $b := 2 return $b)"
        ));
    }

    #[test]
    fn test_inline_function_call_with_let2() {
        assert_debug_snapshot!(run(
            "let $a := 1 return function($x, $y) { $x + $y }($a, let $b := 2 return $b)"
        ));
    }

    #[test]
    fn test_range() {
        assert_debug_snapshot!(run("1 to 5"));
    }

    #[test]
    fn test_range_greater() {
        assert_debug_snapshot!(run("5 to 1"));
    }

    #[test]
    fn test_range_equal() {
        assert_debug_snapshot!(run("1 to 1"));
    }

    #[test]
    fn test_for_loop() {
        assert_debug_snapshot!(run("for $x in 1 to 5 return $x + 2"));
    }

    #[test]
    fn test_nested_for_loop() {
        assert_debug_snapshot!(run("for $i in (10, 20, 30), $j in (1, 2) return $i + $j"));
    }

    #[test]
    fn test_nested_for_loop_variable_scope() {
        assert_debug_snapshot!(run(
            "for $i in (10, 20), $j in ($i + 1, $i + 2) return $i + $j"
        ));
    }

    #[test]
    fn test_simple_map() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1)"));
    }

    #[test]
    fn test_simple_map_sequence() {
        assert_debug_snapshot!(run("(1, 2) ! (., 0)"));
    }

    #[test]
    fn test_simple_map_single() {
        assert_debug_snapshot!(run("1 ! (. , 0)"));
    }

    #[test]
    fn test_simple_map_multiple_steps() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1) ! (. + 2)"));
    }

    #[test]
    fn test_simple_map_multiple_steps2() {
        assert_debug_snapshot!(run("(1, 2) ! (. + 1) ! (. + 2) ! (. + 3)"));
    }

    #[test]
    fn test_simple_map_position() {
        assert_debug_snapshot!(run("(4, 5, 6) ! (fn:position())"));
    }

    #[test]
    fn test_simple_map_last() {
        assert_debug_snapshot!(run("(4, 5, 6) ! (fn:last())"));
    }

    #[test]
    fn test_some_quantifier_expr_true() {
        assert_debug_snapshot!(run("some $x in (1, 2, 3) satisfies $x eq 2"));
    }

    #[test]
    fn test_some_quantifier_expr_false() {
        assert_debug_snapshot!(run("some $x in (1, 2, 3) satisfies $x eq 5"));
    }

    #[test]
    fn test_nested_some_quantifier_expr_true() {
        assert_debug_snapshot!(run("some $x in (1, 2, 3), $y in (2, 3) satisfies $x gt $y"));
    }

    #[test]
    fn test_every_quantifier_expr_true() {
        assert_debug_snapshot!(run("every $x in (1, 2, 3) satisfies $x lt 5"));
    }

    #[test]
    fn test_every_quantifier_expr_false() {
        assert_debug_snapshot!(run("every $x in (1, 2, 3) satisfies $x gt 2"));
    }

    #[test]
    fn test_every_quantifier_nested_true() {
        assert_debug_snapshot!(run(
            "every $x in (2, 3, 4), $y in (0, 1) satisfies $x gt $y"
        ));
    }

    #[test]
    fn test_every_quantifier_nested_false() {
        assert_debug_snapshot!(run(
            "every $x in (2, 3, 4), $y in (1, 2) satisfies $x gt $y"
        ));
    }

    #[test]
    fn test_some_quantifier_empty_sequence() {
        assert_debug_snapshot!(run("some $x in () satisfies $x eq 5"));
    }

    #[test]
    fn test_every_quantifier_empty_sequence() {
        assert_debug_snapshot!(run("every $x in () satisfies $x eq 5"));
    }

    #[test]
    fn test_predicate() {
        assert_debug_snapshot!(run("(1, 2, 3)[. ge 2]"));
    }

    #[test]
    fn test_predicate_empty_sequence() {
        assert_debug_snapshot!(run("() [. ge 1]"));
    }

    #[test]
    fn test_predicate_multiple() {
        assert_debug_snapshot!(run("(1, 2, 3)[. ge 2][. ge 3]"));
    }

    #[test]
    fn test_comma_simple_map() {
        assert_debug_snapshot!(run("(1, 2), (3, 4) ! (. + 1)"));
    }

    #[test]
    fn test_comma_simple_map2() {
        assert_debug_snapshot!(run("(1, 2), (3, 4), (5, 6) ! (. + 1)"));
    }

    #[test]
    fn test_simple_map_empty_sequence() {
        assert_debug_snapshot!(run("() ! (. + 1)"));
    }

    #[test]
    fn test_predicate_index() {
        assert_debug_snapshot!(run("(1, 2, 3)[2]"));
    }

    #[test]
    fn test_predicate_index2() {
        assert_debug_snapshot!(run("(1, 2, 3)[2] + (4, 5)[1]"));
    }

    #[test]
    fn test_predicate_index_all() {
        assert_debug_snapshot!(run("(1, 2, 3)[fn:position()]"));
    }

    #[test]
    fn test_predicate_index_not_whole_number() {
        // since no position matches, we should get the empty sequence
        assert_debug_snapshot!(run("(1, 2, 3)[2.5]"));
    }

    #[test]
    fn test_sequence_predicate() {
        // this should succeed, as IsNumeric sees the sequence as non-numeric.
        // We create the sequence with (2, 3)[. > 2] to ensure it's indeed a
        // sequence underneath, and not atomic. The sequence of a single value
        // is interpreted as boolean and thus we see the full sequence
        assert_debug_snapshot!(run("(1, 2, 3)[(2, 3)[. > 2]]"));
    }

    #[test]
    fn test_sequence_predicate_sequence_too_long() {
        // this should fail: 2, 3 is not a numeric nor an effective boolean value
        assert_debug_snapshot!(run("(1, 2, 3)[(2, 3)]"));
    }

    #[test]
    fn test_sequence_predicate_sequence_empty() {
        // the empty sequence is an effective boolean of false, so we should
        // get the result of the empty sequence
        assert_debug_snapshot!(run("(1, 2, 3)[()]"));
    }

    #[test]
    fn test_child_axis_step1() -> Result<()> {
        assert_nodes(r#"<doc><a/><b/></doc>"#, "doc/*", |xot, document| {
            let doc_el = xot.document_element(document.root).unwrap();
            let a = xot.first_child(doc_el).unwrap();
            let b = xot.next_sibling(a).unwrap();
            vec![a, b]
        })
    }

    #[test]
    fn test_child_axis_step2() -> Result<()> {
        assert_nodes(r#"<doc><a/><b/></doc>"#, "doc/a", |xot, document| {
            let doc_el = xot.document_element(document.root).unwrap();
            let a = xot.first_child(doc_el).unwrap();
            vec![a]
        })
    }

    #[test]
    fn test_step_with_predicate() -> Result<()> {
        assert_nodes(
            r#"<doc><a/><b/></doc>"#,
            "doc/*[fn:position() eq 2]",
            |xot, document| {
                let doc_el = xot.document_element(document.root).unwrap();
                let a = xot.first_child(doc_el).unwrap();
                let b = xot.next_sibling(a).unwrap();
                vec![b]
            },
        )
    }

    #[test]
    fn test_descendant_axis_step() -> Result<()> {
        assert_nodes(
            r#"<doc><a/><b><c/></b></doc>"#,
            "descendant::*",
            |xot, document| {
                let doc_el = xot.document_element(document.root).unwrap();
                let a = xot.first_child(doc_el).unwrap();
                let b = xot.next_sibling(a).unwrap();
                let c = xot.first_child(b).unwrap();
                vec![doc_el, a, b, c]
            },
        )
    }

    #[test]
    fn test_descendant_axis_position() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "descendant::* / fn:position()"
        ));
    }

    #[test]
    fn test_descendant_axis_step2() -> Result<()> {
        assert_nodes(
            r#"<doc><a><c/></a><b/></doc>"#,
            "descendant::*",
            |xot, document| {
                let doc_el = xot.document_element(document.root).unwrap();
                let a = xot.first_child(doc_el).unwrap();
                let b = xot.next_sibling(a).unwrap();
                let c = xot.first_child(a).unwrap();
                vec![doc_el, a, c, b]
            },
        )
    }

    #[test]
    fn test_comma_nodes() -> Result<()> {
        assert_nodes(r#"<doc><a/><b/></doc>"#, "doc/b, doc/a", |xot, document| {
            let doc_el = xot.document_element(document.root).unwrap();
            let a = xot.first_child(doc_el).unwrap();
            let b = xot.next_sibling(a).unwrap();
            vec![b, a]
        })
    }

    #[test]
    fn test_union() -> Result<()> {
        assert_nodes(
            r#"<doc><a/><b/><c/></doc>"#,
            "doc/c | doc/a | doc/b | doc/a",
            |xot, document| {
                let doc_el = xot.document_element(document.root).unwrap();
                let a = xot.first_child(doc_el).unwrap();
                let b = xot.next_sibling(a).unwrap();
                let c = xot.next_sibling(b).unwrap();
                vec![a, b, c]
            },
        )
    }

    #[test]
    fn test_default_position() {
        assert_debug_snapshot!(run_xml("<doc/>", "fn:position()"));
    }

    #[test]
    fn test_default_position_no_context() {
        assert_debug_snapshot!(run("fn:position()"));
    }

    #[test]
    fn test_default_last() {
        assert_debug_snapshot!(run_xml("<doc/>", "fn:last()"));
    }

    #[test]
    fn test_default_last_no_context() {
        assert_debug_snapshot!(run("fn:last()"));
    }

    #[test]
    fn test_position_closure() {
        assert_debug_snapshot!(run("(3, 4) ! (let $p := fn:position#0 return $p())"));
    }

    #[test]
    fn test_simple_string() {
        assert_debug_snapshot!(run("'hello'"));
    }

    #[test]
    fn test_simple_string_concat() {
        assert_debug_snapshot!(run("'hello' || 'world'"));
    }

    #[test]
    fn test_string_compare_eq_true() {
        assert_debug_snapshot!(run("'hello' eq 'hello'"));
    }

    #[test]
    fn test_string_compare_eq_false() {
        assert_debug_snapshot!(run("'hello' eq 'world'"));
    }

    #[test]
    fn test_local_name_element() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "descendant::* / fn:local-name()"
        ));
    }

    #[test]
    fn test_local_name_empty() {
        assert_debug_snapshot!(run("fn:local-name(())"));
    }

    #[test]
    fn test_namespace_uri_element() {
        assert_debug_snapshot!(run_xml(
            r#"<doc xmlns="http://example.com/" xmlns:e="http://example.com/e"><a/><b><e:c/></b></doc>"#,
            "descendant::* / fn:namespace-uri()"
        ));
    }

    #[test]
    fn test_count() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "fn:count(descendant::*)"
        ));
    }

    #[test]
    fn test_fn_root() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "doc/a / fn:root() / doc / fn:local-name()"
        ));
    }

    #[test]
    fn test_fn_root_explicit() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "fn:root(doc/a) / doc / b / fn:local-name()"
        ));
    }

    #[test]
    fn test_fn_root_absent() {
        assert_debug_snapshot!(run("fn:root()"));
    }

    #[test]
    fn test_fn_root_implicit() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "/doc/a / fn:local-name()"
        ));
    }

    #[test]
    fn test_fn_double_slash_root_implicit() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "//a / fn:local-name()"
        ));
    }

    #[test]
    fn test_fn_namespace_default() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a/><b><c/></b></doc>"#,
            "descendant::* / local-name()"
        ));
    }

    #[test]
    fn test_element_namespace_wrong() {
        // we expect no match, as doc is in a namespace and the default is None
        assert_debug_snapshot!(run_xml(
            r#"<doc xmlns="http://example.com"><a/></doc>"#,
            "doc / local-name()",
        ));
    }

    #[test]
    fn test_element_namespace_default() {
        // here we set the default element namespace for xpath expressions
        assert_debug_snapshot!(run_xml_default_ns(
            r#"<doc xmlns="http://example.com"><a/></doc>"#,
            "doc / local-name()",
            "http://example.com"
        ));
    }

    #[test]
    fn test_attribute_namespace_no_default() {
        // here we set the default element namespace for xpath expressions
        assert_debug_snapshot!(run_xml_default_ns(
            r#"<doc xmlns="http://example.com" a="hello"/>"#,
            "doc / @a / local-name()",
            "http://example.com"
        ));
    }

    #[test]
    fn test_string_document_node() {
        assert_debug_snapshot!(run_xml(r#"<doc><a>A</a><b>B</b></doc>"#, "string(doc)"));
    }

    #[test]
    fn test_string_element_node() {
        assert_debug_snapshot!(run_xml(r#"<doc><a>A</a><b>B</b></doc>"#, "string(doc/a)"));
    }

    #[test]
    fn test_string_integer() {
        assert_debug_snapshot!(run("fn:string(1)"));
    }

    #[test]
    fn test_atomize() {
        assert_debug_snapshot!(run("(1) eq (1)"));
    }

    #[test]
    fn test_atomize_xml_eq_true() {
        assert_debug_snapshot!(run_xml(r#"<doc><a>A</a><b>A</b></doc>"#, "doc/a eq doc/b",));
    }

    #[test]
    fn test_atomize_xml_eq_false() {
        assert_debug_snapshot!(run_xml(r#"<doc><a>A</a><b>B</b></doc>"#, "doc/a eq doc/b",));
    }

    #[test]
    fn test_atomize_xml_attribute_eq_true() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a f="FOO"/><b f="FOO"/></doc>"#,
            "doc/a/@f eq doc/b/@f",
        ));
    }

    #[test]
    fn test_atomize_xml_attribute_eq_false() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><a f="FOO"/><b f="BAR"/></doc>"#,
            "doc/a/@f eq doc/b/@f",
        ));
    }

    #[test]
    fn test_atomize_xml_attribute_present() {
        assert_debug_snapshot!(run_xml(r#"<doc><a f="FOO"/></doc>"#, "doc/a/@f eq 'FOO'",));
    }

    #[test]
    fn test_atomize_xml_attribute_missing() {
        assert_debug_snapshot!(run_xml(r#"<doc><a/></doc>"#, "doc/a/@f eq 'FOO'",));
    }

    #[test]
    fn test_attribute_predicate() -> Result<()> {
        assert_nodes(
            r#"<doc><a/><b foo="FOO"/><c/></doc>"#,
            "//*[@foo eq 'FOO']",
            |xot, document| {
                let doc_el = xot.document_element(document.root).unwrap();
                let a = xot.first_child(doc_el).unwrap();
                let b = xot.next_sibling(a).unwrap();
                vec![b]
            },
        )
    }

    #[test]
    fn test_external_variable() {
        assert_debug_snapshot!(run_with_variables(
            "$foo",
            &[(
                ast::Name::without_ns("foo"),
                vec![sequence::Item::from(atomic::Atomic::from("FOO"))]
            )],
        ))
    }

    #[test]
    fn test_external_variables() {
        assert_debug_snapshot!(run_with_variables(
            "$foo + $bar",
            &[
                (
                    ast::Name::without_ns("foo"),
                    vec![sequence::Item::from(atomic::Atomic::from(1i64))]
                ),
                (
                    ast::Name::without_ns("bar"),
                    vec![sequence::Item::from(atomic::Atomic::from(2i64))]
                )
            ]
        ))
    }

    #[test]
    fn test_absent_context() {
        assert_debug_snapshot!(run("."));
    }

    // This results in a type error, because the context is absent and no
    // operations with absent are permitted. This is not ideal - better would
    // be if the access to . already resulted in a XPDY0002 error. But
    // . is compiled away and no function call takes place (unlike for fn:position
    // fn:last), so we don't get an error at that level.
    #[test]
    fn test_absent_context_with_operation() {
        assert_debug_snapshot!(run(". + 1"));
    }

    // Same problem as before, type error instead of XPDY0002 error
    #[test]
    fn test_default_position_with_operation() {
        assert_debug_snapshot!(run("fn:position() + 1"));
    }

    #[test]
    fn test_string_compare_general_eq_true() {
        assert_debug_snapshot!(run("'hello' = 'hello'"));
    }

    #[test]
    fn test_compare_general_eq_sequence_true() {
        assert_debug_snapshot!(run("(1, 2) = (3, 2)"));
    }

    #[test]
    fn test_compare_general_eq_sequence_false() {
        assert_debug_snapshot!(run("(1, 2) = (3, 4)"));
    }

    #[test]
    fn test_generate_id() {
        assert_debug_snapshot!(run_xml(r#"<doc><a/><b/><c/></doc>"#, "generate-id(doc/a)",));
    }

    #[test]
    fn test_fn_string() {
        assert_debug_snapshot!(run_xml(
            r#"<doc><p>Hello world!</p></doc>"#,
            "/doc/p/string()",
        ));
    }

    #[test]
    fn test_let_uses_own_variable() {
        assert_debug_snapshot!(run("let $x := $x return $x"));
    }

    #[test]
    fn test_static_function_call_nested() {
        assert_debug_snapshot!(run(r#"fn:string-join(("A"),xs:string("A"))"#));
    }

    #[test]
    fn test_run_unary_minus() {
        assert_debug_snapshot!(run("-1"));
    }
}

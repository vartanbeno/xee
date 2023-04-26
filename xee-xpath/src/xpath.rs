use crate::ast_ir::Converter;
use crate::builder::{FunctionBuilder, Program};
use crate::context::Context;
use crate::error::Result;
use crate::interpret::Interpreter;
use crate::ir;
use crate::ir_interpret::{InterpreterCompiler, Scopes};
use crate::parse_ast::parse_xpath;
use crate::value::{Atomic, FunctionId, Item, Node, StackValue};

pub(crate) struct CompiledXPath<'a> {
    pub(crate) program: Program,
    context: &'a Context<'a>,
    main: FunctionId,
}

impl<'a> CompiledXPath<'a> {
    pub(crate) fn new(context: &'a Context, xpath: &str) -> Self {
        let ast = parse_xpath(xpath);
        let mut converter = Converter::new(&context.static_context);
        let expr = converter.convert_xpath(&ast);
        // we get an inline function, unwrap it for now
        let (arg_name, expr) = unwrap_inline_function(expr);
        let mut program = Program::new();
        let mut scopes = Scopes::new(ir::Name("dummy".to_string()));
        let builder = FunctionBuilder::new(&mut program);
        let mut compiler = InterpreterCompiler {
            builder,
            scopes: &mut scopes,
            context,
            sequence_length_name: &ir::Name("xee_sequence_length".to_string()),
            sequence_index_name: &ir::Name("xee_sequence_index".to_string()),
        };
        compiler.scopes.push_name(&arg_name);
        compiler.compile_expr(&expr);

        let main = compiler.builder.finish("main".to_string(), 0);
        let main = program.add_function(main);
        Self {
            program,
            context,
            main,
        }
    }

    pub(crate) fn interpret(&self) -> Result<StackValue> {
        // a fake context value
        self.interpret_with_context(Item::Atomic(Atomic::Integer(0)))
    }

    pub(crate) fn interpret_with_context(&self, context_item: Item) -> Result<StackValue> {
        let mut interpreter = Interpreter::new(&self.program, self.context, context_item);
        interpreter.start(self.main);
        interpreter.run()?;
        // the stack has to be 1 values and return the result of the expression
        // why 1 value if the context item is on the top of the stack? This is because
        // the outer main function will pop the context item; this code is there to
        // remove the function id from the stack but the main function has no function id
        assert_eq!(
            interpreter.stack().len(),
            1,
            "stack must only have 1 value but found {:?}",
            interpreter.stack()
        );
        Ok(interpreter.stack().last().unwrap().clone())
    }

    pub(crate) fn interpret_with_xot_node(&self, node: xot::Node) -> Result<StackValue> {
        self.interpret_with_context(Item::Node(Node::Xot(node)))
    }
}

fn unwrap_inline_function(expr: ir::Expr) -> (ir::Name, ir::Expr) {
    match expr {
        ir::Expr::FunctionDefinition(ir::FunctionDefinition { params, body, .. }) => {
            assert_eq!(params.len(), 3);
            (params[0].0.clone(), *body)
        }
        _ => panic!("expected inline function"),
    }
}

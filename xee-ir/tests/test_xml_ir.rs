use insta::assert_debug_snapshot;
use xee_interpreter::interpreter::{instruction::decode_instructions, Program};
use xee_ir::{ir, FunctionBuilder, InterpreterCompiler, Scopes};
use xee_xpath_ast::span::Spanned;

fn spanned<T>(t: T) -> Spanned<T> {
    Spanned::new(t, (0..0).into())
}

#[test]
fn test_generate_element() {
    // first come up with the element name
    let local_name = ir::Atom::Const(ir::Const::String("foo".to_string()));
    let namespace = ir::Atom::Const(ir::Const::String("".to_string()));
    let name = ir::XmlName {
        local_name: spanned(local_name),
        namespace: spanned(namespace),
    };

    let root_name = ir::Name::new("root".to_string());
    let element_name = ir::Name::new("element".to_string());

    // create an element with that name in root
    let element_expr = ir::Expr::Element(ir::Element {
        element: spanned(ir::Atom::Variable(root_name.clone())),
        name: spanned(ir::Atom::Variable(element_name.clone())),
    });

    // we need to make sure the name exists
    let let_name = ir::Let {
        name: element_name,
        var_expr: Box::new(spanned(ir::Expr::XmlName(name))),
        return_expr: Box::new(Spanned::new(element_expr, (0..0).into())),
    };

    // we also need to make sure the root exists
    let root_expr = ir::Expr::Root(ir::Root {});

    let let_root = ir::Let {
        name: root_name,
        var_expr: Box::new(spanned(root_expr)),
        return_expr: Box::new(spanned(ir::Expr::Let(let_name))),
    };
    let expr = spanned(ir::Expr::Let(let_root));
    // wrap all of this into a function definition
    let function_definition = ir::FunctionDefinition {
        params: vec![],
        return_type: None,
        body: Box::new(expr),
    };

    let outer_expr = spanned(ir::Expr::FunctionDefinition(function_definition));

    // now that we have the IR, create bytecode
    let mut program = Program::new((0..0).into());
    let function_builder = FunctionBuilder::new(&mut program);
    let mut scopes = Scopes::new();
    let namespaces = xee_interpreter::Namespaces::default();
    let variable_names = xee_interpreter::VariableNames::default();
    let static_context = xee_interpreter::context::StaticContext::new(namespaces, variable_names);
    let mut compiler = InterpreterCompiler::new(function_builder, &mut scopes, &static_context);

    compiler.compile_expr(&outer_expr).unwrap();

    assert_debug_snapshot!(decode_instructions(&program.functions[0].chunk))
}

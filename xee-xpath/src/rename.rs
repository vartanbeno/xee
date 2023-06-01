use ahash::{HashSet, HashSetExt};

use crate::ast;
use crate::static_context::StaticContext;
use crate::visitor::AstVisitor;

struct UniqueNameGenerator {
    names: HashSet<ast::Name>,
}

impl UniqueNameGenerator {
    fn new() -> Self {
        UniqueNameGenerator {
            names: HashSet::new(),
        }
    }

    fn generate(&mut self, name: &ast::Name) -> ast::Name {
        let mut name = name.clone();
        while self.names.contains(&name) {
            name = name.with_suffix();
        }
        self.names.insert(name.clone());
        name
    }
}

struct Names {
    names: Vec<(ast::Name, ast::Name)>,
    generator: UniqueNameGenerator,
}

impl Names {
    fn new() -> Self {
        Names {
            names: Vec::new(),
            generator: UniqueNameGenerator::new(),
        }
    }

    fn get(&mut self, name: &ast::Name) -> ast::Name {
        // this always returns a name, even if the
        // name is unknown, in which case a unique bogus
        // name is generated
        self.names
            .iter()
            .rev()
            .find(|(old_name, _)| old_name == name)
            .map(|(_, new_name)| new_name.clone())
            .unwrap_or_else(|| self.generator.generate(name))
    }

    fn push_name(&mut self, name: &ast::Name) -> ast::Name {
        let new_name = self.generator.generate(name);
        self.names.push((name.clone(), new_name.clone()));
        new_name
    }

    fn pop_name(&mut self) {
        self.names.pop();
    }
}

struct Renamer {
    names: Names,
}

impl Renamer {
    fn new() -> Self {
        Renamer {
            names: Names::new(),
        }
    }

    fn push_name(&mut self, name: &ast::Name) -> ast::Name {
        self.names.push_name(name)
    }

    fn pop_name(&mut self) {
        self.names.pop_name();
    }
}

impl AstVisitor for Renamer {
    fn visit_let_expr(&mut self, expr: &mut ast::LetExpr) {
        self.visit_expr_single(&mut expr.var_expr);
        expr.var_name = self.push_name(&expr.var_name);
        self.visit_expr_single(&mut expr.return_expr);
        self.pop_name();
    }

    fn visit_for_expr(&mut self, expr: &mut ast::ForExpr) {
        self.visit_expr_single(&mut expr.var_expr);
        expr.var_name = self.push_name(&expr.var_name);
        self.visit_expr_single(&mut expr.return_expr);
        self.pop_name();
    }

    fn visit_quantified_expr(&mut self, expr: &mut ast::QuantifiedExpr) {
        self.visit_expr_single(&mut expr.var_expr);
        expr.var_name = self.push_name(&expr.var_name);
        self.visit_expr_single(&mut expr.satisfies_expr);
        self.pop_name();
    }

    fn visit_inline_function(&mut self, expr: &mut ast::InlineFunction) {
        for param in &mut expr.params {
            param.name = self.push_name(&param.name);
        }
        self.visit_expr(&mut expr.body);
        for _ in &expr.params {
            self.pop_name();
        }
    }

    fn visit_var_ref(&mut self, name: &mut ast::Name) {
        let new_name = self.names.get(name);
        *name = new_name;
    }
}

pub(crate) fn unique_names(expr: &mut ast::XPath, static_context: &StaticContext) {
    let mut renamer = Renamer::new();
    // ensure we know of the outer variable names too;
    // these are never going to be changed as there isn't
    // any other shadowing yet at this point
    for name in &static_context.variables {
        renamer.push_name(name);
    }
    renamer.visit_xpath(expr);
}

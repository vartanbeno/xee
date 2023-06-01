use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use crate::ast;
use crate::visitor::{visit, AstVisitor};

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
}

impl Names {
    fn new() -> Self {
        Names { names: Vec::new() }
    }

    fn get(&self, name: &ast::Name) -> Option<&ast::Name> {
        self.names
            .iter()
            .rev()
            .find(|(old_name, _)| old_name == name)
            .map(|(_, new_name)| new_name)
    }

    fn name(&self) -> Option<&ast::Name> {
        self.names.last().map(|(_, new_name)| new_name)
    }

    fn push_name(&mut self, generator: &mut UniqueNameGenerator, name: &ast::Name) {
        let new_name = generator.generate(name);
        self.names.push((name.clone(), new_name));
    }

    fn pop_name(&mut self) {
        self.names.pop();
    }
}

struct Renamer {
    generator: UniqueNameGenerator,
    names: Names,
}

impl Renamer {
    fn push_name(&mut self, name: &ast::Name) {
        self.names.push_name(&mut self.generator, name);
    }

    fn pop_name(&mut self) {
        self.names.pop_name();
    }
}

impl AstVisitor for Renamer {
    fn visit_let_expr(&mut self, expr: &mut ast::LetExpr) {
        self.visit_expr_single(&mut expr.var_expr);
        self.push_name(&expr.var_name);
        self.visit_expr_single(&mut expr.return_expr);
        self.pop_name();
    }
}

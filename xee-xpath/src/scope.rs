use crate::ast;

#[derive(Debug)]
pub(crate) struct Scope {
    names: Vec<ast::Name>,
}

impl Scope {
    fn new() -> Self {
        Self { names: Vec::new() }
    }

    fn get(&self, name: &ast::Name) -> Option<usize> {
        for (i, n) in self.names.iter().enumerate().rev() {
            if n == name {
                return Some(i);
            }
        }
        None
    }

    fn known_name(&self, name: &ast::Name) -> bool {
        self.names.iter().any(|n| n == name)
    }
}

#[derive(Debug)]
pub(crate) struct Scopes {
    scopes: Vec<Scope>,
    dummy: ast::Name,
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            dummy: ast::Name {
                name: "dummy".to_string(),
                namespace: None,
            },
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn push_name(&mut self, name: &ast::Name) {
        self.scopes.last_mut().unwrap().names.push(name.clone());
    }

    pub(crate) fn push_dummy(&mut self) {
        self.push_name(&self.dummy.clone());
    }

    pub(crate) fn pop_name(&mut self) {
        self.scopes.last_mut().unwrap().names.pop();
    }

    pub(crate) fn pop_dummy(&mut self) {
        self.pop_name();
    }

    pub(crate) fn get(&self, name: &ast::Name) -> Option<usize> {
        self.scopes.last().unwrap().get(name)
    }

    pub(crate) fn is_closed_over_name(&self, name: &ast::Name) -> bool {
        let mut scopes = self.scopes.iter();
        scopes.next();
        scopes.any(|s| s.known_name(name))
    }

    pub(crate) fn count(&self) -> usize {
        let mut count = 0;
        for scope in &self.scopes {
            count += scope.names.len();
        }
        count
    }
}

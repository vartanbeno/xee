#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackEntry<'a> {
    Integer(i64),
    StringRef(&'a str),
    OwnedString(String),
}

impl<'a> StackEntry<'a> {
    pub(crate) fn as_integer(&self) -> i64 {
        match self {
            StackEntry::Integer(i) => *i,
            _ => {
                panic!("not an integer");
            }
        }
    }
    pub(crate) fn as_string(&'a self) -> &'a str {
        match self {
            StackEntry::StringRef(s) => s,
            StackEntry::OwnedString(s) => s.as_str(),
            _ => {
                panic!("not a string");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Operation<'a> {
    Add,
    Sub,
    Concat,
    IntegerLiteral(i64),
    StringLiteral(&'a str),
}

pub(crate) struct Interpreter<'a> {
    pub(crate) stack: Vec<StackEntry<'a>>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub(crate) fn interpret(&mut self, operations: &'a [Operation]) {
        for operation in operations {
            match operation {
                Operation::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack
                        .push(StackEntry::Integer(a.as_integer() + b.as_integer()));
                }
                Operation::Sub => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack
                        .push(StackEntry::Integer(a.as_integer() - b.as_integer()));
                }
                Operation::Concat => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let a = a.as_string();
                    let b = b.as_string();
                    let c = format!("{}{}", b, a);
                    self.stack.push(StackEntry::OwnedString(c));
                }
                Operation::IntegerLiteral(i) => {
                    self.stack.push(StackEntry::Integer(*i));
                }
                Operation::StringLiteral(s) => {
                    self.stack.push(StackEntry::StringRef(s));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        let mut interpreter = Interpreter::new();
        interpreter.stack.push(StackEntry::Integer(1));
        interpreter.stack.push(StackEntry::Integer(2));
        interpreter.interpret(&[Operation::Add]);
        assert_eq!(interpreter.stack.pop().unwrap().as_integer(), 3);
    }
}

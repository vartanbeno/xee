#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackEntry {
    Integer(i64),
}

impl StackEntry {
    pub(crate) fn as_integer(&self) -> i64 {
        match self {
            StackEntry::Integer(i) => *i,
            _ => {
                panic!("not an integer");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Operation {
    Add,
    Sub,
    IntegerLiteral(i64),
}

pub(crate) struct Interpreter {
    pub(crate) stack: Vec<StackEntry>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub(crate) fn interpret(&mut self, operations: &[Operation]) {
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
                Operation::IntegerLiteral(i) => {
                    self.stack.push(StackEntry::Integer(*i));
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

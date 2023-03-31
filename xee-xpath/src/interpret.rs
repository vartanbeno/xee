enum StackEntry {
    Integer(i64),
}

impl StackEntry {
    pub fn as_integer(&self) -> i64 {
        match self {
            StackEntry::Integer(i) => *i,
            _ => {
                panic!("not an integer");
            }
        }
    }
}

enum Operation {
    Add,
    Sub,
}

struct Interpreter {
    stack: Vec<StackEntry>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn interpret(&mut self, operations: &[Operation]) {
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

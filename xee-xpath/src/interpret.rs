use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("integer overflow")]
    IntegerOverflow,
    #[error("type error")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackEntry<'a> {
    Integer(i64),
    StringRef(&'a str),
    OwnedString(String),
}

impl<'a> StackEntry<'a> {
    pub(crate) fn as_integer(&self) -> Result<i64> {
        match self {
            StackEntry::Integer(i) => Ok(*i),
            _ => Err(Error::TypeError),
        }
    }
    pub(crate) fn as_string(&'a self) -> Result<&'a str> {
        match self {
            StackEntry::StringRef(s) => Ok(s),
            StackEntry::OwnedString(s) => Ok(s.as_str()),
            _ => Err(Error::TypeError),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Operation<'a> {
    Add,
    Sub,
    Mul,
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

    pub(crate) fn interpret(&mut self, operations: &'a [Operation]) -> Result<()> {
        for operation in operations {
            match operation {
                Operation::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a + b));
                }
                Operation::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a - b));
                }
                Operation::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a * b));
                }
                Operation::Concat => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_string()?;
                    let b = b.as_string()?;
                    let c = format!("{}{}", a, b);
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() -> Result<()> {
        let mut interpreter = Interpreter::new();
        interpreter.stack.push(StackEntry::Integer(1));
        interpreter.stack.push(StackEntry::Integer(2));
        interpreter.interpret(&[Operation::Add]).unwrap();
        assert_eq!(interpreter.stack.pop().unwrap().as_integer()?, 3);
        Ok(())
    }
}

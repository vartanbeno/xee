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
pub(crate) enum Atomic {
    Integer(i64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    AtomicValue(Atomic),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence(pub Vec<Item>);

impl Sequence {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn combine(&self, other: &Sequence) -> Sequence {
        // XXX should not need to clone contents of sequences as they are
        // immutable, only reference
        let mut r = self.0.clone();
        r.extend(other.0.clone());
        Sequence(r)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackEntry {
    Integer(i64),
    String(String),
    Sequence(Sequence),
}

impl StackEntry {
    pub(crate) fn as_integer(&self) -> Result<i64> {
        match self {
            StackEntry::Integer(i) => Ok(*i),
            _ => Err(Error::TypeError),
        }
    }
    pub(crate) fn as_string(&self) -> Result<&str> {
        match self {
            StackEntry::String(s) => Ok(s.as_str()),
            _ => Err(Error::TypeError),
        }
    }

    pub(crate) fn as_sequence(&self) -> Result<Sequence> {
        match self {
            StackEntry::Sequence(s) => Ok(s.clone()),
            StackEntry::Integer(i) => Ok(Sequence(vec![Item::AtomicValue(Atomic::Integer(*i))])),
            StackEntry::String(s) => {
                Ok(Sequence(vec![Item::AtomicValue(Atomic::String(s.clone()))]))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Operation {
    Add,
    Sub,
    Mul,
    Concat,
    IntegerLiteral(i64),
    StringLiteral(String),
    Comma,
}

pub(crate) struct Interpreter {
    pub(crate) stack: Vec<StackEntry>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub(crate) fn interpret(&mut self, operations: &[Operation]) -> Result<()> {
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
                    self.stack.push(StackEntry::String(c));
                }
                Operation::IntegerLiteral(i) => {
                    self.stack.push(StackEntry::Integer(*i));
                }
                Operation::StringLiteral(s) => {
                    self.stack.push(StackEntry::String(s.to_string()));
                }
                Operation::Comma => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let a = a.as_sequence()?;
                    let b = b.as_sequence()?;
                    self.stack.push(StackEntry::Sequence(a.combine(&b)));
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

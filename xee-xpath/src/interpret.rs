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
    // we could make these references to a pool, so that the stack entry
    // is really cheap to clone
    String(String),
    Sequence(Sequence),
    // StackRef(usize),
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
            _ => Err(Error::TypeError),
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
    LetDone,
    VarRef(usize),
}

pub(crate) struct Interpreter {
    pub(crate) stack: Vec<StackEntry>,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> StackEntry {
        self.stack.pop().unwrap()
        // if let StackEntry::StackRef(index) = entry {
        //     // XXX this isn't the cheapest
        //     self.stack[index].clone()
        // } else {
        //     entry
        // }
    }

    pub(crate) fn interpret(&mut self, operations: &[Operation]) -> Result<()> {
        for operation in operations {
            match operation {
                Operation::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a + b));
                }
                Operation::Sub => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a - b));
                }
                Operation::Mul => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack.push(StackEntry::Integer(a * b));
                }
                Operation::Concat => {
                    let b = self.pop();
                    let a = self.pop();
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
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_sequence()?;
                    let b = b.as_sequence()?;
                    self.stack.push(StackEntry::Sequence(a.combine(&b)));
                }
                Operation::LetDone => {
                    let b = self.pop();
                    // pop the variable assignment
                    let _ = self.pop();
                    self.stack.push(b);
                }
                Operation::VarRef(index) => {
                    // XXX annoying that we have to clone here
                    // We could avoid this by having a StackRef variant
                    // but that would require a clone when we pop
                    self.stack.push(self.stack[*index].clone());
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

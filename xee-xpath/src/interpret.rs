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

    pub(crate) fn push(&mut self, item: Item) {
        self.0.push(item);
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

    pub(crate) fn as_bool(&self) -> Result<bool> {
        match self {
            StackEntry::Integer(i) => Ok(*i != 0),
            StackEntry::String(s) => Ok(!s.is_empty()),
            _ => Err(Error::TypeError),
        }
    }

    pub(crate) fn as_item(&self) -> Result<Item> {
        match self {
            StackEntry::Integer(i) => Ok(Item::AtomicValue(Atomic::Integer(*i))),
            StackEntry::String(s) => Ok(Item::AtomicValue(Atomic::String(s.clone()))),
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
    Peek(usize),
    Pop,
    Dup,
    Add,
    Sub,
    Mul,
    Concat,
    // XXX we could reduce the amount of value compare opcodes by clever compilation
    ValueEq,
    ValueNe,
    ValueLt,
    ValueLe,
    ValueGt,
    ValueGe,
    IntegerLiteral(i64),
    StringLiteral(String),
    Comma,
    LetDone,
    VarRef(usize),
    Jump(usize),
    JumpIfFalse(usize),
    SequenceNew,
    SequencePush(usize),
    SequenceGet(usize),
    SequenceLen(usize),
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

    #[inline]
    pub(crate) fn top(&self) -> &StackEntry {
        &self.stack[self.stack.len() - 1]
    }

    pub(crate) fn interpret(&mut self, operations: &[Operation]) -> Result<()> {
        let mut ip = 0;

        while ip < operations.len() {
            let operation = &operations[ip];
            match operation {
                Operation::Peek(usize) => {
                    let entry = self.stack[self.stack.len() - usize - 1].clone();
                    self.stack.push(entry);
                }
                Operation::Pop => {
                    self.pop();
                }
                Operation::Dup => {
                    let entry = self.top().clone();
                    self.stack.push(entry);
                }
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
                Operation::ValueEq => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a == b { 1 } else { 0 }));
                }
                Operation::ValueNe => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a != b { 1 } else { 0 }));
                }
                Operation::ValueLt => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a < b { 1 } else { 0 }));
                }
                Operation::ValueLe => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a <= b { 1 } else { 0 }));
                }
                Operation::ValueGt => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a > b { 1 } else { 0 }));
                }
                Operation::ValueGe => {
                    let b = self.pop();
                    let a = self.pop();
                    let a = a.as_integer()?;
                    let b = b.as_integer()?;
                    self.stack
                        .push(StackEntry::Integer(if a >= b { 1 } else { 0 }));
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
                    // better to make cloning cheap, which we can
                    // as data structures are immutable
                    self.stack.push(self.stack[*index].clone());
                }
                Operation::Jump(new_ip) => {
                    ip = *new_ip;
                    continue;
                }
                Operation::JumpIfFalse(new_ip) => {
                    let a = self.pop();
                    let a = a.as_bool()?;
                    if !a {
                        ip = *new_ip;
                        continue;
                    }
                }
                Operation::SequenceNew => {
                    self.stack.push(StackEntry::Sequence(Sequence(Vec::new())));
                }
                Operation::SequencePush(index) => {
                    let a = self.pop();
                    let seq = &self.stack[*index];

                    let mut seq = seq.as_sequence()?;
                    if let StackEntry::Sequence(a) = a {
                        for item in &a.0 {
                            seq.push(item.clone());
                        }
                    } else {
                        seq.push(a.as_item()?);
                    }
                }
                Operation::SequenceGet(index) => {
                    let a = self.pop();
                    let seq = &self.stack[*index];
                    let seq = seq.as_sequence()?;

                    let a = a.as_integer()?;
                    let item = seq.0[a as usize].clone();
                    match item {
                        Item::AtomicValue(Atomic::Integer(i)) => {
                            self.stack.push(StackEntry::Integer(i));
                        }
                        Item::AtomicValue(Atomic::String(s)) => {
                            self.stack.push(StackEntry::String(s));
                        }
                    }
                }
                Operation::SequenceLen(index) => {
                    let seq = &self.stack[*index];
                    let seq = seq.as_sequence()?;
                    self.stack.push(StackEntry::Integer(seq.0.len() as i64));
                }
            }
            ip += 1;
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

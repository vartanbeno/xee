use std::rc::Rc;

use ahash::HashMap;
use ahash::HashMapExt;
use miette::SourceSpan;
use xee_schema_type::Xs;
use xee_xpath_ast::ast;
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::ir;
use crate::sequence;
use crate::stack;
use crate::xml;
use crate::Collation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct InlineFunctionId(pub(crate) usize);

impl InlineFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CastType {
    pub(crate) xs: Xs,
    pub(crate) empty_sequence_allowed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct InlineFunction {
    pub(crate) name: String,
    pub(crate) params: Vec<ir::Param>,
    // things referenced by instructions (by index)
    pub(crate) constants: Vec<stack::Value>,
    pub(crate) steps: Vec<xml::Step>,
    pub(crate) cast_types: Vec<CastType>,
    pub(crate) sequence_types: Vec<ast::SequenceType>,
    pub(crate) closure_names: Vec<ir::Name>,
    // the compiled code, and the spans of each instruction
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub(crate) parameter_types: Vec<ast::SequenceType>,
    pub(crate) return_type: ast::SequenceType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array(pub Rc<Vec<sequence::Sequence>>);

impl Array {
    pub(crate) fn new(vec: Vec<sequence::Sequence>) -> Self {
        Self(Rc::new(vec))
    }

    pub(crate) fn join(arrays: &[Self]) -> Self {
        let mut vec = Vec::new();
        for array in arrays {
            vec.extend(array.0.as_ref().iter().cloned());
        }
        Self::new(vec)
    }

    pub(crate) fn index(&self, index: usize) -> Option<&sequence::Sequence> {
        self.0.get(index)
    }

    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &sequence::Sequence> {
        self.0.iter()
    }

    pub(crate) fn push(&mut self, member: sequence::Sequence) {
        Rc::make_mut(&mut self.0).push(member);
    }

    pub(crate) fn put(&self, index: usize, member: &sequence::Sequence) -> Option<Self> {
        if index >= self.0.len() {
            return None;
        }
        let mut vec = self.0.as_ref().clone();
        vec[index] = member.clone();
        Some(Self::new(vec))
    }

    pub(crate) fn append(&self, appendage: &sequence::Sequence) -> Self {
        let mut vec = self.0.as_ref().clone();
        vec.push(appendage.clone());
        Self::new(vec)
    }

    pub(crate) fn subarray(&self, start: usize, length: usize) -> Option<Self> {
        if start > self.0.len() || (start + length) > self.0.len() {
            return None;
        }
        let mut vec = Vec::with_capacity(length);
        for i in start..(start + length) {
            vec.push(self.0[i].clone());
        }
        Some(Self::new(vec))
    }

    pub(crate) fn remove_positions(&self, positions: &[usize]) -> Option<Self> {
        for position in positions {
            if position >= &self.0.len() {
                return None;
            }
        }
        let mut vec = Vec::with_capacity(self.0.len() - positions.len());

        for (i, member) in self.0.iter().enumerate() {
            if !positions.contains(&i) {
                vec.push(member.clone());
            }
        }
        Some(Self::new(vec))
    }

    pub(crate) fn reversed(&self) -> Self {
        let mut vec = self.0.as_ref().clone();
        vec.reverse();
        Self::new(vec)
    }

    pub(crate) fn insert_before(
        &self,
        position: usize,
        member: &sequence::Sequence,
    ) -> Option<Self> {
        if position > self.0.len() {
            return None;
        }
        let mut vec = self.0.as_ref().clone();
        vec.insert(position, member.clone());
        Some(Self::new(vec))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn deep_equal(
        &self,
        other: Array,
        collation: &Collation,
        default_offset: chrono::FixedOffset,
        xot: &Xot,
    ) -> error::Result<bool> {
        if self.0.len() != other.0.len() {
            return Ok(false);
        }
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if !a.deep_equal(b, collation, default_offset, xot)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl From<Vec<sequence::Sequence>> for Array {
    fn from(vec: Vec<sequence::Sequence>) -> Self {
        Self::new(vec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map(pub Rc<HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)>>);

impl Map {
    pub(crate) fn new(entries: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
        let mut map = HashMap::new();
        for (key, value) in entries {
            let map_key = atomic::MapKey::new(key.clone())?;
            if map.contains_key(&map_key) {
                return Err(error::Error::XQDY0137);
            }
            map.insert(map_key, (key, value));
        }
        Ok(Self(Rc::new(map)))
    }

    pub(crate) fn get(&self, key: &atomic::Atomic) -> Option<sequence::Sequence> {
        let map_key = atomic::MapKey::new(key.clone()).ok()?;
        self.0.get(&map_key).map(|(_, v)| v.clone())
    }

    pub(crate) fn deep_equal(
        &self,
        other: Map,
        collation: &Collation,
        default_offset: chrono::FixedOffset,
        xot: &Xot,
    ) -> error::Result<bool> {
        if self.0.len() != other.0.len() {
            return Ok(false);
        }
        for (map_key, (_real_key, value)) in self.0.iter() {
            let entry = other.0.get(map_key);
            if let Some((_real_key, found_value)) = entry {
                if !value.deep_equal(found_value, collation, default_offset, xot)? {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl TryFrom<Vec<(atomic::Atomic, sequence::Sequence)>> for Map {
    type Error = error::Error;
    fn try_from(vec: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
        Self::new(vec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Closure {
    Static {
        static_function_id: StaticFunctionId,
        sequences: Vec<sequence::Sequence>,
    },
    Inline {
        inline_function_id: InlineFunctionId,
        sequences: Vec<sequence::Sequence>,
    },
    Map(Map),
    Array(Array),
}

impl Closure {
    pub(crate) fn sequences(&self) -> &[sequence::Sequence] {
        match self {
            Self::Static { sequences, .. } => sequences,
            Self::Inline { sequences, .. } => sequences,
            _ => unreachable!(),
        }
    }
}

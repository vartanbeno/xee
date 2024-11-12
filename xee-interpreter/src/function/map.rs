use std::rc::Rc;

use ahash::{HashMap, HashMapExt};
use xee_schema_type::Xs;
use xee_xpath_ast::ast;
use xot::Xot;

use crate::{atomic, context, error, sequence, string};

/// An XPath Map (a collection of key-value pairs).
#[derive(Debug, Clone, PartialEq)]
pub struct Map(pub(crate) Rc<HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)>>);

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

    pub(crate) fn from_map(
        map: HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)>,
    ) -> Self {
        Self(Rc::new(map))
    }

    pub(crate) fn get(&self, key: &atomic::Atomic) -> Option<sequence::Sequence> {
        let map_key = atomic::MapKey::new(key.clone()).ok()?;
        self.0.get(&map_key).map(|(_, v)| v.clone())
    }

    // this is to support the option parameter conventions
    pub(crate) fn get_as_type(
        &self,
        key: &atomic::Atomic,
        occurrence: ast::Occurrence,
        atomic_type: Xs,
        static_context: &context::StaticContext,
        xot: &Xot,
    ) -> error::Result<Option<sequence::Sequence>> {
        let value = self.get(key);
        let value = match value {
            Some(value) => value,
            None => return Ok(None),
        };
        let sequence_type = ast::SequenceType::Item(ast::Item {
            occurrence,
            item_type: ast::ItemType::AtomicOrUnionType(atomic_type),
        });
        Ok(Some(value.sequence_type_matching_function_conversion(
            &sequence_type,
            static_context,
            xot,
            // typed function tests can't be invoked
            &|_function| unreachable!(),
        )?))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = atomic::Atomic> + '_ {
        self.0.values().map(|(k, _)| k.clone())
    }

    pub(crate) fn put(&self, key: atomic::Atomic, value: &sequence::Sequence) -> Self {
        let mut map = self.0.as_ref().clone();
        let map_key = atomic::MapKey::new(key.clone()).unwrap();
        map.insert(map_key, (key, value.clone()));
        Self(Rc::new(map))
    }

    pub(crate) fn remove_keys(&self, keys: &[atomic::Atomic]) -> Self {
        let mut map = self.0.as_ref().clone();
        for key in keys {
            let map_key = atomic::MapKey::new(key.clone()).unwrap();
            map.remove(&map_key);
        }
        Self(Rc::new(map))
    }

    pub(crate) fn deep_equal(
        &self,
        other: Map,
        collation: &string::Collation,
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

    pub fn display_representation(&self, xot: &Xot) -> String {
        let mut entries = self
            .0
            .iter()
            .map(|(k, (key, value))| {
                format!(
                    "{}: {}",
                    key.xpath_representation(),
                    value.display_representation(xot)
                )
            })
            .collect::<Vec<_>>();
        entries.sort();
        format!("map {{\n{}\n}}", entries.join(",\n"))
    }
}

impl TryFrom<Vec<(atomic::Atomic, sequence::Sequence)>> for Map {
    type Error = error::Error;
    fn try_from(vec: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
        Self::new(vec)
    }
}

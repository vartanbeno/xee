use std::rc::Rc;

use ahash::{HashMap, HashMapExt};
use xee_schema_type::Xs;
use xee_xpath_ast::ast;
use xot::Xot;

use crate::{atomic, context, error, sequence, string};

/// An XPath Map (a collection of key-value pairs).
#[derive(Debug, Clone, PartialEq)]
pub enum Map {
    Many(ManyMap),
}

impl Map {
    pub(crate) fn new(entries: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
        Ok(Self::Many(ManyMap::try_from(entries)?))
    }

    fn from_map(map: HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)>) -> Self {
        Self::Many(ManyMap(Rc::new(map)))
    }

    pub(crate) fn combine(
        maps: &[Map],
        combine: impl Fn(sequence::Sequence, sequence::Sequence) -> error::Result<sequence::Sequence>,
    ) -> error::Result<Map> {
        let mut result = HashMap::new();
        for map in maps {
            for (map_key, (key, value)) in map.full_entries() {
                let map_key = map_key.clone();
                let entry = result.remove(&map_key);
                let value = if let Some((_, a)) = entry {
                    combine(a, value.clone())?
                } else {
                    value.clone()
                };
                result.insert(map_key, (key.clone(), value));
            }
        }
        Ok(Map::from_map(result))
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Map::Many(map) => map.len(),
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Map::Many(map) => map.is_empty(),
        }
    }
    pub(crate) fn get(&self, key: &atomic::Atomic) -> Option<&sequence::Sequence> {
        match self {
            Map::Many(map) => map.get(key),
        }
    }
    pub(crate) fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_ {
        match self {
            Map::Many(map) => map.keys(),
        }
    }
    pub(crate) fn entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence)> + '_ {
        match self {
            Map::Many(map) => map.entries(),
        }
    }

    pub(crate) fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_ {
        match self {
            Map::Many(map) => map.map_keys(),
        }
    }

    pub(crate) fn map_key_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence)> + '_ {
        match self {
            Map::Many(map) => map.map_key_entries(),
        }
    }

    pub(crate) fn full_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &(atomic::Atomic, sequence::Sequence))> + '_ {
        match self {
            Map::Many(map) => map.full_entries(),
        }
    }

    pub(crate) fn get_as_type(
        &self,
        key: &atomic::Atomic,
        occurrence: ast::Occurrence,
        atomic_type: Xs,
        static_context: &context::StaticContext,
        xot: &Xot,
    ) -> error::Result<Option<sequence::Sequence>> {
        match self {
            Map::Many(map) => map.get_as_type(key, occurrence, atomic_type, static_context, xot),
        }
    }

    pub(crate) fn deep_equal(
        &self,
        other: &Map,
        collation: &string::Collation,
        default_offset: chrono::FixedOffset,
        xot: &Xot,
    ) -> error::Result<bool> {
        match (self, other) {
            (Map::Many(map), Map::Many(other)) => {
                map.deep_equal(other, collation, default_offset, xot)
            }
        }
    }

    pub fn display_representation(&self, xot: &Xot, context: &context::DynamicContext) -> String {
        match self {
            Map::Many(map) => map.display_representation(xot, context),
        }
    }

    pub(crate) fn put(&self, key: atomic::Atomic, value: &sequence::Sequence) -> Self {
        match self {
            // since at most we add keys, this cannot turn into a non-many map
            Map::Many(map) => Map::Many(map.put(key, value)),
        }
    }

    pub(crate) fn remove_keys(&self, keys: &[atomic::Atomic]) -> Self {
        match self {
            Map::Many(map) => Map::from_map(map.remove_keys(keys)),
        }
    }
}

pub(crate) trait Mappable {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    // get a key by underlying map key
    fn get_by_map_key(&self, map_key: &atomic::MapKey) -> Option<&sequence::Sequence>;

    /// get map keys
    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_;

    // get map entries, key is map key
    fn map_key_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence)> + '_;

    fn full_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &(atomic::Atomic, sequence::Sequence))> + '_;

    // get a key by atomic
    fn get(&self, key: &atomic::Atomic) -> Option<&sequence::Sequence> {
        let map_key = atomic::MapKey::new(key.clone()).ok()?;
        self.get_by_map_key(&map_key)
    }

    // get atomic keys
    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_;

    // get entries with atomic key and value
    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence)> + '_;

    // get a key by atomic and convert to a specific type
    fn get_as_type(
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
        // TODO: is value clone really needed?
        Ok(Some(
            value.clone().sequence_type_matching_function_conversion(
                &sequence_type,
                static_context,
                xot,
                // typed function tests can't be invoked
                &|_function| unreachable!(),
            )?,
        ))
    }

    // deep equal compare between two mappables
    fn deep_equal(
        &self,
        other: &impl Mappable,
        collation: &string::Collation,
        default_offset: chrono::FixedOffset,
        xot: &Xot,
    ) -> error::Result<bool> {
        if self.len() != other.len() {
            return Ok(false);
        }
        for (map_key, value) in self.map_key_entries() {
            let other_value = other.get_by_map_key(map_key);
            if let Some(other_value) = other_value {
                if !value.deep_equal(other_value, collation, default_offset, xot)? {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn display_representation(&self, xot: &Xot, context: &context::DynamicContext) -> String {
        let mut entries = self
            .entries()
            .map(|(key, value)| {
                format!(
                    "{}: {}",
                    key.xpath_representation(),
                    value.display_representation(xot, context)
                )
            })
            .collect::<Vec<_>>();
        entries.sort();
        format!("map {{\n{}\n}}", entries.join(",\n"))
    }
}

// empty map
#[derive(Debug, Clone, PartialEq)]
pub struct EmptyMap;

// a normal map uses a hashmap to store > 1 key-value pairs
#[derive(Debug, Clone, PartialEq)]
pub struct ManyMap(Rc<HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)>>);

impl ManyMap {
    fn new(entries: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
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

    pub(crate) fn put(&self, key: atomic::Atomic, value: &sequence::Sequence) -> Self {
        let mut map = self.0.as_ref().clone();
        let map_key = atomic::MapKey::new(key.clone()).unwrap();
        map.insert(map_key, (key, value.clone()));
        Self(Rc::new(map))
    }

    pub(crate) fn remove_keys(
        &self,
        keys: &[atomic::Atomic],
    ) -> HashMap<atomic::MapKey, (atomic::Atomic, sequence::Sequence)> {
        let mut map = self.0.as_ref().clone();
        for key in keys {
            let map_key = atomic::MapKey::new(key.clone()).unwrap();
            map.remove(&map_key);
        }
        map
    }
}

impl Mappable for ManyMap {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn get_by_map_key(&self, map_key: &atomic::MapKey) -> Option<&sequence::Sequence> {
        self.0.get(map_key).map(|(_, v)| v)
    }

    fn map_keys(&self) -> impl Iterator<Item = &'_ atomic::MapKey> + '_ {
        self.0.keys()
    }

    fn map_key_entries(&self) -> impl Iterator<Item = (&atomic::MapKey, &sequence::Sequence)> + '_ {
        self.0.iter().map(|(k, (_, v))| (k, v))
    }

    fn full_entries(
        &self,
    ) -> impl Iterator<Item = (&atomic::MapKey, &(atomic::Atomic, sequence::Sequence))> + '_ {
        self.0.iter()
    }

    fn keys(&self) -> impl Iterator<Item = &atomic::Atomic> + '_ {
        self.0.values().map(|(k, _)| k)
    }

    fn entries(&self) -> impl Iterator<Item = (&atomic::Atomic, &sequence::Sequence)> + '_ {
        self.0.iter().map(|(_, (k, v))| (k, v))
    }
}

impl TryFrom<Vec<(atomic::Atomic, sequence::Sequence)>> for ManyMap {
    type Error = error::Error;
    fn try_from(vec: Vec<(atomic::Atomic, sequence::Sequence)>) -> error::Result<Self> {
        Self::new(vec)
    }
}

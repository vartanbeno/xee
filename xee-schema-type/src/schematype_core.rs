use ahash::{HashMap, HashMapExt};
use std::rc::Rc;

const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug, Clone, PartialEq)]
struct SchemaType {
    namespace: String,
    local_name: String,
    parent_type: Option<Rc<SchemaType>>,
    schema_type_category: SchemaTypeCategory,
}

impl SchemaType {
    pub fn new(
        namespace: &str,
        local_name: &str,
        parent_type: Option<Rc<SchemaType>>,
        schema_type_category: SchemaTypeCategory,
    ) -> Rc<Self> {
        Rc::new(Self {
            namespace: namespace.to_string(),
            local_name: local_name.to_string(),
            parent_type,
            schema_type_category,
        })
    }

    pub fn derives_from(&self, other: &SchemaType) -> bool {
        if self == other {
            return true;
        }
        match &self.parent_type {
            // TODO: handle union type
            Some(parent_type) => parent_type.derives_from(other),
            None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SchemaTypeCategory {
    // TODO: not really very clear to define a category as 'other'
    Other,
    // A generalized atomic type is either atomic or pure union
    // TODO: should this be formalized in the Rust type system?
    AbstractAtomic,
    Atomic(RustInfo),
    // TODO should guarantee that only pure union types can be
    // constructed
    // https://www.w3.org/TR/xpath-31/#id-types
    Union(Vec<SchemaType>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustInfo {
    rust_name: String,
    as_ref: bool,
}

impl RustInfo {
    fn new(rust_name: &str) -> Self {
        Self {
            rust_name: rust_name.to_string(),
            as_ref: false,
        }
    }

    fn as_ref(rust_name: &str) -> Self {
        Self {
            rust_name: rust_name.to_string(),
            as_ref: true,
        }
    }
}

struct SchemaTypeRegistry {
    prefixes: HashMap<String, String>,
    types: HashMap<(String, String), Rc<SchemaType>>,
}

struct Error {}

impl SchemaTypeRegistry {
    pub fn new() -> Self {
        let prefixes = HashMap::from_iter(vec![("xs".to_string(), XS_NAMESPACE.to_string())]);

        // xs:anyType
        let xs_any_type = SchemaType::new(XS_NAMESPACE, "anyType", None, SchemaTypeCategory::Other);
        // xs:anySimpleType
        let xs_any_simple_type = SchemaType::new(
            XS_NAMESPACE,
            "anySimpleType",
            Some(xs_any_type.clone()),
            SchemaTypeCategory::Other,
        );
        // xs:anyAtomicType
        let xs_any_atomic_type = SchemaType::new(
            XS_NAMESPACE,
            "anyAtomicType",
            Some(xs_any_simple_type.clone()),
            SchemaTypeCategory::Other,
        );
        // xs:untyped
        let xs_untyped = SchemaType::new(
            XS_NAMESPACE,
            "untyped",
            Some(xs_any_type.clone()),
            SchemaTypeCategory::Other,
        );
        // xs:untypedAtomic
        let xs_untyped_atomic = SchemaType::new(
            XS_NAMESPACE,
            "untypedAtomic",
            Some(xs_any_atomic_type.clone()),
            SchemaTypeCategory::Atomic(RustInfo::as_ref("String")),
        );
        // xs:decimal
        let xs_decimal = SchemaType::new(
            XS_NAMESPACE,
            "decimal",
            Some(xs_untyped_atomic.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("rust_decimal::Decimal")),
        );
        let xs_float = SchemaType::new(
            XS_NAMESPACE,
            "float",
            Some(xs_untyped_atomic.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("f32")),
        );
        let xs_double = SchemaType::new(
            XS_NAMESPACE,
            "double",
            Some(xs_untyped_atomic.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("f64")),
        );
        let xs_boolean = SchemaType::new(
            XS_NAMESPACE,
            "boolean",
            Some(xs_untyped_atomic.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("bool")),
        );
        let xs_string = SchemaType::new(
            XS_NAMESPACE,
            "string",
            Some(xs_untyped_atomic.clone()),
            SchemaTypeCategory::Atomic(RustInfo::as_ref("String")),
        );
        let xs_integer = SchemaType::new(
            XS_NAMESPACE,
            "integer",
            Some(xs_decimal.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("i64")),
        );
        let xs_non_positive_integer = SchemaType::new(
            XS_NAMESPACE,
            "nonPositiveInteger",
            Some(xs_integer.clone()),
            SchemaTypeCategory::AbstractAtomic,
        );
        let xs_negative_integer = SchemaType::new(
            XS_NAMESPACE,
            "negativeInteger",
            Some(xs_non_positive_integer.clone()),
            SchemaTypeCategory::AbstractAtomic,
        );
        let xs_long = SchemaType::new(
            XS_NAMESPACE,
            "long",
            Some(xs_integer.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("i64")),
        );
        let xs_int = SchemaType::new(
            XS_NAMESPACE,
            "int",
            Some(xs_long.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("i32")),
        );
        let xs_short = SchemaType::new(
            XS_NAMESPACE,
            "short",
            Some(xs_int.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("i16")),
        );
        let xs_byte = SchemaType::new(
            XS_NAMESPACE,
            "byte",
            Some(xs_short.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("i8")),
        );
        let xs_non_negative_integer = SchemaType::new(
            XS_NAMESPACE,
            "nonNegativeInteger",
            Some(xs_integer.clone()),
            SchemaTypeCategory::AbstractAtomic,
        );
        let xs_unsigned_long = SchemaType::new(
            XS_NAMESPACE,
            "unsignedLong",
            Some(xs_non_negative_integer.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("u64")),
        );
        let xs_unsigned_int = SchemaType::new(
            XS_NAMESPACE,
            "unsignedInt",
            Some(xs_unsigned_long.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("u32")),
        );
        let xs_unsigned_short = SchemaType::new(
            XS_NAMESPACE,
            "unsignedShort",
            Some(xs_unsigned_int.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("u16")),
        );
        let xs_unsigned_byte = SchemaType::new(
            XS_NAMESPACE,
            "unsignedByte",
            Some(xs_unsigned_short.clone()),
            SchemaTypeCategory::Atomic(RustInfo::new("u8")),
        );
        let xs_positive_integer = SchemaType::new(
            XS_NAMESPACE,
            "positiveInteger",
            Some(xs_non_negative_integer.clone()),
            SchemaTypeCategory::AbstractAtomic,
        );

        let types = vec![
            xs_any_type,
            xs_any_simple_type,
            xs_any_atomic_type,
            xs_untyped,
            xs_untyped_atomic,
            xs_decimal,
            xs_float,
            xs_double,
            xs_boolean,
            xs_string,
            xs_integer,
            xs_non_positive_integer,
            xs_negative_integer,
            xs_long,
            xs_int,
            xs_short,
            xs_byte,
            xs_non_negative_integer,
            xs_unsigned_long,
            xs_unsigned_int,
            xs_unsigned_short,
            xs_unsigned_byte,
            xs_positive_integer,
        ];

        let types = types
            .into_iter()
            .map(|ty| ((ty.namespace.clone(), ty.local_name.clone()), ty))
            .collect();
        Self { prefixes, types }
    }

    pub fn register_prefix(&mut self, prefix: &str, uri: &str) {
        self.prefixes.insert(prefix.to_string(), uri.to_string());
    }

    pub fn register_type(&mut self, uri: &str, name: &str, ty: SchemaType) {
        self.types
            .insert((uri.to_string(), name.to_string()), Rc::new(ty));
    }

    pub fn register_type_with_prefix(
        &mut self,
        fullname: &str,
        ty: SchemaType,
    ) -> Result<(), Error> {
        let (uri, name) = self.parse_prefixed_name(fullname).ok_or(Error {})?;
        self.register_type(&uri, &name, ty);
        Ok(())
    }

    pub fn lookup(&self, uri: &str, name: &str) -> Option<Rc<SchemaType>> {
        self.types
            .get(&(uri.to_string(), name.to_string()))
            .map(|t| Rc::clone(t))
    }

    pub fn lookup_with_prefix(&self, fullname: &str) -> Option<Rc<SchemaType>> {
        let (uri, name) = self.parse_prefixed_name(fullname)?;
        self.lookup(&uri, &name)
    }

    fn parse_prefixed_name(&self, fullname: &str) -> Option<(String, String)> {
        let mut parts = fullname.split(':');
        let prefix = parts.next()?;
        let name = parts.next()?;
        let uri = self.prefixes.get(prefix)?;
        Some((uri.to_string(), name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derives_from() {
        let registry = SchemaTypeRegistry::new();

        let xs_integer = registry.lookup_with_prefix("xs:integer").unwrap();
        let xs_decimal = registry.lookup_with_prefix("xs:decimal").unwrap();
        let xs_any_atomic_type = registry.lookup_with_prefix("xs:anyAtomicType").unwrap();
        let xs_any_simple_type = registry.lookup_with_prefix("xs:anySimpleType").unwrap();
        let xs_any_type = registry.lookup_with_prefix("xs:anyType").unwrap();
        assert!(xs_integer.derives_from(&xs_decimal));
        assert!(xs_integer.derives_from(&xs_any_atomic_type));
        assert!(xs_integer.derives_from(&xs_any_simple_type));
        assert!(xs_integer.derives_from(&xs_any_type));
    }
}

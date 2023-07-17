const XS_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Xs {
    AnyType,
    AnySimpleType,
    Untyped,
    AnyAtomicType,
    String,
    UntypedAtomic,
    Boolean,
    Decimal,
    NonPositiveInteger,
    NegativeInteger,
    NonNegativeInteger,
    PositiveInteger,
    Integer,
    Long,
    Int,
    Short,
    Byte,
    UnsignedLong,
    UnsignedInt,
    UnsignedShort,
    UnsignedByte,
    Float,
    Double,
    QName,
    Notation,
    Duration,
    YearMonthDuration,
    DayTimeDuration,
    Time,
    GYearMonth,
    GYear,
    GMonthDay,
    GMonth,
    GDay,
    Base64Binary,
    HexBinary,
    AnyURI,
    DateTime,
    DateTimeStamp,
    Date,
    NormalizedString,
    Token,
    Language,
    NMTOKEN,
    Name,
    NCName,
    ID,
    IDREF,
    ENTITY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseNumericType {
    Decimal,
    Integer,
    Float,
    Double,
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

    pub fn rust_name(&self) -> &str {
        &self.rust_name
    }

    pub fn is_reference(&self) -> bool {
        self.as_ref
    }
}

impl Xs {
    pub fn by_name(namespace: Option<&str>, local_name: &str) -> Option<Self> {
        if namespace == Some(XS_NAMESPACE) {
            Xs::by_local_name(local_name)
        } else {
            None
        }
    }

    pub fn by_local_name(local_name: &str) -> Option<Self> {
        use Xs::*;
        let xs = match local_name {
            "anyType" => AnyType,
            "anySimpleType" => AnySimpleType,
            "untyped" => Untyped,
            "anyAtomicType" => AnyAtomicType,
            "string" => String,
            "untypedAtomic" => UntypedAtomic,
            "boolean" => Boolean,
            "decimal" => Decimal,
            "nonPositiveInteger" => NonPositiveInteger,
            "negativeInteger" => NegativeInteger,
            "nonNegativeInteger" => NonNegativeInteger,
            "positiveInteger" => PositiveInteger,
            "integer" => Integer,
            "long" => Long,
            "int" => Int,
            "short" => Short,
            "byte" => Byte,
            "unsignedLong" => UnsignedLong,
            "unsignedInt" => UnsignedInt,
            "unsignedShort" => UnsignedShort,
            "unsignedByte" => UnsignedByte,
            "float" => Float,
            "double" => Double,
            "QName" => QName,
            "NOTATION" => Notation,
            "duration" => Duration,
            "yearMonthDuration" => YearMonthDuration,
            "dayTimeDuration" => DayTimeDuration,
            "time" => Time,
            "gYearMonth" => GYearMonth,
            "gYear" => GYear,
            "gMonthDay" => GMonthDay,
            "gMonth" => GMonth,
            "gDay" => GDay,
            "base64Binary" => Base64Binary,
            "hexBinary" => HexBinary,
            "anyURI" => AnyURI,
            "dateTime" => DateTime,
            "dateTimeStamp" => DateTimeStamp,
            "date" => Date,
            "normalizedString" => NormalizedString,
            "token" => Token,
            "language" => Language,
            "NMTOKEN" => NMTOKEN,
            "Name" => Name,
            "NCName" => NCName,
            "ID" => ID,
            "IDREF" => IDREF,
            "ENTITY" => ENTITY,
            _ => return None,
        };
        Some(xs)
    }

    pub fn namespace() -> &'static str {
        XS_NAMESPACE
    }

    pub fn local_name(&self) -> &str {
        use Xs::*;
        match self {
            AnyType => "anyType",
            AnySimpleType => "anySimpleType",
            Untyped => "untyped",
            AnyAtomicType => "anyAtomicType",
            String => "string",
            UntypedAtomic => "untypedAtomic",
            Boolean => "boolean",
            Decimal => "decimal",
            NonPositiveInteger => "nonPositiveInteger",
            NegativeInteger => "negativeInteger",
            NonNegativeInteger => "nonNegativeInteger",
            PositiveInteger => "positiveInteger",
            Integer => "integer",
            Long => "long",
            Int => "int",
            Short => "short",
            Byte => "byte",
            UnsignedLong => "unsignedLong",
            UnsignedInt => "unsignedInt",
            UnsignedShort => "unsignedShort",
            UnsignedByte => "unsignedByte",
            Float => "float",
            Double => "double",
            QName => "QName",
            Notation => "NOTATION",
            Duration => "duration",
            YearMonthDuration => "yearMonthDuration",
            DayTimeDuration => "dayTimeDuration",
            Time => "time",
            GYearMonth => "gYearMonth",
            GYear => "gYear",
            GMonthDay => "gMonthDay",
            GMonth => "gMonth",
            GDay => "gDay",
            Base64Binary => "base64Binary",
            HexBinary => "hexBinary",
            AnyURI => "anyURI",
            DateTime => "dateTime",
            DateTimeStamp => "dateTimeStamp",
            Date => "date",
            NormalizedString => "normalizedString",
            Token => "token",
            Language => "language",
            NMTOKEN => "NMTOKEN",
            Name => "Name",
            NCName => "NCName",
            ID => "ID",
            IDREF => "IDREF",
            ENTITY => "ENTITY",
        }
    }

    pub fn parent(&self) -> Option<Xs> {
        use Xs::*;
        match self {
            AnyType => None,
            AnySimpleType => Some(AnyType),
            Untyped => Some(AnyType),
            AnyAtomicType => Some(AnySimpleType),
            UntypedAtomic => Some(AnyAtomicType),
            String => Some(UntypedAtomic),
            Boolean => Some(UntypedAtomic),
            Float => Some(UntypedAtomic),
            Double => Some(UntypedAtomic),
            Decimal => Some(UntypedAtomic),
            Integer => Some(Decimal),
            NonPositiveInteger => Some(Integer),
            NegativeInteger => Some(NonPositiveInteger),
            Long => Some(Integer),
            Int => Some(Long),
            Short => Some(Int),
            Byte => Some(Short),
            NonNegativeInteger => Some(Integer),
            PositiveInteger => Some(NonNegativeInteger),
            UnsignedLong => Some(NonNegativeInteger),
            UnsignedInt => Some(UnsignedLong),
            UnsignedShort => Some(UnsignedInt),
            UnsignedByte => Some(UnsignedShort),
            QName => Some(AnyAtomicType),
            Notation => Some(AnyAtomicType),
            Duration => Some(AnyAtomicType),
            YearMonthDuration => Some(Duration),
            DayTimeDuration => Some(Duration),
            Time => Some(AnyAtomicType),
            GYearMonth => Some(AnyAtomicType),
            GYear => Some(AnyAtomicType),
            GMonthDay => Some(AnyAtomicType),
            GMonth => Some(AnyAtomicType),
            GDay => Some(AnyAtomicType),
            Base64Binary => Some(AnyAtomicType),
            HexBinary => Some(AnyAtomicType),
            AnyURI => Some(AnyAtomicType),
            DateTime => Some(AnyAtomicType),
            DateTimeStamp => Some(DateTime),
            Date => Some(AnyAtomicType),
            NormalizedString => Some(String),
            Token => Some(NormalizedString),
            Language => Some(Token),
            NMTOKEN => Some(Token),
            Name => Some(Token),
            NCName => Some(Name),
            ID => Some(NCName),
            IDREF => Some(NCName),
            ENTITY => Some(NCName),
        }
    }

    pub fn derives_from(&self, other: Xs) -> bool {
        if self == &other {
            return true;
        }
        match self.parent() {
            Some(parent_type) => parent_type.derives_from(other),
            None => false,
        }
    }

    pub fn rust_info(&self) -> Option<RustInfo> {
        use Xs::*;
        match self {
            AnyType => None,
            AnySimpleType => None,
            Untyped => None,
            AnyAtomicType => None,
            UntypedAtomic => Some(RustInfo::as_ref("String")),
            String => Some(RustInfo::as_ref("String")),
            Boolean => Some(RustInfo::new("bool")),
            Float => Some(RustInfo::new("f32")),
            Double => Some(RustInfo::new("f64")),
            Decimal => Some(RustInfo::new("rust_decimal::Decimal")),
            Integer => Some(RustInfo::new("ibig::IBig")),
            NonPositiveInteger => Some(RustInfo::new("i64")),
            NegativeInteger => Some(RustInfo::new("i64")),
            Long => Some(RustInfo::new("i64")),
            Int => Some(RustInfo::new("i32")),
            Short => Some(RustInfo::new("i16")),
            Byte => Some(RustInfo::new("i8")),
            NonNegativeInteger => Some(RustInfo::new("u64")),
            PositiveInteger => Some(RustInfo::new("u64")),
            UnsignedLong => Some(RustInfo::new("u64")),
            UnsignedInt => Some(RustInfo::new("u32")),
            UnsignedShort => Some(RustInfo::new("u16")),
            UnsignedByte => Some(RustInfo::new("u8")),
            Notation => None,
            // TODO: need to find rust representation for the missing atomic types
            _ => None,
        }
    }

    pub fn base_numeric_type(&self) -> Option<BaseNumericType> {
        use Xs::*;
        match self {
            Float => Some(BaseNumericType::Float),
            Double => Some(BaseNumericType::Double),
            Decimal => Some(BaseNumericType::Decimal),
            Integer => Some(BaseNumericType::Integer),
            NonPositiveInteger => Some(BaseNumericType::Integer),
            NegativeInteger => Some(BaseNumericType::Integer),
            Long => Some(BaseNumericType::Integer),
            Int => Some(BaseNumericType::Integer),
            Short => Some(BaseNumericType::Integer),
            Byte => Some(BaseNumericType::Integer),
            NonNegativeInteger => Some(BaseNumericType::Integer),
            PositiveInteger => Some(BaseNumericType::Integer),
            UnsignedLong => Some(BaseNumericType::Integer),
            UnsignedInt => Some(BaseNumericType::Integer),
            UnsignedShort => Some(BaseNumericType::Integer),
            UnsignedByte => Some(BaseNumericType::Integer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derives_from() {
        assert!(Xs::Integer.derives_from(Xs::Integer));
        assert!(Xs::Integer.derives_from(Xs::Decimal));
        assert!(Xs::Integer.derives_from(Xs::AnyAtomicType));
        assert!(Xs::Integer.derives_from(Xs::AnySimpleType));
        assert!(Xs::Integer.derives_from(Xs::AnyType));
        assert!(Xs::Byte.derives_from(Xs::UntypedAtomic));
    }
}

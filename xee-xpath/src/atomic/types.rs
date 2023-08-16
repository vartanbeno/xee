use xee_schema_type::Xs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerType {
    Integer,
    NonPositiveInteger,
    NegativeInteger,
    NonNegativeInteger,
    PositiveInteger,
    Long,
    Int,
    Short,
    Byte,
    UnsignedLong,
    UnsignedInt,
    UnsignedShort,
    UnsignedByte,
}

impl IntegerType {
    pub(crate) fn schema_type(&self) -> Xs {
        match self {
            IntegerType::Integer => Xs::Integer,
            IntegerType::Long => Xs::Long,
            IntegerType::Int => Xs::Int,
            IntegerType::Short => Xs::Short,
            IntegerType::Byte => Xs::Byte,
            IntegerType::UnsignedLong => Xs::UnsignedLong,
            IntegerType::UnsignedInt => Xs::UnsignedInt,
            IntegerType::UnsignedShort => Xs::UnsignedShort,
            IntegerType::UnsignedByte => Xs::UnsignedByte,
            IntegerType::NonPositiveInteger => Xs::NonPositiveInteger,
            IntegerType::NegativeInteger => Xs::NegativeInteger,
            IntegerType::NonNegativeInteger => Xs::NonNegativeInteger,
            IntegerType::PositiveInteger => Xs::PositiveInteger,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    String,
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

impl StringType {
    pub(crate) fn schema_type(&self) -> Xs {
        match self {
            StringType::String => Xs::String,
            StringType::NormalizedString => Xs::NormalizedString,
            StringType::Token => Xs::Token,
            StringType::Language => Xs::Language,
            StringType::NMTOKEN => Xs::NMTOKEN,
            StringType::Name => Xs::Name,
            StringType::NCName => Xs::NCName,
            StringType::ID => Xs::ID,
            StringType::IDREF => Xs::IDREF,
            StringType::ENTITY => Xs::ENTITY,
        }
    }
}

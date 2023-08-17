use ibig::IBig;
use num_traits::Float;
use ordered_float::OrderedFloat;
use regex::Regex;
use rust_decimal::prelude::*;
use std::rc::Rc;
use std::sync::OnceLock;

use xee_schema_type::Xs;
use xee_xpath_ast::ast;

use crate::atomic;
use crate::context;
use crate::error;

// https://www.w3.org/TR/xml11/#NT-Nmtoken
// 	NameStartChar	   ::=   	":" | [A-Z] | "_" | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
// We create the NCName versions without colon (:) so we can do ncnames easily later
static NCNAME_START_CHAR: &str = r"A-Z_a-z\xc0-\xd6\xd8-\xf6\xf8-\u02ff\u0370-\u037d\u037f-\u1fff\u200c\u200d\u2070-\u218f\u2c00-\u2fef\u3001-\ud7ff\uf900-\ufdcf\ufdf0-\ufffd\U00010000-\U000effff";
// 	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
static NCNAME_CHAR_ADDITIONS: &str = r"-\.0-9\xb7\u0300-\u036F\u203F-\u2040";
static LANGUAGE_REGEX: OnceLock<Regex> = OnceLock::new();
static NMTOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static NAME_REGEX: OnceLock<Regex> = OnceLock::new();
static NC_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

impl atomic::Atomic {
    pub(crate) fn parse_atomic<V>(s: &str) -> error::Result<atomic::Atomic>
    where
        Parsed<V>: FromStr,
        V: Into<atomic::Atomic>,
    {
        // TODO: re-establish error. I am not sure what error is returned
        // from the FromStr trait; it should be error::Error but evidently it's
        // not
        s.parse::<Parsed<V>>()
            .map(|p| p.into_inner().into())
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_boolean(s: &str) -> error::Result<bool> {
        match s {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(error::Error::FORG0001),
        }
    }

    // from an atomic type to a canonical representation as a string
    pub(crate) fn into_canonical(self) -> String {
        match self {
            atomic::Atomic::String(_, s) => s.as_ref().clone(),
            atomic::Atomic::Untyped(s) => s.as_ref().clone(),
            atomic::Atomic::AnyURI(s) => s.as_ref().clone(),
            atomic::Atomic::Boolean(b) => {
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            atomic::Atomic::Decimal(d) => {
                if d.is_integer() {
                    let i = self.cast_to_integer_value::<IBig>().unwrap();
                    i.to_string()
                } else {
                    d.normalize().to_string()
                }
            }
            atomic::Atomic::Integer(_, i) => i.to_string(),
            atomic::Atomic::Float(OrderedFloat(f)) => canonical_float(f),
            atomic::Atomic::Double(OrderedFloat(f)) => canonical_float(f),
            atomic::Atomic::QName(name) => name.to_full_name(),
            atomic::Atomic::Binary(binary_type, data) => match binary_type {
                atomic::BinaryType::Hex => canonical_hex_binary(data.as_ref()),
                atomic::BinaryType::Base64 => canonical_base64_binary(data.as_ref()),
            },
        }
    }

    pub(crate) fn cast_to_schema_type(
        self,
        xs: Xs,
        dynamic_context: &context::DynamicContext,
    ) -> error::Result<atomic::Atomic> {
        // if we try to cast to any atomic type, we're already the correct type
        if xs == Xs::AnyAtomicType {
            return Ok(self);
        }
        if !xs.derives_from(Xs::AnyAtomicType) {
            todo!("We can only cast to atomic types right now")
        }
        if self.schema_type() == xs {
            return Ok(self.clone());
        }
        match xs {
            Xs::String => Ok(self.cast_to_string()),
            Xs::NormalizedString => Ok(self.cast_to_normalized_string()),
            Xs::Token => Ok(self.cast_to_token()),
            Xs::Language => self.cast_to_language(),
            Xs::NMTOKEN => self.cast_to_nmtoken(),
            Xs::Name => self.cast_to_name(),
            Xs::NCName => self.cast_to_ncname(),
            Xs::ID => self.cast_to_id(),
            Xs::IDREF => self.cast_to_idref(),
            Xs::ENTITY => self.cast_to_entity(),
            Xs::UntypedAtomic => Ok(self.cast_to_untyped_atomic()),
            Xs::AnyURI => self.cast_to_any_uri(),
            Xs::Boolean => self.cast_to_boolean(),
            Xs::Decimal => self.cast_to_decimal(),
            Xs::Integer => self.cast_to_integer(),
            Xs::Long => self.cast_to_long(),
            Xs::Int => self.cast_to_int(),
            Xs::Short => self.cast_to_short(),
            Xs::Byte => self.cast_to_byte(),
            Xs::UnsignedLong => self.cast_to_unsigned_long(),
            Xs::UnsignedInt => self.cast_to_unsigned_int(),
            Xs::UnsignedShort => self.cast_to_unsigned_short(),
            Xs::UnsignedByte => self.cast_to_unsigned_byte(),
            Xs::Float => self.cast_to_float(),
            Xs::Double => self.cast_to_double(),
            Xs::NonPositiveInteger => self.cast_to_non_positive_integer(),
            Xs::NegativeInteger => self.cast_to_negative_integer(),
            Xs::NonNegativeInteger => self.cast_to_non_negative_integer(),
            Xs::PositiveInteger => self.cast_to_positive_integer(),
            Xs::QName => self.cast_to_qname(dynamic_context),
            Xs::HexBinary => self.cast_to_hex_binary(),
            Xs::Base64Binary => self.cast_to_base64_binary(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn cast_to_schema_type_of(
        self,
        other: &atomic::Atomic,
        context: &context::DynamicContext,
    ) -> error::Result<atomic::Atomic> {
        self.cast_to_schema_type(other.schema_type(), context)
    }

    // if a derives from b, cast to b, otherwise vice versa
    pub(crate) fn cast_to_same_schema_type(
        self,
        other: atomic::Atomic,
        context: &context::DynamicContext,
    ) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
        if self.derives_from(&other) {
            let a = self.cast_to_schema_type_of(&other, context)?;
            Ok((a, other))
        } else if other.derives_from(&self) {
            let b = other.cast_to_schema_type_of(&self, context)?;
            Ok((self, b))
        } else {
            Err(error::Error::Type)
        }
    }

    pub(crate) fn cast_to_string(self) -> atomic::Atomic {
        atomic::Atomic::String(atomic::StringType::String, Rc::new(self.into_canonical()))
    }

    pub(crate) fn cast_to_untyped_atomic(self) -> atomic::Atomic {
        atomic::Atomic::Untyped(Rc::new(self.into_canonical()))
    }

    pub(crate) fn cast_to_any_uri(self) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xpath-functions-31/#casting-to-anyuri
        match self {
            atomic::Atomic::AnyURI(s) => Ok(atomic::Atomic::AnyURI(s.clone())),
            atomic::Atomic::String(_, s) => Ok(atomic::Atomic::AnyURI(s.clone())),
            atomic::Atomic::Untyped(s) => Ok(atomic::Atomic::AnyURI(s.clone())),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_normalized_string(self) -> atomic::Atomic {
        let s = whitespace_replace(&self.into_canonical());
        atomic::Atomic::String(atomic::StringType::NormalizedString, Rc::new(s))
    }

    pub(crate) fn cast_to_token(self) -> atomic::Atomic {
        let s = whitespace_collapse(&self.into_canonical());
        atomic::Atomic::String(atomic::StringType::Token, Rc::new(s))
    }

    fn cast_to_regex<F>(
        self,
        string_type: atomic::StringType,
        regex_once_lock: &OnceLock<Regex>,
        f: F,
    ) -> error::Result<atomic::Atomic>
    where
        F: FnOnce() -> Regex,
    {
        let regex = regex_once_lock.get_or_init(f);
        let s = whitespace_collapse(&self.into_canonical());
        if regex.is_match(&s) {
            Ok(atomic::Atomic::String(string_type, Rc::new(s)))
        } else {
            Err(error::Error::FORG0001)
        }
    }

    pub(crate) fn cast_to_language(self) -> error::Result<atomic::Atomic> {
        self.cast_to_regex(atomic::StringType::Language, &LANGUAGE_REGEX, || {
            Regex::new(r"^[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*$").expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_nmtoken(self) -> error::Result<atomic::Atomic> {
        // Nmtoken	 ::= (NameChar)+
        self.cast_to_regex(atomic::StringType::NMTOKEN, &NMTOKEN_REGEX, || {
            // we have to add the colon for NAME_START_CHAR / NAME_CHAR
            Regex::new(&format!(
                "^[:{}{}]+$",
                NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_name(self) -> error::Result<atomic::Atomic> {
        // 	Name	   ::=   	NameStartChar (NameChar)*
        self.cast_to_regex(atomic::StringType::Name, &NAME_REGEX, || {
            // we have to add the colon for NAME_START_CHAR / NAME_CHAR
            Regex::new(&format!(
                "^[:{}][:{}{}]*$",
                NCNAME_START_CHAR, NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    fn cast_to_ncname_helper(
        self,
        string_type: atomic::StringType,
    ) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xml-names11/#NT-NCName
        // 	NCName	   ::=   	NCNameStartChar NCNameChar*
        // 	NCNameChar	   ::=   	NameChar - ':'
        //	NCNameStartChar	   ::=   	NameStartChar - ':'
        self.cast_to_regex(string_type, &NC_NAME_REGEX, || {
            Regex::new(&format!(
                "^[{}][{}{}]*$",
                NCNAME_START_CHAR, NCNAME_START_CHAR, NCNAME_CHAR_ADDITIONS
            ))
            .expect("Invalid regex")
        })
    }

    pub(crate) fn cast_to_ncname(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::NCName)
    }

    pub(crate) fn cast_to_id(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::ID)
    }

    pub(crate) fn cast_to_idref(self) -> error::Result<atomic::Atomic> {
        self.cast_to_ncname_helper(atomic::StringType::IDREF)
    }

    pub(crate) fn cast_to_entity(self) -> error::Result<atomic::Atomic> {
        // https://www.w3.org/TR/xpath-functions-31/#casting-to-ENTITY
        // we don't need to check whether it matches unparsed entities
        self.cast_to_ncname_helper(atomic::StringType::ENTITY)
    }

    pub(crate) fn cast_to_boolean(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Boolean(_) => Ok(self.clone()),
            atomic::Atomic::Float(f) => Ok(atomic::Atomic::Boolean(!(f.is_nan() || f.is_zero()))),
            atomic::Atomic::Double(d) => Ok(atomic::Atomic::Boolean(!(d.is_nan() || d.is_zero()))),
            atomic::Atomic::Decimal(d) => Ok(atomic::Atomic::Boolean(!d.is_zero())),
            atomic::Atomic::Integer(_, i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::String(_, s) => Self::parse_atomic::<bool>(&s),
            atomic::Atomic::Untyped(s) => Self::parse_atomic::<bool>(&s),
            atomic::Atomic::AnyURI(_) | atomic::Atomic::QName(_) | atomic::Atomic::Binary(_, _) => {
                Err(error::Error::Type)
            }
        }
    }

    pub(crate) fn cast_to_qname(
        self,
        dynamic_context: &context::DynamicContext,
    ) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::QName(_) => Ok(self.clone()),
            atomic::Atomic::String(_, s) | atomic::Atomic::Untyped(s) => {
                // https://www.w3.org/TR/xpath-functions-31/#constructor-qname-notation
                let namespaces = dynamic_context.static_context.namespaces;

                let name = ast::Name::parse(&s, namespaces);
                match name {
                    Ok(name) => {
                        let name = name.value;
                        if name.has_namespace_without_prefix() {
                            // we deliberately do not parse Qualified names, as they are not
                            // legal for xs:QName
                            Err(error::Error::FORG0001)
                        } else {
                            Ok(atomic::Atomic::QName(Rc::new(name)))
                        }
                    }
                    // TODO: We really want to distinguish between parse errors
                    // and namespace lookup errors, which should be a FONS0004 error
                    // This requires the parser to be modified so it retains that
                    // information.
                    Err(_) => Err(error::Error::FORG0001),
                }
            }
            _ => Err(error::Error::Type),
        }
    }

    fn cast_to_binary<F>(
        self,
        binary_type: atomic::BinaryType,
        decode: F,
    ) -> error::Result<atomic::Atomic>
    where
        F: Fn(&str) -> error::Result<Vec<u8>>,
    {
        match self {
            atomic::Atomic::Binary(_, data) => Ok(atomic::Atomic::Binary(binary_type, data)),
            atomic::Atomic::String(_, s) | atomic::Atomic::Untyped(s) => {
                let s = s.as_ref();
                let s = whitespace_remove(s);
                let data = decode(&s)?;
                Ok(atomic::Atomic::Binary(binary_type, Rc::new(data)))
            }
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_hex_binary(self) -> error::Result<atomic::Atomic> {
        self.cast_to_binary(atomic::BinaryType::Hex, |s: &str| {
            let data = hex::decode(s);
            data.map_err(|_| error::Error::FORG0001)
        })
    }

    pub(crate) fn cast_to_base64_binary(self) -> error::Result<atomic::Atomic> {
        self.cast_to_binary(atomic::BinaryType::Base64, |s: &str| {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(s)
                .map_err(|_| error::Error::FORG0001)
        })
    }
}

pub(crate) struct Parsed<V>(pub(crate) V);

impl<V> Parsed<V> {
    pub(crate) fn into_inner(self) -> V {
        self.0
    }
}

impl FromStr for Parsed<bool> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_boolean(s)?))
    }
}

struct WrappedIBig(IBig);

fn canonical_float<F>(f: F) -> String
where
    F: Float
        + TryInto<Decimal, Error = rust_decimal::Error>
        + lexical::ToLexicalWithOptions<Options = lexical::WriteFloatOptions>
        + num::Signed,
{
    // https://www.w3.org/TR/xpath-functions-31/#casting-to-string
    // If SV has an absolute value that is greater than or equal to
    // 0.000001 (one millionth) and less than 1000000 (one
    // million), then the value is converted to an xs:decimal and
    // the resulting xs:decimal is converted to an xs:string
    let abs_f = f.abs();
    let minimum: F = num::cast(0.000001).unwrap();
    let maximum: F = num::cast(1000000.0).unwrap();
    if abs_f >= minimum && abs_f < maximum {
        // TODO: is this the right conversion?
        let d: Decimal = f.try_into().unwrap();
        atomic::Atomic::Decimal(d).into_canonical()
    } else {
        if f.is_zero() {
            if f.is_negative() {
                return "-0".to_string();
            } else {
                return "0".to_string();
            }
        }
        let options = lexical::WriteFloatOptionsBuilder::new()
            .exponent(b'E')
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::to_string_with_options::<_, { lexical::format::XML }>(f, &options)
    }
}

fn canonical_hex_binary(data: &[u8]) -> String {
    hex::encode_upper(data)
}

fn canonical_base64_binary(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn whitespace_replace(s: &str) -> String {
    // XML Schema whitespace: replace
    // all tab, linefeeds and carriage returns are replaced with a space
    // character
    s.replace(|c| c == '\t' || c == '\n' || c == '\r', " ")
}

fn whitespace_collapse(s: &str) -> String {
    // XML Schema whitespace: collapse
    // after doing a replace, collapse all space characters into a single
    // space character. Any space characters at the start or end of string
    // are then removed.
    whitespace_replace(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn whitespace_remove(s: &str) -> String {
    // XML Schema whitespace: remove
    // after doing a replace, remove all space characters
    whitespace_replace(s).replace(' ', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    use ibig::ibig;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_decimal() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<Decimal>("1.0").unwrap(),
            atomic::Atomic::Decimal(dec!(1.0))
        );
    }

    #[test]
    fn test_parse_decimal_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<Decimal>("1_000.0"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<IBig>("1").unwrap(),
            atomic::Atomic::Integer(atomic::IntegerType::Integer, ibig!(1).into())
        );
    }

    #[test]
    fn test_parse_long() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<i64>("1").unwrap(),
            atomic::Atomic::Integer(atomic::IntegerType::Long, ibig!(1).into())
        );
    }

    #[test]
    fn test_parse_integer_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<i64>("1_000"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_double() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0))
        );
    }

    #[test]
    fn test_parse_double_exponent() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0e10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_exponent_capital() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0E10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_inf() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_minus_inf() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("-INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(-f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_nan() {
        let a = atomic::Atomic::parse_atomic::<f64>("NaN").unwrap();
        match a {
            atomic::Atomic::Double(OrderedFloat(f)) => assert!(f.is_nan()),
            _ => panic!("Expected double"),
        }
    }

    #[test]
    fn test_parse_double_invalid_nan() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("NAN"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_canonical_decimal_is_integer() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.0)).into_canonical(), "1");
    }

    #[test]
    fn test_canonical_decimal_is_decimal() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.5)).into_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_no_trailing_zeroes() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.50)).into_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_no_leading_zeroes() {
        assert_eq!(atomic::Atomic::Decimal(dec!(01.50)).into_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_single_leading_zero() {
        assert_eq!(atomic::Atomic::Decimal(dec!(0.50)).into_canonical(), "0.5");
    }

    #[test]
    fn test_canonical_integer() {
        assert_eq!(
            atomic::Atomic::Integer(atomic::IntegerType::Integer, ibig!(15).into())
                .into_canonical(),
            "15"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_integer() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.0)).into_canonical(),
            "15"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.5)).into_canonical(),
            "15.5"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_float_big() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(1500000000000000f32)).into_canonical(),
            "1.5E15"
        );
    }

    #[test]
    fn test_canonical_formatted_as_float_small() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(0.000000000000001f32)).into_canonical(),
            "1.0E-15"
        );
    }

    #[test]
    fn test_canonical_float_zero() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(0.0)).into_canonical(),
            "0"
        );
    }

    #[test]
    fn test_canonical_float_minus_zero() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(-0.0)).into_canonical(),
            "-0"
        );
    }

    #[test]
    fn test_canonical_float_inf() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY)).into_canonical(),
            "INF"
        );
    }

    #[test]
    fn test_canonical_double_formatted_as_decimal() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(15.5)).into_canonical(),
            "15.5"
        );
    }

    #[test]
    fn test_cast_double_to_float_inf() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_negative_inf() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f64::NEG_INFINITY))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::NEG_INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_nan() {
        let a = atomic::Atomic::Double(OrderedFloat(f64::NAN))
            .cast_to_float()
            .unwrap();
        match a {
            atomic::Atomic::Float(OrderedFloat(f)) => assert!(f.is_nan()),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_cast_double_to_float_zero() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(0.0))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_negative_zero() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(-0.0))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(-0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_too_big() {
        // TODO: are the boundaries exactly as in spec?
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f32::MAX as f64 * 1.1))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_too_small() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(1e-150))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_not_too_small() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(1e-5))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(1e-5))
        );
    }

    #[test]
    fn test_cast_int_to_decimal() {
        assert_eq!(
            atomic::Atomic::Integer(atomic::IntegerType::Integer, ibig!(15).into())
                .cast_to_decimal()
                .unwrap(),
            atomic::Atomic::Decimal(dec!(15))
        );
    }

    #[test]
    fn test_cast_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.5))
                .cast_to_decimal()
                .unwrap(),
            atomic::Atomic::Decimal(dec!(15.5))
        );
    }

    #[test]
    fn test_cast_inf_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY)).cast_to_decimal(),
            Err(error::Error::FOCA0002)
        );
    }

    #[test]
    fn test_cast_huge_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(1.0e30)).cast_to_decimal(),
            Err(error::Error::FOCA0001)
        );
    }

    #[test]
    fn test_cast_short_to_short() {
        assert_eq!(
            atomic::Atomic::Integer(atomic::IntegerType::Short, ibig!(15).into())
                .cast_to_short()
                .unwrap(),
            atomic::Atomic::Integer(atomic::IntegerType::Short, ibig!(15).into())
        );
    }
}

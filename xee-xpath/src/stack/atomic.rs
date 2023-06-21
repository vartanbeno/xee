use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::rc::Rc;

use crate::comparison;
use crate::stack;

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, Eq)]
pub enum Atomic {
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),
    Decimal(Decimal),
    String(Rc<String>),
    Untyped(Rc<String>),
    // a special marker to indicate an absent context item
    Absent,
}

impl Atomic {
    pub(crate) fn to_integer(&self) -> stack::Result<i64> {
        match self {
            Atomic::Integer(i) => Ok(*i),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_decimal(&self) -> stack::Result<Decimal> {
        match self {
            Atomic::Decimal(d) => Ok(*d),
            Atomic::Integer(i) => Ok(Decimal::from(*i)),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_float(&self) -> stack::Result<OrderedFloat<f32>> {
        match self {
            Atomic::Float(f) => Ok(*f),
            Atomic::Decimal(d) => Ok(OrderedFloat(d.to_f32().ok_or(stack::Error::Type)?)),
            Atomic::Integer(_) => Ok(OrderedFloat(
                self.to_decimal()?.to_f32().ok_or(stack::Error::Type)?,
            )),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_double(&self) -> stack::Result<OrderedFloat<f64>> {
        match self {
            Atomic::Double(d) => Ok(*d),
            Atomic::Float(OrderedFloat(f)) => Ok(OrderedFloat(*f as f64)),
            Atomic::Decimal(d) => Ok(OrderedFloat(d.to_f64().ok_or(stack::Error::Type)?)),
            Atomic::Integer(_) => Ok(OrderedFloat(
                self.to_decimal()?.to_f64().ok_or(stack::Error::Type)?,
            )),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn effective_boolean_value(&self) -> stack::Result<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
            Atomic::Boolean(b) => Ok(*b),
            Atomic::String(s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
            Atomic::Absent => Err(stack::Error::Absent),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub(crate) fn to_str(&self) -> stack::Result<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_string(&self) -> stack::Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub(crate) fn string_value(&self) -> stack::Result<String> {
        Ok(match self {
            Atomic::String(s) => s.to_string(),
            Atomic::Untyped(s) => s.to_string(),
            Atomic::Boolean(b) => b.to_string(),
            Atomic::Integer(i) => i.to_string(),
            Atomic::Float(f) => f.to_string(),
            Atomic::Double(d) => d.to_string(),
            Atomic::Decimal(d) => d.to_string(),
            Atomic::Absent => Err(stack::Error::Absent)?,
        })
    }

    pub(crate) fn is_nan(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_nan(),
            Atomic::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    pub(crate) fn is_infinite(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_infinite(),
            Atomic::Double(d) => d.is_infinite(),
            _ => false,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_zero(),
            Atomic::Double(d) => d.is_zero(),
            Atomic::Decimal(d) => d.is_zero(),
            Atomic::Integer(i) => *i == 0,
            _ => false,
        }
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            Atomic::Float(_) | Atomic::Double(_) | Atomic::Decimal(_) | Atomic::Integer(_)
        )
    }

    pub(crate) fn is_true(&self) -> bool {
        if let Atomic::Boolean(b) = self {
            *b
        } else {
            false
        }
    }

    pub(crate) fn general_comparison_cast(&self, v: &str) -> stack::Result<Atomic> {
        match self {
            // i. If T is a numeric type or is derived from a numeric type, then V
            // is cast to xs:double.
            Atomic::Integer(_) | Atomic::Decimal(_) | Atomic::Float(_) | Atomic::Double(_) => {
                // cast string to double
                // Need to unify the parsing code with literal parser in parse_ast
                Ok(Atomic::Double(OrderedFloat(
                    v.parse::<f64>().map_err(|_| stack::Error::Overflow)?,
                )))
            }
            // don't handle ii and iii for now
            // iv. In all other cases, V is cast to the primitive base type of T.
            Atomic::String(_) => Ok(Atomic::String(Rc::new(v.to_string()))),
            Atomic::Boolean(_) => {
                // XXX casting rules are way more complex, see 19.2 in the
                // XPath and Functions spec
                Ok(Atomic::Boolean(
                    v.parse::<bool>().map_err(|_| stack::Error::Type)?,
                ))
            }
            Atomic::Untyped(_) => unreachable!(),
            Atomic::Absent => Err(stack::Error::Type),
        }
    }
}

impl PartialEq for Atomic {
    fn eq(&self, other: &Self) -> bool {
        match comparison::value_eq(self, other) {
            Ok(b) => b.is_true(),
            Err(_) => false,
        }
    }
}

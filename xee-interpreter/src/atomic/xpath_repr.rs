use super::{Atomic, StringType};

impl Atomic {
    /// XPath representation of the atomic value.
    pub fn xpath_representation(&self) -> String {
        match self {
            Atomic::String(string_type, v) => match string_type {
                StringType::String => {
                    if v.contains('\"') {
                        if v.contains('\'') {
                            let v = v.replace('\"', r#""""#);
                            format!(r#""{}""#, v)
                        } else {
                            format!(r#"'{}'"#, v)
                        }
                    } else {
                        format!(r#""{}""#, v)
                    }
                }
                _ => todo!(),
            },
            Atomic::Untyped(_) => todo!(),
            Atomic::Boolean(_) => todo!(),
            Atomic::Decimal(_) => todo!(),
            Atomic::Integer(integer_type, _) => todo!(),
            Atomic::Float(_) => todo!(),
            Atomic::Double(_) => todo!(),
            Atomic::QName(_) => todo!(),
            Atomic::Binary(binary_type, _) => todo!(),
            Atomic::Duration(_) => todo!(),
            Atomic::YearMonthDuration(_) => todo!(),
            Atomic::DayTimeDuration(_) => todo!(),
            Atomic::Time(_) => todo!(),
            Atomic::Date(_) => todo!(),
            Atomic::DateTime(_) => todo!(),
            Atomic::DateTimeStamp(_) => todo!(),
            Atomic::GYearMonth(_) => todo!(),
            Atomic::GYear(_) => todo!(),
            Atomic::GMonthDay(_) => todo!(),
            Atomic::GMonth(_) => todo!(),
            Atomic::GDay(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_simple() {
        let atomic: Atomic = "foo".into();
        assert_eq!(atomic.xpath_representation(), r#""foo""#);
    }

    #[test]
    fn test_string_with_single_quote() {
        let atomic: Atomic = "foo'bar".into();
        assert_eq!(atomic.xpath_representation(), r#""foo'bar""#);
    }

    #[test]
    fn test_string_with_double_quote() {
        let atomic: Atomic = r#"foo"bar"#.into();
        assert_eq!(atomic.xpath_representation(), r#"'foo"bar'"#);
    }

    #[test]
    fn test_string_with_both_quotes() {
        let atomic: Atomic = r#"foo'bar"baz"#.into();
        assert_eq!(atomic.xpath_representation(), r#""foo'bar""baz""#);
    }
}

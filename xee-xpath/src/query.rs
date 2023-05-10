use crate::dynamic_context::DynamicContext;
use crate::error::Error;
use crate::error::Result;
use crate::static_context::StaticContext;
use crate::value::{Item, ValueError};
use crate::xpath::XPath;

pub trait Convert<V>: Fn(Item) -> std::result::Result<V, ValueError> {}
impl<T, V> Convert<V> for T where T: Fn(Item) -> std::result::Result<V, ValueError> {}

#[derive(Debug)]
pub struct OneQuery<'a, T, F>
where
    F: Convert<T>,
{
    xpath: XPath<'a>,
    convert: F,
    t: std::marker::PhantomData<T>,
}

impl<'a, T, F> OneQuery<'a, T, F>
where
    F: Convert<T>,
{
    pub fn new(static_context: &'a StaticContext<'a>, s: &str, convert: F) -> Result<Self> {
        let xpath = XPath::new(static_context, s)?;
        Ok(Self {
            xpath,
            convert,
            t: std::marker::PhantomData,
        })
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<T> {
        let item = self.xpath.one(dynamic_context, item)?;
        (self.convert)(item).map_err(|value_error| {
            Error::from_value_error(&self.xpath.program, (0, 0).into(), value_error)
        })
    }
}

#[derive(Debug)]
pub struct OptionQuery<'a, T, F>
where
    F: Convert<T>,
{
    xpath: XPath<'a>,
    convert: F,
    t: std::marker::PhantomData<T>,
}

impl<'a, T, F> OptionQuery<'a, T, F>
where
    F: Convert<T>,
{
    pub fn new(static_context: &'a StaticContext<'a>, s: &str, convert: F) -> Result<Self> {
        let xpath = XPath::new(static_context, s)?;
        Ok(Self {
            xpath,
            convert,
            t: std::marker::PhantomData,
        })
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Option<T>> {
        let item = self.xpath.option(dynamic_context, item)?;
        if let Some(item) = item {
            match (self.convert)(item) {
                Ok(value) => Ok(Some(value)),
                Err(value_error) => Err(Error::from_value_error(
                    &self.xpath.program,
                    (0, 0).into(),
                    value_error,
                )),
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xot::Xot;

    use crate::Atomic;
    use crate::Namespaces;

    #[test]
    fn test_one_query() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let q = OneQuery::new(&static_context, "1 + 2", |item: Item| {
            item.as_atomic()?.as_integer()
        })
        .unwrap();
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, 3);
    }

    #[test]
    fn test_option_query_some() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let q = OptionQuery::new(&static_context, "1 + 2", |item: Item| {
            item.as_atomic()?.as_integer()
        })
        .unwrap();
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, Some(3));
    }

    #[test]
    fn test_option_query_none() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let q = OptionQuery::new(&static_context, "()", |item: Item| {
            item.as_atomic()?.as_integer()
        })
        .unwrap();
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, None);
    }
}

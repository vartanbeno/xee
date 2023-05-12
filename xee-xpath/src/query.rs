use crate::dynamic_context::DynamicContext;
use crate::error::Error;
use crate::error::Result;
use crate::static_context::StaticContext;
use crate::value::{Item, ValueError};
use crate::xpath::XPath;

pub trait Convert<V>: Fn(&DynamicContext, &Item) -> std::result::Result<V, ConvertError> {}
impl<T, V> Convert<V> for T where
    T: Fn(&DynamicContext, &Item) -> std::result::Result<V, ConvertError>
{
}

/// Convert functions may return either a ValueError, or do queries of their
/// own, which can result in a Error. We want to handle them both.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("Value error")]
    ValueError(#[from] ValueError),
    #[error("Error")]
    Error(#[from] Error),
}

#[derive(Debug)]
pub struct ManyQuery<V, F>
where
    F: Convert<V>,
{
    xpath: XPath,
    convert: F,
    t: std::marker::PhantomData<V>,
}

impl<V, F> ManyQuery<V, F>
where
    F: Convert<V>,
{
    pub fn new(static_context: &StaticContext, s: &str, convert: F) -> Result<Self> {
        let xpath = XPath::new(static_context, s)?;
        Ok(Self {
            xpath,
            convert,
            t: std::marker::PhantomData,
        })
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Vec<V>> {
        let items = self.xpath.many(dynamic_context, item)?;
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            let value =
                (self.convert)(dynamic_context, &item).map_err(
                    |query_error| match query_error {
                        ConvertError::ValueError(value_error) => {
                            Error::from_value_error(&self.xpath.program, (0, 0).into(), value_error)
                        }
                        ConvertError::Error(error) => error,
                    },
                )?;
            result.push(value);
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub struct OneQuery<V, F>
where
    F: Convert<V>,
{
    xpath: XPath,
    convert: F,
    t: std::marker::PhantomData<V>,
}

impl<V, F> OneQuery<V, F>
where
    F: Convert<V>,
{
    pub fn new(static_context: &StaticContext, s: &str, convert: F) -> Result<Self> {
        let xpath = XPath::new(static_context, s)?;
        Ok(Self {
            xpath,
            convert,
            t: std::marker::PhantomData,
        })
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<V> {
        let item = self.xpath.one(dynamic_context, item)?;
        (self.convert)(dynamic_context, &item).map_err(|query_error| match query_error {
            ConvertError::ValueError(value_error) => {
                Error::from_value_error(&self.xpath.program, (0, 0).into(), value_error)
            }
            ConvertError::Error(error) => error,
        })
    }
}

#[derive(Debug)]
pub struct OptionQuery<V, F>
where
    F: Convert<V>,
{
    xpath: XPath,
    convert: F,
    t: std::marker::PhantomData<V>,
}

impl<V, F> OptionQuery<V, F>
where
    F: Convert<V>,
{
    pub fn new(static_context: &StaticContext, s: &str, convert: F) -> Result<Self> {
        let xpath = XPath::new(static_context, s)?;
        Ok(Self {
            xpath,
            convert,
            t: std::marker::PhantomData,
        })
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<Option<V>> {
        let item = self.xpath.option(dynamic_context, item)?;
        if let Some(item) = item {
            match (self.convert)(dynamic_context, &item) {
                Ok(value) => Ok(Some(value)),
                Err(query_error) => match query_error {
                    ConvertError::ValueError(value_error) => Err(Error::from_value_error(
                        &self.xpath.program,
                        (0, 0).into(),
                        value_error,
                    )),
                    ConvertError::Error(error) => Err(error),
                },
            }
        } else {
            Ok(None)
        }
    }
}

pub struct OneQueryRef<'s, V> {
    inner: std::cell::RefCell<Option<Box<dyn OneQueryTrait<V> + 's>>>,
}

trait OneQueryTrait<V> {
    fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<V>;
}

impl<V, F> OneQueryTrait<V> for OneQuery<V, F>
where
    F: Convert<V>,
{
    fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<V> {
        OneQuery::execute(self, dynamic_context, item)
    }
}

impl<'s, V: 's> OneQueryRef<'s, V> {
    pub fn new() -> Self {
        Self {
            inner: std::cell::RefCell::new(None),
        }
    }

    pub fn execute(&self, dynamic_context: &DynamicContext, item: &Item) -> Result<V> {
        if let Some(inner) = self.inner.borrow().as_ref() {
            inner.execute(dynamic_context, item)
        } else {
            panic!("No query set")
        }
    }

    pub fn fulfill(
        &self,
        static_context: &StaticContext,
        s: &str,
        f: Box<dyn Convert<V> + 's>,
    ) -> Result<()> {
        let mut inner = self.inner.borrow_mut();
        let query = OneQuery::new(static_context, s, f)?;
        *inner = Some(Box::new(query));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xot::Xot;

    use crate::Atomic;
    use crate::Namespaces;

    #[test]
    fn test_many_query() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let q = ManyQuery::new(&static_context, "(3, 4)", |_, item| {
            Ok(item.as_atomic()?.as_integer()?)
        })
        .unwrap();
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, vec![3, 4]);
    }

    #[test]
    fn test_one_query() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let q = OneQuery::new(&static_context, "1 + 2", |_, item| {
            Ok(item.as_atomic()?.as_integer()?)
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
        let q = OptionQuery::new(&static_context, "1 + 2", |_, item| {
            Ok(item.as_atomic()?.as_integer()?)
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
        let q = OptionQuery::new(&static_context, "()", |_, item| {
            Ok(item.as_atomic()?.as_integer()?)
        })
        .unwrap();
        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, None);
    }

    #[test]
    fn test_one_query_ref() {
        let namespaces = Namespaces::default();
        let static_context = StaticContext::new(&namespaces);
        let one_query_ref = OneQueryRef::new();
        let q = OneQuery::new(&static_context, "1 + 2", |dynamic_context, item| {
            let v = item.as_atomic()?.as_integer()?;
            let s = one_query_ref.execute(dynamic_context, item)?;
            Ok(v + s)
        })
        .unwrap();
        one_query_ref
            .fulfill(
                &static_context,
                "5",
                Box::new(|_, item| Ok(item.as_atomic()?.as_integer()?)),
            )
            .unwrap();

        let xot = Xot::new();
        let dynamic_context = DynamicContext::new(&xot, &static_context);
        let r = q
            .execute(&dynamic_context, &Item::Atomic(Atomic::Integer(1)))
            .unwrap();
        assert_eq!(r, 8);
    }
}

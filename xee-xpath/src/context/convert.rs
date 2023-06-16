use super::DynamicContext;

pub(crate) trait ContextFrom<'a, T>: Sized {
    fn context_from(value: T, context: &DynamicContext<'a>) -> Self;
}

pub(crate) trait ContextTryFrom<'a, T>: Sized {
    type Error;

    fn context_try_from(value: T, context: &DynamicContext<'a>) -> Result<Self, Self::Error>;
}

pub(crate) trait ContextInto<'a, T>: Sized {
    fn context_into(self, context: &DynamicContext<'a>) -> T;
}

pub(crate) trait ContextTryInto<'a, T>: Sized {
    type Error;

    fn context_try_into(self, context: &DynamicContext<'a>) -> Result<T, Self::Error>;
}

impl<'a, T, U> ContextInto<'a, U> for T
where
    U: ContextFrom<'a, T>,
{
    fn context_into(self, context: &DynamicContext<'a>) -> U {
        U::context_from(self, context)
    }
}

impl<'a, T, U> ContextTryInto<'a, U> for T
where
    U: ContextTryFrom<'a, T>,
{
    type Error = U::Error;

    fn context_try_into(self, context: &DynamicContext<'a>) -> Result<U, Self::Error> {
        U::context_try_from(self, context)
    }
}

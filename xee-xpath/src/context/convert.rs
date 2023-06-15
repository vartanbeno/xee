use crate::stack;

use super::DynamicContext;

pub(crate) trait ContextFrom<T>: Sized {
    fn context_from(value: T, context: &DynamicContext) -> Self;
}

pub(crate) trait ContextTryFrom<T>: Sized {
    type Error;

    fn context_try_from(value: T, context: &DynamicContext) -> Result<Self, Self::Error>;
}

pub(crate) trait ContextInto<T>: Sized {
    fn context_into(self, context: &DynamicContext) -> T;
}

pub(crate) trait ContextTryInto<T>: Sized {
    type Error;

    fn context_try_into(self, context: &DynamicContext) -> Result<T, Self::Error>;
}

impl<T, U> ContextInto<U> for T
where
    U: ContextFrom<T>,
{
    fn context_into(self, context: &DynamicContext) -> U {
        U::context_from(self, context)
    }
}

impl<T, U> ContextTryInto<U> for T
where
    U: ContextTryFrom<T>,
{
    type Error = U::Error;

    fn context_try_into(self, context: &DynamicContext) -> Result<U, Self::Error> {
        U::context_try_from(self, context)
    }
}

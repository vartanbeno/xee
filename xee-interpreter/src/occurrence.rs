use std::result::Result;

use crate::error;

pub trait Occurrence<A, E>
where
    A: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn one(&mut self) -> Result<A, E>;
    fn option(&mut self) -> Result<Option<A>, E>;
    fn many(&mut self) -> Result<Vec<A>, E>;
    fn error(&self) -> E;
}

pub(crate) fn one<'a, T>(
    mut iter: impl Iterator<Item = error::Result<T>> + 'a,
) -> error::Result<T> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(one?)
        } else {
            Err(error::Error::XPTY0004)
        }
    } else {
        Err(error::Error::XPTY0004)
    }
}

pub(crate) fn option<'a, T>(
    mut iter: impl Iterator<Item = error::Result<T>> + 'a,
) -> error::Result<Option<T>> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(Some(one?))
        } else {
            Err(error::Error::XPTY0004)
        }
    } else {
        Ok(None)
    }
}

use std::result::Result;

/// An occurrence API for an iterator that returns results.
pub trait Occurrence<A, E>: Iterator<Item = Result<A, E>>
where
    A: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn one(&mut self) -> Result<A, E> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(one?)
            } else {
                Err(self.error())
            }
        } else {
            Err(self.error())
        }
    }

    fn option(&mut self) -> Result<Option<A>, E> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(Some(one?))
            } else {
                Err(self.error())
            }
        } else {
            Ok(None)
        }
    }

    fn many(&mut self) -> Result<Vec<A>, E> {
        self.collect::<Result<Vec<_>, _>>()
    }

    fn error(&self) -> E;
}

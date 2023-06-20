use std::result::Result;

/// An occurrence API for an iterator that does not return results.
pub trait Occurrence<A, E>: Iterator<Item = A> {
    fn one(&mut self) -> Result<A, E> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(one)
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
                Ok(Some(one))
            } else {
                Err(self.error())
            }
        } else {
            Ok(None)
        }
    }
    fn many(&mut self) -> Vec<A> {
        self.collect::<Vec<_>>()
    }

    fn error(&self) -> E;
}

/// An occurrence API for an iterator that returns results.
pub trait ResultOccurrence<A, E>: Iterator<Item = Result<A, E>> {
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

use crate::{error, occurrence};

use super::Item;

// impl<T> occurrence::Occurrence<Item, error::Error> for T
// where
//     T: Iterator<Item = Item>,
// {
//     fn one(&mut self) -> Result<Item, error::Error> {
//         if let Some(one) = self.next() {
//             if self.next().is_none() {
//                 Ok(one)
//             } else {
//                 Err(self.error())
//             }
//         } else {
//             Err(self.error())
//         }
//     }

//     fn option(&mut self) -> Result<Option<Item>, error::Error> {
//         if let Some(one) = self.next() {
//             if self.next().is_none() {
//                 Ok(Some(one))
//             } else {
//                 Err(self.error())
//             }
//         } else {
//             Ok(None)
//         }
//     }

//     fn many(&mut self) -> Result<Vec<Item>, error::Error> {
//         Ok(self.collect::<Vec<_>>())
//     }

//     fn error(&self) -> error::Error {
//         error::Error::XPTY0004
//     }
// }

impl<V, U> occurrence::Occurrence<V, error::Error> for U
where
    V: std::fmt::Debug,
    U: Iterator<Item = error::Result<V>>,
{
    fn one(&mut self) -> Result<V, error::Error> {
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

    fn option(&mut self) -> Result<Option<V>, error::Error> {
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

    fn many(&mut self) -> Result<Vec<V>, error::Error> {
        self.collect::<Result<Vec<_>, _>>()
    }

    fn error(&self) -> error::Error {
        error::Error::XPTY0004
    }
}

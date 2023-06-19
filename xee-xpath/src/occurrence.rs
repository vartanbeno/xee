use crate::error;

pub trait Occurrence {
    type Item;

    fn one(&mut self) -> error::Result<Self::Item>;
    fn option(&mut self) -> error::Result<Option<Self::Item>>;
    fn many(&mut self) -> error::Result<Vec<Self::Item>>;
}

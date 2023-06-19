use std::result::Result;

pub trait Occurrence {
    type Item;
    type Error;

    fn one(&mut self) -> Result<Self::Item, Self::Error>;
    fn option(&mut self) -> Result<Option<Self::Item>, Self::Error>;
    fn many(&mut self) -> Result<Vec<Self::Item>, Self::Error>;
}

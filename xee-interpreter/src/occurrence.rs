use std::result::Result;

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

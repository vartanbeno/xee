use chumsky::{extra::Full, prelude::*};
use std::borrow::Cow;

use crate::lexer::Token;
use crate::namespaces::Namespaces;

pub(crate) struct State<'a> {
    pub(crate) namespaces: Cow<'a, Namespaces<'a>>,
}

type Extra<'a, T> = Full<Rich<'a, T>, State<'a>, ()>;

pub(crate) type BoxedParser<'a, I, T> = Boxed<'a, 'a, I, T, Extra<'a, Token<'a>>>;

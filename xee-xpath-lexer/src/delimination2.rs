use std::iter::Peekable;

use logos::{Logos, Span, SpannedIter};

use crate::symbol_type::SymbolType;
use crate::{collapse_whitespace::CollapseWhitespace, Token};

pub(crate) struct DeliminationIterator<'a> {
    base: Peekable<CollapseWhitespace<'a>>,
}

impl<'a> DeliminationIterator<'a> {
    pub(crate) fn new(base: CollapseWhitespace<'a>) -> Self {
        Self {
            base: base.peekable(),
        }
    }

    pub(crate) fn from_spanned(spanned_iter: SpannedIter<'a, Token<'a>>) -> Self {
        let base = CollapseWhitespace::from_spanned(spanned_iter);
        Self::new(base)
    }

    pub(crate) fn from_str(input: &'a str) -> Self {
        let spanned_lexer = Token::lexer(input).spanned();
        Self::from_spanned(spanned_lexer)
    }
}

impl<'a> Iterator for DeliminationIterator<'a> {
    type Item = (Token<'a>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let (token, span) = self.base.next()?;
        match token.symbol_type2() {
            SymbolType::NonDelimiting => {
                // if we are not followed by either a delimiting symbol or a symbol separator, or are
                // at the end we are in error
                let next = self.base.peek();
                if let Some((next_token, _)) = next {
                    match next_token.symbol_type2() {
                        SymbolType::Delimiting
                        | SymbolType::SymbolSeparator
                        | SymbolType::Error => {
                            // a non-delimiting symbol can be followed by these without error
                            Some((token, span))
                        }
                        SymbolType::NonDelimiting => {
                            // a non-delimiting symbol may not be followed by another one,
                            // so this is an error
                            Some((Token::Error, span))
                        }
                    }
                } else {
                    // if we are at the end we're delimited
                    Some((token, span))
                }
            }
            SymbolType::Delimiting | SymbolType::Error => Some((token, span)),
            SymbolType::SymbolSeparator => {
                // we suppress symbol separators so return the next token
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_delimination() {
    //     let base = CollapseWhitespace::new("a  b".logos());
    //     let mut delimination = DeliminationIterator::new(base);

    //     assert_eq!(delimination.next(), Some((Token::NCName("a"), 0..1)));
    //     assert_eq!(delimination.next(), Some((Token::Whitespace, 1..3)));
    //     assert_eq!(delimination.next(), Some((Token::NCName("b"), 3..4)));
    //     assert_eq!(delimination.next(), None);
    // }
}

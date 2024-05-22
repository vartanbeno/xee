use itertools::{Itertools, MultiPeek};
use logos::{Span, SpannedIter};

use crate::{lexer::PrefixedQName, Token};

pub(crate) struct PrefixedQNameIterator<'a> {
    base: MultiPeek<SpannedIter<'a, Token<'a>>>,
}

impl<'a> PrefixedQNameIterator<'a> {
    pub(crate) fn new(spanned_iter: SpannedIter<'a, Token<'a>>) -> Self {
        let base = spanned_iter.multipeek();
        Self { base }
    }

    fn prefixed_qname<'b>(
        &mut self,
        token: &'b Token<'a>,
        span: &'b Span,
    ) -> Option<(PrefixedQName<'a>, Span)> {
        if let Token::NCName(prefix) = token {
            // if we are followed by a token
            if let Some((Ok(Token::Colon), _)) = self.base.peek() {
                // and then an ncname
                if let Some((Ok(Token::NCName(local_name)), local_name_span)) = self.base.peek() {
                    // we create a span from the start of the original prefix span
                    // to the end of the localname span
                    let span = span.start..local_name_span.end;
                    return Some((PrefixedQName { prefix, local_name }, span));
                }
            }
        }
        None
    }
}

impl<'a> Iterator for PrefixedQNameIterator<'a> {
    type Item = (Result<Token<'a>, ()>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        // if we find a ncname, we peek two tokens ahead to determine
        // whether we find a colon and a ncname. If so, we absorb the three
        // tokens and produce a prefixed qname token. If not, we just produce
        // the token from the base iterator.
        let (token, span) = self.base.next()?;
        if let Ok(token) = token {
            if let Some((prefixed_qname, prefixed_qname_span)) = self.prefixed_qname(&token, &span)
            {
                // consume two next tokens
                self.base.next();
                self.base.next();
                return Some((
                    Ok(Token::PrefixedQName(prefixed_qname)),
                    prefixed_qname_span,
                ));
            } else {
                Some((Ok(token), span))
            }
        } else {
            Some((Err(()), span))
        }
    }
}

#[cfg(test)]
mod tests {
    use ibig::ibig;
    use logos::Logos;

    use crate::delimination::XPathLexer;

    use super::*;

    fn spanned_lexer(input: &str) -> SpannedIter<Token> {
        Token::lexer(input).spanned()
    }

    #[test]
    fn test_no_ncname_no_prefixed_qname() {
        let lex = spanned_lexer("1 + 1");
        let mut iter = PrefixedQNameIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), 0..1))
        );
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 1..2)));
        assert_eq!(iter.next(), Some((Ok(Token::Plus), 2..3)));
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 3..4)));
        assert_eq!(
            iter.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), 4..5))
        );
    }

    #[test]
    fn test_ncname_no_prefixed_qname() {
        let lex = spanned_lexer("foo + 1");
        let mut iter = PrefixedQNameIterator::new(lex);
        assert_eq!(iter.next(), Some((Ok(Token::NCName("foo")), 0..3)));
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 3..4)));
        assert_eq!(iter.next(), Some((Ok(Token::Plus), 4..5)));
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 5..6)));
        assert_eq!(
            iter.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), 6..7))
        );
    }

    #[test]
    fn test_ncname_colon_no_prefixed_qname() {
        let lex = spanned_lexer("foo: + 1");
        let mut iter = PrefixedQNameIterator::new(lex);
        assert_eq!(iter.next(), Some((Ok(Token::NCName("foo")), 0..3)));
        assert_eq!(iter.next(), Some((Ok(Token::Colon), 3..4)));
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 4..5)));
        assert_eq!(iter.next(), Some((Ok(Token::Plus), 5..6)));
        assert_eq!(iter.next(), Some((Ok(Token::Whitespace), 6..7)));
        assert_eq!(
            iter.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), 7..8))
        );
    }

    #[test]
    fn test_prefixed_qname() {
        let lex = spanned_lexer("foo:bar");
        let mut iter = PrefixedQNameIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Ok(Token::PrefixedQName(PrefixedQName {
                    prefix: "foo",
                    local_name: "bar"
                })),
                0..7
            ))
        );
        assert_eq!(iter.next(), None);
    }
}

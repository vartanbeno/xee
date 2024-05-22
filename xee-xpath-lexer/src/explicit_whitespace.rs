use itertools::{Itertools, MultiPeek};
use logos::{Span, SpannedIter};

use crate::{
    lexer::{
        BracedURILiteralWildcard, LocalNameWildcard, PrefixWildcard, PrefixedQName,
        URIQualifiedName,
    },
    Token,
};

// this iterator is used to combine multiple tokens into a prefixed qname, uri qualified name,
// prefix wildcard, local name wildcard, uri qualified name wildcard.
// these tokens cannot have whitespace inside, but by the time tokens reach the
// parser all whitespace has been elimited, so we cannot use the parser to enforce this.
// we therefore combine them early here, where we can still see the whitespace.
// this iterator also turns any errors into error tokens
pub(crate) struct ExplicitWhitespaceIterator<'a> {
    base: MultiPeek<SpannedIter<'a, Token<'a>>>,
}

impl<'a> ExplicitWhitespaceIterator<'a> {
    pub(crate) fn new(spanned_iter: SpannedIter<'a, Token<'a>>) -> Self {
        let base = spanned_iter.multipeek();
        Self { base }
    }

    pub(crate) fn peek(&mut self) -> Option<(&Result<Token<'a>, ()>, Span)> {
        self.base.peek().map(|(token, span)| (token, span.clone()))
    }

    // if we find a qname, this could be merged into a prefixed qname, or a
    // local name wildcard.
    fn qname_prefix<'b>(&mut self, name: &'a str, span: &'b Span) -> Option<(Token<'a>, Span)> {
        let (next_token, next_span) = self.base.peek()?;

        match next_token {
            // if we are followed by a colon token, this may be a prefixed qname
            Ok(Token::Colon) => {
                // and then an ncname
                if let Some((Ok(Token::NCName(local_name)), local_name_span)) = self.base.peek() {
                    // we create a span from the start of the original prefix span
                    // to the end of the localname span
                    let span = span.start..local_name_span.end;
                    return Some((
                        Token::PrefixedQName(PrefixedQName {
                            prefix: name,
                            local_name,
                        }),
                        span,
                    ));
                }
                None
            }
            // if we are followed by a ColonAsterisk token, this is a local name wildcard
            Ok(Token::ColonAsterisk) => {
                let span = span.start..next_span.end;
                return Some((
                    Token::LocalNameWildcard(LocalNameWildcard { prefix: name }),
                    span,
                ));
            }
            Ok(_) => {
                return None;
            }
            Err(_) => {
                return Some((Token::Error, span.clone()));
            }
        }
    }

    fn braced_uri_literal_prefix<'b>(
        &mut self,
        uri: &'a str,
        span: &'b Span,
    ) -> Option<(Token<'a>, Span)> {
        let (next_token, next_span) = self.base.peek()?;
        match next_token {
            Ok(Token::NCName(local_name)) => {
                let span = span.start..next_span.end;
                return Some((
                    Token::URIQualifiedName(URIQualifiedName { uri, local_name }),
                    span,
                ));
            }
            Ok(Token::Asterisk) => {
                let span = span.start..next_span.end;
                return Some((
                    Token::BracedURILiteralWildcard(BracedURILiteralWildcard { uri }),
                    span,
                ));
            }
            Err(_) => {
                return Some((Token::Error, span.clone()));
            }
            _ => None,
        }
    }

    fn prefix_wildcard(&mut self, span: &Span) -> Option<(Token<'a>, Span)> {
        let (next_token, next_span) = self.base.peek()?;
        match next_token {
            Ok(Token::NCName(local_name)) => {
                let span = span.start..next_span.end;
                return Some((Token::PrefixWildcard(PrefixWildcard { local_name }), span));
            }
            _ => None,
        }
    }
}

impl<'a> Iterator for ExplicitWhitespaceIterator<'a> {
    type Item = (Token<'a>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let (token, span) = self.base.next()?;
        if let Ok(token) = token {
            match token {
                // if we have an ncname, it may be merged into either a prefixed qname or
                // a local name wildcard
                Token::NCName(name) => {
                    if let Some((token, span)) = self.qname_prefix(name, &span) {
                        match token {
                            Token::PrefixedQName(_) => {
                                // consume two next tokens
                                self.base.next();
                                self.base.next();
                            }
                            Token::LocalNameWildcard(_) => {
                                // consume next token
                                self.base.next();
                            }
                            _ => {}
                        }
                        return Some((token, span));
                    }
                    Some((token, span))
                }
                // if we have asterisk colon it may be merged into a prefix wildcard
                Token::AsteriskColon => {
                    if let Some((token, span)) = self.prefix_wildcard(&span) {
                        // consume next token
                        self.base.next();
                        return Some((token, span));
                    }
                    Some((token, span))
                }
                // if we have a braced URI literal it may be either a URI qualified name
                // or a braced uri literal wildcard
                Token::BracedURILiteral(uri) => {
                    if let Some((token, span)) = self.braced_uri_literal_prefix(uri, &span) {
                        // consume next token
                        self.base.next();
                        return Some((token, span));
                    }
                    Some((token, span))
                }
                _ => Some((token, span)),
            }
        } else {
            Some((Token::Error, span))
        }
    }
}

#[cfg(test)]
mod tests {
    use ibig::ibig;
    use logos::Logos;

    use super::*;

    fn spanned_lexer(input: &str) -> SpannedIter<Token> {
        Token::lexer(input).spanned()
    }

    #[test]
    fn test_no_ncname_no_prefixed_qname() {
        let lex = spanned_lexer("1 + 1");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(iter.next(), Some((Token::IntegerLiteral(ibig!(1)), 0..1)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 1..2)));
        assert_eq!(iter.next(), Some((Token::Plus, 2..3)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(iter.next(), Some((Token::IntegerLiteral(ibig!(1)), 4..5)));
    }

    #[test]
    fn test_ncname_no_prefixed_qname() {
        let lex = spanned_lexer("foo + 1");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(iter.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(iter.next(), Some((Token::Plus, 4..5)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 5..6)));
        assert_eq!(iter.next(), Some((Token::IntegerLiteral(ibig!(1)), 6..7)));
    }

    #[test]
    fn test_ncname_colon_no_prefixed_qname() {
        let lex = spanned_lexer("foo: + 1");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(iter.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(iter.next(), Some((Token::Colon, 3..4)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 4..5)));
        assert_eq!(iter.next(), Some((Token::Plus, 5..6)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 6..7)));
        assert_eq!(iter.next(), Some((Token::IntegerLiteral(ibig!(1)), 7..8)));
    }

    #[test]
    fn test_prefixed_qname() {
        let lex = spanned_lexer("foo:bar");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::PrefixedQName(PrefixedQName {
                    prefix: "foo",
                    local_name: "bar"
                }),
                0..7
            ))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_prefixed_qname_followed() {
        let lex = spanned_lexer("foo:bar + 1");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::PrefixedQName(PrefixedQName {
                    prefix: "foo",
                    local_name: "bar"
                }),
                0..7
            ))
        );
        assert_eq!(iter.next(), Some((Token::Whitespace, 7..8)));
        assert_eq!(iter.next(), Some((Token::Plus, 8..9)));
        assert_eq!(iter.next(), Some((Token::Whitespace, 9..10)));
        assert_eq!(iter.next(), Some((Token::IntegerLiteral(ibig!(1)), 10..11)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_local_name_wildcard() {
        let lex = spanned_lexer("foo:*");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::LocalNameWildcard(LocalNameWildcard { prefix: "foo" }),
                0..5
            ))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_prefix_wilcard() {
        let lex = spanned_lexer("*:bar");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::PrefixWildcard(PrefixWildcard { local_name: "bar" }),
                0..5
            ))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_uri_qualified_name() {
        let lex = spanned_lexer("Q{http://example.com}bar");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::URIQualifiedName(URIQualifiedName {
                    uri: "http://example.com",
                    local_name: "bar"
                }),
                0..24
            ))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_braced_uri_literal_wildcard() {
        let lex = spanned_lexer("Q{http://example.com}*");
        let mut iter = ExplicitWhitespaceIterator::new(lex);
        assert_eq!(
            iter.next(),
            Some((
                Token::BracedURILiteralWildcard(BracedURILiteralWildcard {
                    uri: "http://example.com"
                }),
                0..22
            ))
        );
        assert_eq!(iter.next(), None);
    }
}

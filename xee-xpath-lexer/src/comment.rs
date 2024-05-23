// an iterator that replaces comments, potentially nested, with a single
// whitespace token (with the same span of what is replaced).
use logos::{Logos, Span, SpannedIter};

use crate::{explicit_whitespace::ExplicitWhitespaceIterator, Token};

pub(crate) struct ReplaceCommentWithWhitespace<'a> {
    base: ExplicitWhitespaceIterator<'a>,
}

impl<'a> ReplaceCommentWithWhitespace<'a> {
    pub(crate) fn new(base: ExplicitWhitespaceIterator<'a>) -> Self {
        Self { base }
    }

    pub(crate) fn from_spanned(spanned_iter: SpannedIter<'a, Token<'a>>) -> Self {
        let base = ExplicitWhitespaceIterator::new(spanned_iter);
        Self::new(base)
    }

    pub(crate) fn from_str(input: &'a str) -> Self {
        let spanned_lexer = Token::lexer(input).spanned();
        Self::from_spanned(spanned_lexer)
    }
}

impl<'a> Iterator for ReplaceCommentWithWhitespace<'a> {
    type Item = (Token<'a>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let (token, span) = self.base.next()?;
        match token {
            Token::CommentStart => {
                let mut depth = 1;
                // we track the span from the start of the first
                // comment start
                let start = span.start;
                let mut end = span.end;
                // now we find the commend end that matches,
                // taking into account nested comments
                // we track the end of the span of what we
                // found next, so that we can report it in
                // case of errors
                while depth > 0 {
                    match self.base.next() {
                        Some((Token::CommentStart, span)) => {
                            end = span.end;
                            depth += 1
                        }
                        Some((Token::CommentEnd, span)) => {
                            end = span.end;
                            depth -= 1;
                            // comments are balanced, so done
                            if depth == 0 {
                                break;
                            }
                        }
                        Some((_, span)) => {
                            end = span.end;
                        }
                        // if we reach the end and things are unclosed,
                        // we bail out with an error
                        None => {
                            return Some((Token::Error, start..end));
                        }
                    }
                }
                Some((Token::Whitespace, start..end))
            }
            Token::CommentEnd => {
                // we should never see a comment end without a start
                Some((Token::Error, span))
            }
            _ => {
                // we just pass through anything that is not a comment
                Some((token, span))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_comment() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo (: bar :) baz");

        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..13)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 13..14)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 14..17)));
    }

    #[test]
    fn test_single_comment_with_expression_content() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo (: 1 + 2 :) baz");
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..15)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 15..16)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 16..19)));
    }

    #[test]
    fn test_nested_comment() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo (: bar (: baz :) quux :) baz");
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..28)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 28..29)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 29..32)));
    }

    #[test]
    fn test_nested_comment_unbalanced() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo (: bar (: quux :) baz");
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..25)));
    }

    #[test]
    fn test_unclosed_comment() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo (: bar");
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..10)));
    }

    #[test]
    fn test_closed_comment_without_opening() {
        let mut lexer = ReplaceCommentWithWhitespace::from_str("foo :) bar");
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..6)));
    }
}

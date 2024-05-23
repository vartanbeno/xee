// an iterator that replaces comments, potentially nested, with a single
// whitespace token (with the same span of what is replaced).
use logos::Span;

use crate::{explicit_whitespace::ExplicitWhitespaceIterator, Token};

struct ReplaceCommentWithWhitespaceIterator<'a> {
    base: ExplicitWhitespaceIterator<'a>,
}

impl<'a> ReplaceCommentWithWhitespaceIterator<'a> {
    pub(crate) fn new(base: ExplicitWhitespaceIterator<'a>) -> Self {
        Self { base }
    }
}

impl<'a> Iterator for ReplaceCommentWithWhitespaceIterator<'a> {
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
    use logos::Logos;

    use super::*;

    fn base_lexer(input: &str) -> ExplicitWhitespaceIterator {
        let spanned_lexer = Token::lexer(input).spanned();
        ExplicitWhitespaceIterator::new(spanned_lexer)
    }

    #[test]
    fn test_single_comment() {
        let explicit_whitespace = base_lexer("foo (: bar :) baz");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..13)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 13..14)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 14..17)));
    }

    #[test]
    fn test_single_comment_with_expression_content() {
        let explicit_whitespace = base_lexer("foo (: 1 + 2 :) baz");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..15)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 15..16)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 16..19)));
    }

    #[test]
    fn test_nested_comment() {
        let explicit_whitespace = base_lexer("foo (: bar (: baz :) quux :) baz");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 4..28)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 28..29)));
        assert_eq!(lexer.next(), Some((Token::NCName("baz"), 29..32)));
    }

    #[test]
    fn test_nested_comment_unbalanced() {
        let explicit_whitespace = base_lexer("foo (: bar (: quux :) baz");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..25)));
    }

    #[test]
    fn test_unclosed_comment() {
        let explicit_whitespace = base_lexer("foo (: bar");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..10)));
    }

    #[test]
    fn test_closed_comment_without_opening() {
        let explicit_whitespace = base_lexer("foo :) bar");
        let mut lexer = ReplaceCommentWithWhitespaceIterator::new(explicit_whitespace);
        assert_eq!(lexer.next(), Some((Token::NCName("foo"), 0..3)));
        assert_eq!(lexer.next(), Some((Token::Whitespace, 3..4)));
        assert_eq!(lexer.next(), Some((Token::Error, 4..6)));
    }
}

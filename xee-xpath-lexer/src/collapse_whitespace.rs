use itertools::{Itertools, MultiPeek};
use logos::Span;

use crate::{comment::ReplaceCommentWithWhitespaceIterator, Token};

struct CollapseWhitespace<'a> {
    base: MultiPeek<ReplaceCommentWithWhitespaceIterator<'a>>,
}

impl<'a> CollapseWhitespace<'a> {
    pub(crate) fn new(base: ReplaceCommentWithWhitespaceIterator<'a>) -> Self {
        Self {
            base: base.multipeek(),
        }
    }
}

impl<'a> Iterator for CollapseWhitespace<'a> {
    type Item = (Token<'a>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let (token, span) = self.base.next()?;
        match token {
            Token::Whitespace => {
                let start = span.start;
                let mut end = span.end;
                // peek ahead to see if there's more whitespace and collapse it
                let mut whitespace_peeked = 0;
                while let Some((Token::Whitespace, next_span)) = self.base.peek() {
                    end = next_span.end;
                    whitespace_peeked += 1;
                }
                // now eat the whitespace
                for _ in 0..whitespace_peeked {
                    println!("eat whitespace");
                    self.base.next();
                }
                Some((Token::Whitespace, start..end))
            }
            _ => Some((token, span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::explicit_whitespace::ExplicitWhitespaceIterator;

    use super::*;

    use logos::Logos;

    fn base_lexer(input: &str) -> ReplaceCommentWithWhitespaceIterator {
        let spanned_lexer = Token::lexer(input).spanned();
        let explicit = ExplicitWhitespaceIterator::new(spanned_lexer);
        ReplaceCommentWithWhitespaceIterator::new(explicit)
    }

    #[test]
    fn test_collapse_whitespace() {
        let base = base_lexer("a  b");
        let mut collapse_whitespace = CollapseWhitespace::new(base);

        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("a"), 0..1)));
        assert_eq!(collapse_whitespace.next(), Some((Token::Whitespace, 1..3)));
        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("b"), 3..4)));
        assert_eq!(collapse_whitespace.next(), None);
    }

    #[test]
    fn test_collapse_whitespace_with_comment() {
        let base = base_lexer("a (: comment :) b");
        let mut collapse_whitespace = CollapseWhitespace::new(base);

        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("a"), 0..1)));
        assert_eq!(collapse_whitespace.next(), Some((Token::Whitespace, 1..16)));
        assert_eq!(
            collapse_whitespace.next(),
            Some((Token::NCName("b"), 16..17))
        );
        assert_eq!(collapse_whitespace.next(), None);
    }
}

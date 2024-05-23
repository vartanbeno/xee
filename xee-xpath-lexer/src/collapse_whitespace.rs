use itertools::{Itertools, MultiPeek};
use logos::{Logos, Span, SpannedIter};

use crate::{comment::ReplaceCommentWithWhitespaceIterator, Token};

pub(crate) struct CollapseWhitespace<'a> {
    base: MultiPeek<ReplaceCommentWithWhitespaceIterator<'a>>,
}

impl<'a> CollapseWhitespace<'a> {
    pub(crate) fn new(base: ReplaceCommentWithWhitespaceIterator<'a>) -> Self {
        Self {
            base: base.multipeek(),
        }
    }

    pub(crate) fn from_spanned(spanned_iter: SpannedIter<'a, Token<'a>>) -> Self {
        let base = ReplaceCommentWithWhitespaceIterator::from_spanned(spanned_iter);
        Self::new(base)
    }

    pub(crate) fn from_str(input: &'a str) -> Self {
        let spanned_lexer = Token::lexer(input).spanned();
        Self::from_spanned(spanned_lexer)
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
    use super::*;

    #[test]
    fn test_collapse_whitespace() {
        let mut collapse_whitespace = CollapseWhitespace::from_str("a  b");

        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("a"), 0..1)));
        assert_eq!(collapse_whitespace.next(), Some((Token::Whitespace, 1..3)));
        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("b"), 3..4)));
        assert_eq!(collapse_whitespace.next(), None);
    }

    #[test]
    fn test_collapse_whitespace_with_comment() {
        let mut collapse_whitespace = CollapseWhitespace::from_str("a (: comment :) b");

        assert_eq!(collapse_whitespace.next(), Some((Token::NCName("a"), 0..1)));
        assert_eq!(collapse_whitespace.next(), Some((Token::Whitespace, 1..16)));
        assert_eq!(
            collapse_whitespace.next(),
            Some((Token::NCName("b"), 16..17))
        );
        assert_eq!(collapse_whitespace.next(), None);
    }
}

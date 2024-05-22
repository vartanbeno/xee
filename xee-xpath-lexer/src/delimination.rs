use logos::{Lexer, Logos, Span, SpannedIter};

use crate::{lexer::SymbolType, Token};

enum LastTerminal {
    NCName,
    NumericLiteral,
    Dot,
    Other,
}

pub struct XPathLexer<'a> {
    spanned: SpannedIter<'a, Token<'a>>,
    last_is_separator: bool,
    last_is_non_delimiting: bool,
    last_terminal: LastTerminal,
    forbid_whitespace: bool,
}

impl<'a> XPathLexer<'a> {
    pub(crate) fn new(lexer: Lexer<'a, Token<'a>>) -> Self {
        let spanned = lexer.spanned();
        Self {
            spanned,
            last_is_separator: true,
            last_is_non_delimiting: false,
            forbid_whitespace: false,
            last_terminal: LastTerminal::Other,
        }
    }
}

impl<'a> Iterator for XPathLexer<'a> {
    type Item = (Result<Token<'a>, ()>, Span);

    // A.2.2 Terminal Delimination
    fn next(&mut self) -> Option<Self::Item> {
        let token_span = self.spanned.next()?;
        match &token_span {
            (token, span) => match token {
                Ok(token) => {
                    use SymbolType::*;
                    match token.symbol_type() {
                        Delimiting => {
                            match token {
                                Token::AsteriskColon | Token::BracedURILiteral(_) => {
                                    self.forbid_whitespace = true;
                                }
                                _ => {
                                    self.forbid_whitespace = false;
                                }
                            }
                            // if T is an NCName and U is "-" or ".", then the
                            // lexer will absorb the "-" and "." at the end of
                            // the ncname. This is a valid NCName and should be
                            // accepted.

                            // We still need to handle the case where a dot
                            // appears after a numeric literal
                            if matches!(token, Token::Dot) {
                                if self.last_is_non_delimiting
                                    && !self.last_is_separator
                                    && matches!(self.last_terminal, LastTerminal::NumericLiteral)
                                {
                                    return Some((Err(()), span.clone()));
                                }
                                self.last_terminal = LastTerminal::Dot;
                            }
                            self.last_is_separator = false;
                            self.last_is_non_delimiting = false;
                            Some(token_span)
                        }
                        NonDelimiting => {
                            match token {
                                Token::NCName(_) => {
                                    self.last_terminal = LastTerminal::NCName;
                                }
                                Token::IntegerLiteral(_)
                                | Token::DecimalLiteral(_)
                                | Token::DoubleLiteral(_) => {
                                    // vice versa: T is a "." and U is a numeric literal
                                    // Checking thatextra.state isn't necessary, as a leading
                                    // dot will automatically be interpreted as starting
                                    // a decimal or double. The vice versa rule is there
                                    // for disambiguation only
                                    self.last_terminal = LastTerminal::NumericLiteral;
                                }
                                _ => {}
                            }
                            // if we have seen a non-delimiting last,
                            let r = if self.last_is_non_delimiting {
                                // then there has to be a separator, or
                                // it's an error
                                if self.last_is_separator {
                                    Some(token_span)
                                } else {
                                    Some((Err(()), span.clone()))
                                }
                            } else {
                                // if we've seen delimiting last, we're fine
                                Some(token_span)
                            };
                            self.last_is_separator = false;
                            self.last_is_non_delimiting = true;
                            self.forbid_whitespace = false;
                            r
                        }
                        Whitespace => {
                            if self.forbid_whitespace {
                                return Some((Err(()), span.clone()));
                            }
                            self.forbid_whitespace = false;
                            self.last_is_separator = true;
                            self.next()
                        }
                        CommentStart => {
                            if self.forbid_whitespace {
                                return Some((Err(()), span.clone()));
                            }
                            self.forbid_whitespace = false;
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
                                match self.spanned.next() {
                                    Some((Ok(Token::CommentStart), span)) => {
                                        end = span.end;
                                        depth += 1
                                    }
                                    Some((Ok(Token::CommentEnd), span)) => {
                                        end = span.end;
                                        depth -= 1;
                                        // comments are balanced, so done
                                        if depth == 0 {
                                            break;
                                        }
                                    }
                                    // If we run into a non-comment, we skip it
                                    Some((_, span)) => {
                                        end = span.end;
                                    }
                                    // if we reach the end and things are unclosed,
                                    // we bail out
                                    None => {
                                        return Some((Err(()), start..end));
                                    }
                                }
                            }
                            self.last_is_separator = true;
                            self.next()
                        }
                        CommentEnd => Some(token_span),
                    }
                }
                Err(_) => Some(token_span),
            },
        }
    }
}

pub fn lexer(input: &str) -> XPathLexer {
    XPathLexer::new(Token::lexer(input))
}

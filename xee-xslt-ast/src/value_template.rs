use crate::ast_core::Span;

struct ValueTemplateTokenizer<'a> {
    s: &'a str,
    char_indices: std::iter::Peekable<std::str::CharIndices<'a>>,
    span: Span,
    mode: Mode,
    start: usize,
    done: bool,
}

enum Mode {
    String,
    Value,
    StartCurly,
    EndCurly,
}

impl<'a> ValueTemplateTokenizer<'a> {
    fn new(s: &'a str, span: Span) -> Self {
        Self {
            s,
            char_indices: s.char_indices().peekable(),
            span,
            mode: Mode::String,
            start: 0,
            done: false,
        }
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span {
            start: self.span.start + start,
            end: self.span.start + end,
        }
    }

    fn string_item(
        &mut self,
        start: usize,
        end: usize,
    ) -> Option<Result<ValueTemplateItem<'a>, Error>> {
        if let Some(text) = self.s.get(start..end) {
            if text.is_empty() {
                return self.next();
            }
            Some(Ok(ValueTemplateItem::String {
                text,
                span: self.span(start, end),
            }))
        } else {
            Some(Err(Error::IllegalSlice))
        }
    }

    fn value_item(&self, start: usize, end: usize) -> Result<ValueTemplateItem<'a>, Error> {
        if let Some(text) = self.s.get(start..end) {
            Ok(ValueTemplateItem::Value {
                text,
                span: self.span(start, end),
            })
        } else {
            Err(Error::IllegalSlice)
        }
    }

    fn start_curly_item(&self) -> Result<ValueTemplateItem<'a>, Error> {
        Ok(ValueTemplateItem::Curly { c: '{' })
    }

    fn end_curly_item(&self) -> Result<ValueTemplateItem<'a>, Error> {
        Ok(ValueTemplateItem::Curly { c: '}' })
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
enum ValueTemplateItem<'a> {
    String { text: &'a str, span: Span },
    Curly { c: char },
    Value { text: &'a str, span: Span },
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
enum Error {
    UnescapedCurly { c: char, span: Span },
    IllegalSlice,
}

impl<'a> Iterator for ValueTemplateTokenizer<'a> {
    type Item = Result<ValueTemplateItem<'a>, Error>;

    fn next(&mut self) -> Option<Result<ValueTemplateItem<'a>, Error>> {
        if self.done {
            return None;
        }
        let start = self.start;
        let end;
        match self.mode {
            Mode::String => loop {
                if let Some((i, c)) = self.char_indices.next() {
                    match c {
                        '{' => {
                            self.mode = Mode::StartCurly;
                            end = i;
                            self.start = end + 1;
                            return self.string_item(start, end);
                        }
                        '}' => {
                            self.mode = Mode::EndCurly;
                            end = i;
                            self.start = end + 1;
                            return self.string_item(start, end);
                        }
                        _ => {
                            continue;
                        }
                    }
                } else {
                    end = self.s.len();
                    self.done = true;
                    return self.string_item(start, end);
                }
            },
            Mode::Value => loop {
                // TODO: this should parse an XPath expression
                // with the following string and return the index
                // of the end of the expression, which should contain
                // a close curly brace or be an error
                // this should be fed into the lexer in an iterative way

                if let Some((i, c)) = self.char_indices.next() {
                    if c == '}' {
                        self.mode = Mode::String;
                        end = i;
                        self.start = end + 1;
                        return Some(self.value_item(start, end));
                    }
                } else {
                    self.done = true;
                    return Some(Err(Error::UnescapedCurly {
                        c: '{',
                        span: Span {
                            start: self.span.start + self.start,
                            end: self.span.start + self.start + 1,
                        },
                    }));
                }
            },
            Mode::StartCurly => {
                if let Some((_peek_i, peek_c)) = self.char_indices.peek() {
                    if *peek_c == '{' {
                        if let Some((i, _c)) = self.char_indices.next() {
                            self.mode = Mode::String;
                            end = i + 1;
                            self.start = end;
                            return Some(self.start_curly_item());
                        } else {
                            unreachable!();
                        }
                    }
                }
                self.mode = Mode::Value;
                self.next()
            }
            Mode::EndCurly => {
                if let Some((_peek_i, peek_c)) = self.char_indices.peek() {
                    if *peek_c == '}' {
                        if let Some((i, _c)) = self.char_indices.next() {
                            self.mode = Mode::String;
                            end = i + 1;
                            self.start = end;
                            return Some(self.end_curly_item());
                        } else {
                            unreachable!();
                        }
                    }
                }
                self.done = true;
                Some(Err(Error::UnescapedCurly {
                    c: '}',
                    span: Span {
                        start: self.span.start + self.start,
                        end: self.span.start + self.start + 1,
                    },
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_ron_snapshot;

    fn parse_with_span(s: &str, span: Span) -> Result<Vec<ValueTemplateItem>, Error> {
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        tokenizer.collect()
    }

    fn parse(s: &str) -> Result<Vec<ValueTemplateItem>, Error> {
        let span = Span {
            start: 0,
            end: s.len(),
        };
        parse_with_span(s, span)
    }

    #[test]
    fn test_string_without_curly() {
        assert_ron_snapshot!(parse("hello world"));
    }

    #[test]
    fn test_string_start_curly_escaped() {
        assert_ron_snapshot!(parse("hello{{world"));
    }

    #[test]
    fn test_string_end_curly_escaped() {
        assert_ron_snapshot!(parse("hello}}world"));
    }

    #[test]
    fn test_string_with_value() {
        assert_ron_snapshot!(parse("hello {world}!"));
    }

    #[test]
    fn test_string_with_value_in_span() {
        let s = "hello {world}!";
        let span = Span {
            start: 10,
            end: s.len() + 10,
        };
        assert_ron_snapshot!(parse_with_span(s, span));
    }

    #[test]
    fn test_string_with_empty_value() {
        assert_ron_snapshot!(parse("hello {}!"));
    }

    #[test]
    fn test_string_with_multiple_values() {
        assert_ron_snapshot!(parse("hello {a} and {b}!"));
    }

    #[test]
    fn test_string_with_multiple_adjacent_values() {
        assert_ron_snapshot!(parse("hello {a}{b}!"));
    }

    #[test]
    fn test_string_unescaped_unclosed_start_curly() {
        assert_ron_snapshot!(parse("hello{world"));
    }

    #[test]
    fn test_string_unescaped_unclosed_start_curly_with_span() {
        let s = "hello{world";
        let span = Span {
            start: 10,
            end: 10 + s.len(),
        };
        assert_ron_snapshot!(parse_with_span(s, span));
    }

    #[test]
    fn test_string_unescaped_end_curly() {
        assert_ron_snapshot!(parse("hello}world"));
    }
}

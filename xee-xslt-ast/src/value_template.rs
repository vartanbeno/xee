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

    fn item(
        &self,
        start: usize,
        end: usize,
        f: impl Fn(&'a str, Span) -> ValueTemplateItem<'a>,
    ) -> Result<ValueTemplateItem<'a>, Error> {
        if let Some(text) = self.s.get(start..end) {
            let span = Span {
                start: self.span.start + start,
                end: self.span.start + end,
            };
            Ok(f(text, span))
        } else {
            Err(Error::IllegalSlice)
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
            let span = Span {
                start: self.span.start + start,
                end: self.span.start + end,
            };
            Some(Ok(ValueTemplateItem::String { text, span }))
        } else {
            Some(Err(Error::IllegalSlice))
        }
    }

    fn value_item(&self, start: usize, end: usize) -> Result<ValueTemplateItem<'a>, Error> {
        self.item(start, end, |text, span| ValueTemplateItem::Value {
            text,
            span,
        })
    }

    fn start_curly_item(&self) -> Result<ValueTemplateItem<'a>, Error> {
        Ok(ValueTemplateItem::Curly { c: '{' })
    }

    fn end_curly_item(&self) -> Result<ValueTemplateItem<'a>, Error> {
        Ok(ValueTemplateItem::Curly { c: '}' })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ValueTemplateItem<'a> {
    String { text: &'a str, span: Span },
    Curly { c: char },
    Value { text: &'a str, span: Span },
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    UnfinishedValue(Span),
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
                if let Some((i, c)) = self.char_indices.next() {
                    if c == '}' {
                        self.mode = Mode::EndCurly;
                        end = i;
                        self.start = end + 1;
                        return Some(self.value_item(start, end));
                    }
                } else {
                    // we cannot have a start curly without an end curly
                    end = self.s.len();
                    let span = Span {
                        start: self.span.start + self.start,
                        end: self.span.start + end,
                    };
                    self.done = true;
                    return Some(Err(Error::UnfinishedValue(span)));
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

                self.mode = Mode::String;
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_without_curly() {
        let s = "hello world";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(tokens, vec![ValueTemplateItem::String { text: s, span }]);
    }

    #[test]
    fn test_string_start_curly_escaped() {
        let s = "hello{{world";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello",
                    span: Span { start: 0, end: 5 }
                },
                ValueTemplateItem::Curly { c: '{' },
                ValueTemplateItem::String {
                    text: "world",
                    span: Span { start: 7, end: 12 }
                }
            ]
        );
    }

    #[test]
    fn test_string_end_curly_escaped() {
        let s = "hello}}world";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello",
                    span: Span { start: 0, end: 5 }
                },
                ValueTemplateItem::Curly { c: '}' },
                ValueTemplateItem::String {
                    text: "world",
                    span: Span { start: 7, end: 12 }
                }
            ]
        );
    }

    #[test]
    fn test_string_with_value() {
        let s = "hello {world}!";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello ",
                    span: Span { start: 0, end: 6 },
                },
                ValueTemplateItem::Value {
                    text: "world",
                    span: Span { start: 7, end: 12 },
                },
                ValueTemplateItem::String {
                    text: "!",
                    span: Span { start: 13, end: 14 },
                },
            ]
        );
    }

    #[test]
    fn test_string_with_empty_value() {
        let s = "hello {}!";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello ",
                    span: Span { start: 0, end: 6 },
                },
                ValueTemplateItem::Value {
                    text: "",
                    span: Span { start: 7, end: 7 },
                },
                ValueTemplateItem::String {
                    text: "!",
                    span: Span { start: 8, end: 9 },
                },
            ]
        );
    }

    #[test]
    fn test_string_with_multiple_values() {
        let s = "hello {a} and {b}!";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello ",
                    span: Span { start: 0, end: 6 },
                },
                ValueTemplateItem::Value {
                    text: "a",
                    span: Span { start: 7, end: 8 },
                },
                ValueTemplateItem::String {
                    text: " and ",
                    span: Span { start: 9, end: 14 },
                },
                ValueTemplateItem::Value {
                    text: "b",
                    span: Span { start: 15, end: 16 },
                },
                ValueTemplateItem::String {
                    text: "!",
                    span: Span { start: 17, end: 18 },
                },
            ]
        );
    }

    #[test]
    fn test_string_with_multiple_adjacent_values() {
        let s = "hello {a}{b}!";
        let span = Span {
            start: 0,
            end: s.len(),
        };
        let tokenizer = ValueTemplateTokenizer::new(s, span);
        let tokens = tokenizer.collect::<Result<Vec<_>, Error>>().unwrap();
        assert_eq!(
            tokens,
            vec![
                ValueTemplateItem::String {
                    text: "hello ",
                    span: Span { start: 0, end: 6 },
                },
                ValueTemplateItem::Value {
                    text: "a",
                    span: Span { start: 7, end: 8 },
                },
                ValueTemplateItem::Value {
                    text: "b",
                    span: Span { start: 10, end: 11 },
                },
                ValueTemplateItem::String {
                    text: "!",
                    span: Span { start: 12, end: 13 },
                },
            ]
        );
    }
}

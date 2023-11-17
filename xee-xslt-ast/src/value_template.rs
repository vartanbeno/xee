use crate::ast_core::Span;

struct ValueTemplateTokenizer<'a> {
    s: &'a str,
    char_indices: std::str::CharIndices<'a>,
    span: Span,
    mode: Mode,
    start: usize,
    done: bool,
}

enum Mode {
    String,
    StartCurly,
}

impl<'a> ValueTemplateTokenizer<'a> {
    fn new(s: &'a str, span: Span) -> Self {
        Self {
            s,
            char_indices: s.char_indices(),
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

    fn string_item(&self, start: usize, end: usize) -> Result<ValueTemplateItem<'a>, Error> {
        self.item(start, end, |text, span| ValueTemplateItem::String {
            text,
            span,
        })
    }

    fn value_item(&self, start: usize, end: usize) -> Result<ValueTemplateItem<'a>, Error> {
        self.item(start, end, |text, span| ValueTemplateItem::Value {
            text,
            span,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ValueTemplateItem<'a> {
    String { text: &'a str, span: Span },
    Curly { c: char, span: Span },
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
                    if c == '{' {
                        self.mode = Mode::StartCurly;
                        end = i;
                        self.start = end + 1;
                        return Some(self.string_item(start, end));
                    }
                } else {
                    end = self.s.len();
                    self.done = true;
                    return Some(self.string_item(start, end));
                }
            },
            Mode::StartCurly => loop {
                if let Some((i, c)) = self.char_indices.next() {
                    if c == '}' {
                        self.mode = Mode::String;
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
    fn test_string_with_curly() {
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
}

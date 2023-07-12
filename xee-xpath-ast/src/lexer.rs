use std::borrow::Cow;

use ibig::IBig;
use logos::{FilterResult, Lexer, Logos, Span, SpannedIter};
use rust_decimal::Decimal;

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(subpattern name_start_char_without_colon = r"[A-Za-z_\u{c0}-\u{d6}\u{d8}-\u{f6}\u{f8}-\u{2ff}\u{370}-\u{37d}\u{37f}-\u{1fff}\u{200c}-\u{200d}\u{2070}-\u{218f}\u{2c00}-\u{2fef}\u{3001}-\u{d7ff}\u{f900}-\u{fdfc}\u{fdf0}-\u{fffd}\u{10000}-\u{effff}]")]
#[logos(subpattern name_char_without_colon = r"(?&name_start_char_without_colon)|[\-\.0-9\u{b7}\u{300}-\u{36F}\u{203f}-\u{2040}]")]
#[logos(subpattern ncname = r"(?&name_start_char_without_colon)(?&name_char_without_colon)*")]
pub enum Token<'a> {
    Error,
    #[regex(r"[0-9]+", integer_literal, priority = 3)]
    IntegerLiteral(IBig),
    #[regex(r"(\.[0-9]+)|([0-9]+\.[0-9]*)", decimal_literal, priority = 2)]
    DecimalLiteral(Decimal),
    #[regex(
        r"(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][+-]?[0-9]+)",
        double_literal,
        priority = 2
    )]
    DoubleLiteral(f64),
    #[regex(r#""(?:""|[^"])*"|'(?:''|[^'])*'"#, string_literal, priority = 1)]
    StringLiteral(Cow<'a, str>),
    #[regex(r"(?&ncname)")]
    NCName(&'a str),
    // QName is a token according to the spec, but it's too complex to analyze
    // in the lexer, so we will do it in the grammar. QName always ends with an
    // NCName so the delimiter rules based on NCName should be okay.
    #[regex(r#"Q\{[^\{\}]*\}"#, braced_uri_literal, priority = 4)]
    BracedURILiteral(&'a str),

    #[token("!")]
    ExclamationMark,
    #[token("!=")]
    NotEqual,
    #[token("#")]
    Hash,
    #[token("$")]
    Dollar,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("*")]
    Asterisk,
    #[token("*:")]
    AsteriskColon,
    #[token("+")]
    Plus,
    #[token(",")]
    Comma,
    #[token("-")]
    Minus,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("/")]
    Slash,
    #[token("//")]
    DoubleSlash,
    #[token(":")]
    Colon,
    #[token(":*")]
    ColonAsterisk,
    #[token("::")]
    DoubleColon,
    #[token(":=")]
    ColonEqual,
    #[token("<")]
    LessThan,
    #[token("<<")]
    Precedes,
    #[token("<=")]
    LessThanEqual,
    #[token("=")]
    Equal,
    #[token("=>")]
    Arrow,
    #[token(">")]
    GreaterThan,
    #[token(">=")]
    GreaterThanEqual,
    #[token(">>")]
    Follows,
    #[token("?")]
    QuestionMark,
    #[token("@")]
    At,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("|")]
    Pipe,
    #[token("||")]
    DoublePipe,
    #[token("}")]
    RightBrace,

    #[token("ancestor")]
    Ancestor,
    #[token("ancestor-or-self")]
    AncestorOrSelf,
    #[token("and")]
    And,
    #[token("array")]
    Array,
    #[token("as")]
    As,
    #[token("attribute")]
    Attribute,
    #[token("cast")]
    Cast,
    #[token("castable")]
    Castable,
    #[token("child")]
    Child,
    #[token("comment")]
    Comment,
    #[token("descendant")]
    Descendant,
    #[token("descendant-or-self")]
    DescendantOrSelf,
    #[token("div")]
    Div,
    #[token("document-node")]
    DocumentNode,
    #[token("element")]
    Element,
    #[token("else")]
    Else,
    #[token("empty-sequence")]
    EmptySequence,
    #[token("eq")]
    Eq,
    #[token("every")]
    Every,
    #[token("except")]
    Except,
    #[token("following")]
    Following,
    #[token("following-sibling")]
    FollowingSibling,
    #[token("for")]
    For,
    #[token("function")]
    Function,
    #[token("ge")]
    Ge,
    #[token("gt")]
    Gt,
    #[token("idiv")]
    Idiv,
    #[token("if")]
    If,
    #[token("in")]
    In,
    #[token("instance")]
    Instance,
    #[token("intersect")]
    Intersect,
    #[token("is")]
    Is,
    #[token("item")]
    Item,
    #[token("le")]
    Le,
    #[token("let")]
    Let,
    #[token("lt")]
    Lt,
    #[token("map")]
    Map,
    #[token("mod")]
    Mod,
    #[token("namespace")]
    Namespace,
    #[token("namespace-node")]
    NamespaceNode,
    #[token("ne")]
    Ne,
    #[token("node")]
    Node,
    #[token("of")]
    Of,
    #[token("or")]
    Or,
    #[token("parent")]
    Parent,
    #[token("preceding")]
    Preceding,
    #[token("preceding-sibling")]
    PrecedingSibling,
    #[token("processing-instruction")]
    ProcessingInstruction,
    #[token("return")]
    Return,
    #[token("satisfies")]
    Satisfies,
    #[token("schema-attribute")]
    SchemaAttribute,
    #[token("schema-element")]
    SchemaElement,
    #[token("self")]
    Self_,
    #[token("some")]
    Some,
    #[token("text")]
    Text,
    #[token("then")]
    Then,
    #[token("to")]
    To,
    #[token("treat")]
    Treat,
    #[token("union")]
    Union,
    // whitespace
    #[regex(r"[\u{20}\u{9}\u{d}\u{a}]+", priority = 4)]
    Whitespace,
    // comments
    #[regex(r"\(:")]
    CommentStart,
    #[regex(r":\)")]
    CommentEnd,
}

enum SymbolType {
    Delimiting,
    NonDelimiting,
    Whitespace,
    CommentStart,
    CommentEnd,
}

impl<'a> Token<'a> {
    fn symbol_type(&self) -> SymbolType {
        use Token::*;
        match self {
            // A.2.2 terminal delimination
            // delimiting terminal symbols
            ExclamationMark | NotEqual | StringLiteral(_) | Hash | Dollar | LeftParen
            | RightParen | Asterisk | AsteriskColon | Plus | Comma | Minus | Dot | DotDot
            | Slash | DoubleSlash | Colon | ColonAsterisk | DoubleColon | ColonEqual | LessThan
            | Precedes | LessThanEqual | Equal | Arrow | GreaterThan | GreaterThanEqual
            | Follows | QuestionMark | At | BracedURILiteral(_) | LeftBracket | RightBracket
            | LeftBrace | Pipe | DoublePipe | RightBrace => SymbolType::Delimiting,

            // non-delimiting terminal symbols
            IntegerLiteral(_)
            | NCName(_)
            | DecimalLiteral(_)
            | DoubleLiteral(_)
            | Ancestor
            | AncestorOrSelf
            | And
            | Array
            | As
            | Attribute
            | Cast
            | Castable
            | Child
            | Comment
            | Descendant
            | DescendantOrSelf
            | Div
            | DocumentNode
            | Element
            | Else
            | EmptySequence
            | Eq
            | Every
            | Except
            | Following
            | FollowingSibling
            | For
            | Function
            | Ge
            | Gt
            | Idiv
            | If
            | In
            | Instance
            | Intersect
            | Is
            | Item
            | Le
            | Let
            | Lt
            | Map
            | Mod
            | Namespace
            | NamespaceNode
            | Ne
            | Node
            | Of
            | Or
            | Parent
            | Preceding
            | PrecedingSibling
            | ProcessingInstruction
            | Return
            | Satisfies
            | SchemaAttribute
            | SchemaElement
            | Self_
            | Some
            | Text
            | Then
            | To
            | Treat
            | Union => SymbolType::NonDelimiting,

            // symbols that in some way deliminate
            Token::Whitespace => SymbolType::Whitespace,
            Token::CommentStart => SymbolType::CommentStart,
            Token::CommentEnd => SymbolType::CommentEnd,

            // required by Chumsky, but not present yet in the raw
            // lexer
            Token::Error => unreachable!(),
        }
    }
}

fn comment<'a>(lex: &mut Lexer<'a, Token<'a>>) -> FilterResult<(), ()> {
    let mut depth = 1;
    while depth > 0 {
        match lex.next() {
            Some(Ok(Token::CommentStart)) => depth += 1,
            Some(Ok(Token::CommentEnd)) => depth -= 1,
            None => break,
            _ => {}
        }
    }
    if depth > 0 {
        FilterResult::Error(())
    } else {
        FilterResult::Skip
    }
}

fn integer_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> IBig {
    IBig::from_str_radix(lex.slice(), 10).unwrap()
}

fn decimal_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Decimal, ()> {
    let d: Result<Decimal, ()> = lex.slice().try_into().map_err(|_| ());
    d
}

fn double_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<f64, ()> {
    let d: Result<f64, ()> = lex.slice().parse().map_err(|_| ());
    d
}

fn string_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Cow<'a, str> {
    let slice = lex.slice();
    let s = &slice[1..slice.len() - 1];
    if slice.starts_with('\"') {
        if s.contains("\"\"") {
            Cow::Owned(s.replace("\"\"", "\""))
        } else {
            Cow::Borrowed(s)
        }
    } else if s.contains("''") {
        Cow::Owned(s.replace("''", "'"))
    } else {
        Cow::Borrowed(s)
    }
}

fn braced_uri_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> &'a str {
    let slice = lex.slice();
    &slice[2..slice.len() - 1]
}

enum LastTerminal {
    NCName,
    NumericLiteral,
    Dot,
    Other,
}

pub(crate) struct XPathLexer<'a> {
    spanned: SpannedIter<'a, Token<'a>>,
    last_is_separator: bool,
    last_is_non_delimiting: bool,
    last_terminal: LastTerminal,
}

impl<'a> XPathLexer<'a> {
    fn new(lexer: Lexer<'a, Token<'a>>) -> Self {
        let spanned = lexer.spanned();
        Self {
            spanned,
            last_is_separator: true,
            last_is_non_delimiting: false,
            last_terminal: LastTerminal::Other,
        }
    }
}

impl<'a> Iterator for XPathLexer<'a> {
    type Item = (Result<Token<'a>, ()>, Span);

    // A.2.2 Terminal Delimination
    fn next(&mut self) -> Option<Self::Item> {
        let token_span = self.spanned.next();
        match &token_span {
            Some((token, span)) => match token {
                Ok(token) => {
                    use SymbolType::*;
                    match token.symbol_type() {
                        Delimiting => {
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
                            token_span
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
                                    // Checking that isn't necessary, as a leading
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
                                    token_span
                                } else {
                                    Some((Err(()), span.clone()))
                                }
                            } else {
                                // if we've seen delimiting last, we're fine
                                token_span
                            };
                            self.last_is_separator = false;
                            self.last_is_non_delimiting = true;

                            r
                        }
                        Whitespace => {
                            self.last_is_separator = true;
                            self.next()
                        }
                        CommentStart => {
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
                        CommentEnd => token_span,
                    }
                }
                Err(_) => token_span,
            },
            None => None,
        }
    }
}

pub(crate) fn lexer(input: &str) -> XPathLexer {
    XPathLexer::new(Token::lexer(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ibig::ibig;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tokenize() {
        let mut lex = lexer("cast as");
        assert_eq!(lex.next(), Some((Ok(Token::Cast), (0..4))));
        assert_eq!(lex.next(), Some((Ok(Token::As), (5..7))));
    }

    #[test]
    fn test_comment_by_itself() {
        let mut lex = lexer("(: this is a comment :)");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comment_weird_content() {
        let mut lex = lexer("(: 1name :)");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comment_multiline() {
        let mut lex = lexer("(: this is\na comment :)");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comment_nested() {
        let mut lex = lexer("(: this (:is a:) comment :)");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comment_nested_broken() {
        let mut lex = lexer("(: this (:is a:) comment :");
        assert_eq!(lex.next(), Some((Err(()), (0..26))));
    }

    #[test]
    fn test_close_comment_by_itself() {
        let mut lex = lexer(":)");
        // this comment end will appear in the token stream,
        // but the parser will reject it
        assert_eq!(lex.next(), Some((Ok(Token::CommentEnd), (0..2))));
    }

    #[test]
    fn test_open_comment_by_itself() {
        let mut lex = lexer("(:");
        // this is an unterminated comment
        assert_eq!(lex.next(), Some((Err(()), (0..2))));
    }

    #[test]
    fn test_integer_literal() {
        let mut lex = lexer("123");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(123))), (0..3)))
        );
    }

    #[test]
    fn test_decimal_literal() {
        let mut lex = lexer("123.456");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DecimalLiteral(dec!(123.456))), (0..7)))
        );
    }

    #[test]
    fn test_decimal_starts_with_dot() {
        let mut lex = lexer(".456");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DecimalLiteral(dec!(0.456))), (0..4)))
        );
    }

    #[test]
    fn test_decimal_literal_too_big() {
        let mut lex = lexer("12300000000000000000000000000000000.456");
        assert_eq!(lex.next(), Some((Err(()), (0..39))));
    }

    #[test]
    fn test_double_literal() {
        let mut lex = lexer("123.456e-5");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DoubleLiteral(123.456e-5)), (0..10)))
        );
    }

    #[test]
    fn test_double_literal_starts_with_dot() {
        let mut lex = lexer(".456e-5");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DoubleLiteral(0.456e-5)), (0..7)))
        );
    }

    #[test]
    fn test_string_literal_double_quotes() {
        let mut lex = lexer(r#""foo""#);
        assert_eq!(
            lex.next(),
            Some((Ok(Token::StringLiteral(Cow::Borrowed("foo"))), (0..5)))
        );
    }

    #[test]
    fn test_string_literal_double_quotes_escape() {
        let mut lex = lexer(r#""fo""o""#);
        assert_eq!(
            lex.next(),
            Some((
                Ok(Token::StringLiteral(Cow::Owned(r#"fo"o"#.to_string()))),
                (0..7)
            ))
        );
    }

    #[test]
    fn test_string_literal_single_quotes() {
        let mut lex = lexer(r#"'foo'"#);
        assert_eq!(
            lex.next(),
            Some((Ok(Token::StringLiteral(Cow::Borrowed("foo"))), (0..5)))
        );
    }

    #[test]
    fn test_string_literal_single_quotes_escape() {
        let mut lex = lexer(r#"'fo''o'"#);
        assert_eq!(
            lex.next(),
            Some((
                Ok(Token::StringLiteral(Cow::Owned(r#"fo'o"#.to_string()))),
                (0..7)
            ))
        );
    }

    #[test]
    fn test_string_literal_single_quotes_escape2() {
        let mut lex = lexer(r#"'fo''o''l'"#);
        assert_eq!(
            lex.next(),
            Some((
                Ok(Token::StringLiteral(Cow::Owned(r#"fo'o'l"#.to_string()))),
                (0..10)
            ))
        );
    }

    #[test]
    fn test_ncname() {
        let mut lex = lexer("foo");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("foo")), (0..3))));
    }

    #[test]
    fn test_prefixed_name() {
        let mut lex = lexer("xs:integer");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("xs")), (0..2))));
        assert_eq!(lex.next(), Some((Ok(Token::Colon), (2..3))));
        assert_eq!(lex.next(), Some((Ok(Token::NCName("integer")), (3..10))));
    }

    #[test]
    fn test_braced_uri_literal() {
        let mut lex = lexer("Q{http://example.com}");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::BracedURILiteral("http://example.com")), (0..21)))
        );
    }

    #[test]
    fn two_non_delimiting_tokens_not_separated() {
        let mut lex = lexer("1cast");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), (0..1)))
        );
        assert_eq!(lex.next(), Some((Err(()), 1..5)));
    }

    #[test]
    fn delimiting_and_non_delimiting_not_separated() {
        let mut lex = lexer("'1'cast");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::StringLiteral(Cow::Borrowed("1"))), (0..3)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::Cast), 3..7)));
    }

    #[test]
    fn non_delimiting_and_delimiting_not_separated() {
        let mut lex = lexer("cast'1'");
        assert_eq!(lex.next(), Some((Ok(Token::Cast), 0..4)));
        assert_eq!(
            lex.next(),
            Some((Ok(Token::StringLiteral(Cow::Borrowed("1"))), (4..7)))
        );
    }

    #[test]
    fn two_non_delimiting_tokens_separated_by_whitespace() {
        let mut lex = lexer("1 cast");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), (0..1)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::Cast), 2..6)));
    }

    #[test]
    fn two_non_delimiting_tokens_separated_by_comment() {
        let mut lex = lexer("1(:hello:)cast");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), (0..1)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::Cast), (10..14))));
    }

    #[test]
    fn qname_then_dot_is_ncname() {
        let mut lex = lexer("xs:integer.");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("xs")), (0..2))));
        assert_eq!(lex.next(), Some((Ok(Token::Colon), (2..3))));
        assert_eq!(lex.next(), Some((Ok(Token::NCName("integer.")), (3..11))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn qname_then_dot_with_separator_is_ok() {
        let mut lex = lexer("xs:integer .");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("xs")), (0..2))));
        assert_eq!(lex.next(), Some((Ok(Token::Colon), (2..3))));
        assert_eq!(lex.next(), Some((Ok(Token::NCName("integer")), (3..10))));
        assert_eq!(lex.next(), Some((Ok(Token::Dot), 11..12)));
    }

    #[test]
    fn qname_then_minus_is_ncname() {
        let mut lex = lexer("xs:integer-");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("xs")), (0..2))));
        assert_eq!(lex.next(), Some((Ok(Token::Colon), (2..3))));
        assert_eq!(lex.next(), Some((Ok(Token::NCName("integer-")), (3..11))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn qname_then_minus_with_separator_is_ok() {
        let mut lex = lexer("xs:integer -");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("xs")), (0..2))));
        assert_eq!(lex.next(), Some((Ok(Token::Colon), (2..3))));
        assert_eq!(lex.next(), Some((Ok(Token::NCName("integer")), (3..10))));
        assert_eq!(lex.next(), Some((Ok(Token::Minus), 11..12)));
    }

    #[test]
    fn decimal_with_dot_must_have_separator() {
        let mut lex = lexer("1.0.");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DecimalLiteral(dec!(1.0))), (0..3)))
        );
        assert_eq!(lex.next(), Some((Err(()), 3..4)));
    }

    #[test]
    fn double_with_dot_must_have_separator() {
        let mut lex = lexer("1.2e3.");
        assert_eq!(lex.next(), Some((Ok(Token::DoubleLiteral(1.2e3)), (0..5))));
        assert_eq!(lex.next(), Some((Err(()), 5..6)));
    }

    #[test]
    fn dot_with_decimal_must_have_separator() {
        let mut lex = lexer(".1.2");
        assert_eq!(
            lex.next(),
            Some((Ok(Token::DecimalLiteral(dec!(0.1))), (0..2)))
        );
        assert_eq!(lex.next(), Some((Err(()), 2..4)));
    }

    #[test]
    fn test_simple_map() {
        let mut lex = lexer("(1, 2) ! (. * 2)");
        assert_eq!(lex.next(), Some((Ok(Token::LeftParen), (0..1))));
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(1))), (1..2)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::Comma), (2..3))));
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(2))), (4..5)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::RightParen), (5..6))));
        assert_eq!(lex.next(), Some((Ok(Token::ExclamationMark), (7..8))));
        assert_eq!(lex.next(), Some((Ok(Token::LeftParen), (9..10))));
        assert_eq!(lex.next(), Some((Ok(Token::Dot), (10..11))));
        assert_eq!(lex.next(), Some((Ok(Token::Asterisk), (12..13))));
        assert_eq!(
            lex.next(),
            Some((Ok(Token::IntegerLiteral(ibig!(2))), (14..15)))
        );
        assert_eq!(lex.next(), Some((Ok(Token::RightParen), (15..16))));
    }

    #[test]
    fn test_ncname_contains_minus() {
        let mut lex = lexer("a-b");
        assert_eq!(lex.next(), Some((Ok(Token::NCName("a-b")), (0..3))));
    }
}

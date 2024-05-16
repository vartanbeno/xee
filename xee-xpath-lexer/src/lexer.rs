use std::borrow::Cow;

use ibig::IBig;
use logos::{Lexer, Logos, Span, SpannedIter};
use rust_decimal::Decimal;

// #[derive(Clone, Debug, PartialEq)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize))]
// pub struct PrefixedQName<'a> {
//     pub prefix: &'a str,
//     pub local_name: &'a str,
// }

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(subpattern name_start_char_without_colon = r"[A-Za-z_\u{c0}-\u{d6}\u{d8}-\u{f6}\u{f8}-\u{2ff}\u{370}-\u{37d}\u{37f}-\u{1fff}\u{200c}-\u{200d}\u{2070}-\u{218f}\u{2c00}-\u{2fef}\u{3001}-\u{d7ff}\u{f900}-\u{fdfc}\u{fdf0}-\u{fffd}\u{10000}-\u{effff}]")]
#[logos(subpattern name_char_without_colon = r"(?&name_start_char_without_colon)|[\-\.0-9\u{b7}\u{300}-\u{36F}\u{203f}-\u{2040}]")]
#[logos(subpattern ncname = r"(?&name_start_char_without_colon)(?&name_char_without_colon)*")]
// #[logos(subpattern prefixedname = r"(?&ncname):(?&ncname)")]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    // QName is a token according to the spec, but it's too complex to analyze
    // in the lexer, as ncnames can also occur by themselves. Instead we find
    // prefixed qnames and figure out whether an ncname is an unprefixed qname
    // later on in the parser
    // #[regex(r"(?&prefixedname)", prefixed_qname, priority = 2)]
    // PrefixedQName(PrefixedQName<'a>),
    #[regex(r"(?&ncname)", priority = 2)]
    NCName(&'a str),

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

// fn prefixed_qname<'a>(lex: &mut Lexer<'a, Token<'a>>) -> PrefixedQName<'a> {
//     let slice = lex.slice();
//     let colon = slice.find(':').unwrap();
//     PrefixedQName {
//         prefix: &slice[..colon],
//         local_name: &slice[colon + 1..],
//     }
// }

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
    fn new(lexer: Lexer<'a, Token<'a>>) -> Self {
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
        let token_span = self.spanned.next();
        match &token_span {
            Some((token, span)) => match token {
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
                        CommentEnd => token_span,
                    }
                }
                Err(_) => token_span,
            },
            None => None,
        }
    }
}

pub fn lexer(input: &str) -> XPathLexer {
    XPathLexer::new(Token::lexer(input))
}

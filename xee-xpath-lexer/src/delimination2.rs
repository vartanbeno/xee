use logos::Span;

use crate::lexer::SymbolType;
use crate::{explicit_whitespace::ExplicitWhitespaceIterator, Token};

pub(crate) struct DeliminationIterator<'a> {
    base: ExplicitWhitespaceIterator<'a>,
}

impl<'a> DeliminationIterator<'a> {
    pub(crate) fn new(base: ExplicitWhitespaceIterator<'a>) -> Self {
        Self { base }
    }

    // fn eat_comments(&mut self) {
    //     let mut depth = 1;
    //     // we track the span from the start of the first
    //     // comment start
    //     let start = span.start;
    //     let mut end = span.end;
    //     // now we find the commend end that matches,
    //     // taking into account nested comments
    //     // we track the end of the span of what we
    //     // found next, so that we can report it in
    //     // case of errors
    //     while depth > 0 {
    //         match self.spanned.next() {
    //             Some((Ok(Token::CommentStart), span)) => {
    //                 end = span.end;
    //                 depth += 1
    //             }
    //             Some((Ok(Token::CommentEnd), span)) => {
    //                 end = span.end;
    //                 depth -= 1;
    //                 // comments are balanced, so done
    //                 if depth == 0 {
    //                     break;
    //                 }
    //             }
    //             // If we run into a non-comment, we skip it
    //             Some((_, span)) => {
    //                 end = span.end;
    //             }
    //             // if we reach the end and things are unclosed,
    //             // we bail out
    //             None => {
    //                 return Some((Err(()), start..end));
    //             }
    //         }
    //     }
    //     self.last_is_separator = true;
    //     self.next()
    // }
}

// impl<'a> Iterator for DeliminationIterator<'a> {
//     type Item = (Result<Token<'a>, ()>, Span);

//     fn next(&mut self) -> Option<Self::Item> {
//         let (token, span) = self.base.next()?;
//         if let Ok(token) = token {
//             match token.symbol_type() {
//                 SymbolType::Delimiting => return Some((Ok(token), span)),
//                 SymbolType::NonDelimiting => match self.peek_symbol_type() {
//                     Some(SymbolType::Delimiting) => {}
//                     Some(SymbolType::Whitespace) => {}
//                     Some(SymbolType::NonDelimiting) => {}
//                 },
//                 SymbolType::Whitespace => self.next(),
//                 SymbolType::CommentStart => {}
//                 SymbolType::CommentEnd => {}
//             }
//         } else {
//             Some((Err(()), span))
//         }
//     }
// }

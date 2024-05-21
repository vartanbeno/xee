use logos::{Span, SpannedIter};

use crate::Token;

// we have an iterator with spans

// we keep track of previously seen tokens if they match the pattern
// if not, we release the previously seen tokens if any are present, then
// release the current token

// enum PatternState {
//     NotInPattern,
//     InPattern,
//     EndsPattern,
// }

// struct PatternMatchIterator<
//     'a,
//     const L: usize,
//     F: Fn(&[Option<(Token<'a>, Span)>; L], (Token<'a>, Span)) -> PatternState,
// > {
//     spanned_iter: SpannedIter<'a, Token<'a>>,
//     pattern_tokens: [Option<(Token<'a>, Span)>; L],
//     in_pattern: F,
//     release_index: Option<usize>,
// }

// impl<
//         'a,
//         const L: usize,
//         F: Fn(&[Option<(Token<'a>, Span)>; L], (Token<'a>, Span)) -> PatternState,
//     > Iterator for PatternMatchIterator<'a, L, F>
// {
//     type Item = (Result<Token<'a>, ()>, Span);

//     fn next(&mut self) -> Option<Self::Item> {
//         // release any buffered tokens if we have any
//         if let Some(release_index) = self.release_index {
//             // now take the token to release
//             if let Some(buffered_token) = self.pattern_tokens[release_index].take() {
//                 // update release index, or clear it if we're done
//                 self.release_index = if release_index < L {
//                     Some(release_index + 1)
//                 } else {
//                     None
//                 };
//                 return Some((Ok(buffered_token), *span));
//             } else {
//                 // we don't find a token in our buffer, so we are done with
//                 // buffered tokens for now
//                 self.release_index = None;
//             }
//             return Some((Ok(token), *span));
//         }

//         let token_span = self.spanned_iter.next()?;
//         match &token_span {
//             (token, span) => match token {
//                 Ok(token) => {
//                     let state = (self.in_pattern)(&self.pattern_tokens, (token, span));
//                     match state {
//                         PatternState::NotInPattern => {}
//                         PatternState::InPattern => {}
//                         PatternState::EndsPattern => {}
//                     }
//                 }
//                 Err(_) => Some((Err(()), *span)),
//             },
//         }
//     }
// }

// if that iterator contains the sequence ncname colon ncname, then we
// combine this into a single token, prefixed ncname
// otherwise we just yield the elements

// if we see an ncname, and we just saw ncname colon,
// consume these and yield prefixed name

// if we see an ncname and we didn't see ncname colon, yield ncname

// if we see a colon if we're tracking ncname, keep track of it, otherwise yield token
// if we see something else, track it, yield token
// if we see something else, track it, yield

// we could keep track of the last two tokens
// if that is an ncname and a colon, and we get an ncname, wipe out the
// information seen, and yield a prefixed ncname
// if it's not an ncname, then we yield the first token, keep the second
// if there's still a second token, we yield that
// finally we yield the ncname

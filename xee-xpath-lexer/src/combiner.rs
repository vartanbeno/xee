use logos::{Span, SpannedIter};

use crate::Token;

// an iterator that keeps track of the last n items seen
struct SeenIterator<T: Clone, S: Clone, I, const L: usize>
where
    I: Iterator<Item = (Result<T, ()>, S)>,
{
    spanned_iter: I,
    seen_tokens: [Option<(T, S)>; L],
}

impl<T: Clone, S: Clone, I, const L: usize> SeenIterator<T, S, I, L>
where
    I: Iterator<Item = (Result<T, ()>, S)>,
{
    const ARRAY_REPEAT_VALUE: std::option::Option<(T, S)> = None;

    fn new(spanned_iter: I) -> Self {
        Self {
            spanned_iter,
            seen_tokens: [Self::ARRAY_REPEAT_VALUE; L],
        }
    }

    // get back an iterator that returns the seen items if they're not None
    pub fn seen(&self) -> impl Iterator<Item = (T, S)> + '_ {
        self.seen_tokens
            .iter()
            .filter_map(|x| x.as_ref().map(|(t, s)| (t.clone(), s.clone())))
    }
}

impl<T: Clone, S: Clone, I, const L: usize> Iterator for SeenIterator<T, S, I, L>
where
    I: Iterator<Item = (Result<T, ()>, S)>,
{
    type Item = (Result<T, ()>, S);

    fn next(&mut self) -> Option<Self::Item> {
        let token_span = self.spanned_iter.next()?;
        match token_span {
            (Ok(token), span) => {
                // shift the seen tokens before the last token, each goes one back
                for i in 0..L - 1 {
                    self.seen_tokens[i] = self.seen_tokens[i + 1].take();
                }
                // add the new token at the end
                self.seen_tokens[L - 1] = Some((token.clone(), span.clone()));
                Some((Ok(token), span.clone()))
            }
            (Err(_), span) => Some((Err(()), span.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seen_iterator_no_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.seen().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_seen_iterator_one_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.seen().collect::<Vec<_>>(), vec![(0, 0..0)]);
    }

    #[test]
    fn test_seen_iterator_two_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(1), 0..0)));
        assert_eq!(
            seen_iter.seen().collect::<Vec<_>>(),
            vec![(0, 0..0), (1, 0..0)]
        );
    }

    #[test]
    fn test_seen_iterator_three_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(1), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(2), 0..0)));
        assert_eq!(
            seen_iter.seen().collect::<Vec<_>>(),
            vec![(1, 0..0), (2, 0..0)]
        );
    }

    #[test]
    fn test_seen_iterator_four_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(1), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(2), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(3), 0..0)));
        assert_eq!(
            seen_iter.seen().collect::<Vec<_>>(),
            vec![(2, 0..0), (3, 0..0)]
        );
    }

    #[test]
    fn test_seen_iterator_five_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(1), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(2), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(3), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(4), 0..0)));
        assert_eq!(
            seen_iter.seen().collect::<Vec<_>>(),
            vec![(3, 0..0), (4, 0..0)]
        );
    }

    #[test]
    fn test_seen_iterator_six_iteration() {
        let v = vec![0, 1, 2, 3, 4];
        let spanned_iter = v.iter().map(|x| (Ok(*x), 0..0));
        let mut seen_iter = SeenIterator::<_, _, _, 2>::new(spanned_iter);
        assert_eq!(seen_iter.next(), Some((Ok(0), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(1), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(2), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(3), 0..0)));
        assert_eq!(seen_iter.next(), Some((Ok(4), 0..0)));
        assert_eq!(seen_iter.next(), None);
        assert_eq!(
            seen_iter.seen().collect::<Vec<_>>(),
            vec![(3, 0..0), (4, 0..0)]
        );
    }
}

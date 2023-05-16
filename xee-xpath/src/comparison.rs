// use ahash::{HashSet, HashSetExt};

// use crate::value::Atomic;

// // generalized comparison
// fn compare_eq(a: &[Atomic], b: &[Atomic]) -> bool {
//     // index the shortest sequence
//     if a.len() > b.len() {
//         return compare_eq(b, a);
//     }
//     // a should be the shortest sequence, turn into a hash set
//     let a: HashSet<_> = a.iter().collect();
// }

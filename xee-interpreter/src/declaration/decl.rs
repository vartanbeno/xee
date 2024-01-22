use std::rc::Rc;

use crate::sequence::Sequence;

use super::globalvar::GlobalVariables;

struct Declarations<'a> {
    global_variables: Rc<GlobalVariables<'a, Sequence>>,
}

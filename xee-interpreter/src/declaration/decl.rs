use std::rc::Rc;

use crate::sequence::Sequence;

use super::globalvar::GlobalVariables;

pub struct Declarations<'a> {
    global_variables: Rc<GlobalVariables<'a, Sequence>>,
}

impl Declarations<'_> {
    pub(crate) fn new() -> Self {
        Self {
            global_variables: Rc::new(GlobalVariables::new()),
        }
    }
}

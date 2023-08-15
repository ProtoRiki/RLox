use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::class::LoxClass;

pub struct LoxInstance {
    class: Rc<LoxClass>
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self { class }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
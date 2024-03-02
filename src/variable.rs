use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::bindings::Bindings;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Variable(pub isize, pub Option<String>);

impl Variable {
    pub fn new_named(bindings: Rc<Bindings>, name: &str) -> Self {
        Self(bindings.next(), Some(name.to_string()))
    }

    pub fn new(bindings: Rc<Bindings>) -> Self {
        Self(bindings.next(), None)
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.1 {
            Some(name) => { f.write_str(name) }
            None => {
                f.write_str("X")?;
                f.write_str(&self.0.to_string())
            }
        }
    }
}
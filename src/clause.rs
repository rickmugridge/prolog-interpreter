use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::term::Term;

#[derive(Debug, Eq, PartialEq)]
pub struct Clause {
    // head :- body
    // body of [] is true
    pub head: Rc<Term>,
    pub body: Vec<Rc<Term>>,
    pub contains_variables: bool, // todo Check this when creating one
}

impl Clause {
    pub fn rule(head: Rc<Term>,
                body: Vec<Rc<Term>>) -> Rc<Self> {
        let contains_variables = head.clone().contains_variables() ||
            body.clone().iter().any(|t| t.contains_variables());
        Rc::new(Self {
            head,
            body,
            contains_variables,
        })
    }

    pub fn fact(head: Rc<Term>) -> Rc<Self> {
        let contains_variables = head.clone().contains_variables();
        Rc::new(Self { head, body: vec![], contains_variables })
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.head.to_string())?;
        f.write_str(" -> ")?;
        if self.body.clone().is_empty() {
            f.write_str(" true.")
        } else {
            for term in self.body.clone() {
                f.write_str(&term.to_string())?;
            }
            f.write_str(".")
        }
    }
}

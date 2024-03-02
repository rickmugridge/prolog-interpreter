use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::bindings::Bindings;
use crate::term::Term;

pub struct StaticContext {
    pub bindings: Rc<Bindings>,
    variables: RefCell<HashMap<String, Rc<Term>>>,
}

impl StaticContext {
    pub fn new(bindings: Rc<Bindings>) -> Rc<Self> {
        Rc::new(Self { bindings, variables: RefCell::new(HashMap::new()) })
    }

    pub fn new_all() -> Rc<Self> {
        Rc::new(Self {
            bindings: Bindings::new(),
            variables: RefCell::new(HashMap::new()),
        })
    }

    pub fn var(&self, name: &str) -> Rc<Term> {
        if let Some(term) = self.variables.borrow().get(name) {
            return term.clone();
        }
        let term = Term::var(name, self.bindings.clone());
        self.variables.borrow_mut().insert(name.to_string(), term.clone());
        term
    }
}
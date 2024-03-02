use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::term::{Term};
use crate::variable::Variable;

#[derive(Debug, Clone)]
pub struct Bindings {
    bind: RefCell<HashMap<isize, Rc<Term>>>,
    stack: Option<Rc<Bindings>>,
    next_variable: RefCell<isize>,
}

impl Bindings {
    pub fn len(&self) -> usize {
        let length = self.bind.borrow().len();
        if let Some(stack) = &self.stack {
            return length + stack.len();
        }
        length
    }

    // Instantiate all variables, recursively
    pub fn instantiate(&self, term: Rc<Term>) -> Rc<Term> {
        match term.clone().as_ref() {
            Term::Variable(variable) =>
                match self.bound_directly_to(variable) {
                    None => term,
                    Some(term2) => self.instantiate(term2)
                },
            Term::CompoundTerm(functor, args) =>
                Term::compound(functor, args.iter()
                    .map(|arg| self.instantiate(arg.clone()))
                    .collect()),
            _ => term,
        }
    }

    pub fn bound_directly_to(&self, variable: &Variable) -> Option<Rc<Term>> {
        match self.bind.borrow().get(&variable.0).cloned() {
            Some(result) => Some(result),
            None => match &self.stack {
                Some(bindings) => bindings.bound_directly_to(variable),
                None => None
            }
        }
    }

    pub fn term_bound_directly_to(&self, variable: Rc<Term>) -> Option<Rc<Term>> {
        if let Term::Variable(var) = variable.as_ref() {
            self.bound_directly_to(var)
        } else {
            panic!("Must be a Variable")
        }
    }

    pub fn add(&self, v: isize, term: Rc<Term>) {
        self.bind.borrow_mut().insert(v, term);
    }

    pub fn add_variable(&self, variable: Rc<Term>, term: Rc<Term>) {
        if let Term::Variable(Variable(i, _)) = *variable {
            self.bind.borrow_mut().insert(i, term);
        } else {
            panic!("Must be a Variable")
        }
    }

    pub fn stack(current: Rc<Bindings>) -> Rc<Self> {
        Rc::new(Self {
            bind: RefCell::new(HashMap::new()),
            stack: Some(current.clone()),
            next_variable: RefCell::new(current.next_variable.clone().into_inner()),
        })
    }
}

impl Bindings {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            bind: RefCell::new(HashMap::new()),
            stack: None,
            next_variable: RefCell::new(0),
        })
    }

    pub fn next(&self) -> isize {
        *self.next_variable.borrow_mut() += 1;
        *self.next_variable.borrow()
    }
}

impl Display for Bindings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Bindings(")?;
        f.write_str(&self.next_variable.borrow().to_string())?;
        f.write_str(", [")?;
        for (key, value) in self.bind.borrow().iter() {
            f.write_str("(")?;
            f.write_str(&key.to_string())?;
            f.write_str(" -> ")?;
            f.write_str(&value.clone().to_string())?;
            f.write_str(")")?;
        }
        f.write_str("])")?;
        if let Some(s) = &self.stack {
            f.write_str(" + ")?;
            std::fmt::Display::fmt(s, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod variable_binding_tests {
    use crate::term::Term;
    use crate::bindings::Bindings;

    #[test]
    fn bound_variable() {
        let bindings = Bindings::new();
        let x = Term::var("X", bindings.clone());
        let a = Term::atom("a");
        bindings.add_variable(x.clone(), a.clone());
        assert_eq!(bindings.term_bound_directly_to(x.clone()).expect("Some"), a.clone());
    }

    #[test]
    fn bound_variable_chain_to_atom() {
        /*
           X -> Y.
           Y -> a.
           ?- X.
        */
        let bindings = Bindings::new();
        let x = Term::var("X", bindings.clone());
        let y = Term::var("Y", bindings.clone());
        let a = Term::atom("a");
        bindings.add_variable(x.clone(), y.clone());
        bindings.add_variable(y.clone(), a.clone());
        assert_eq!(bindings.term_bound_directly_to(x.clone()).expect("Some"), y.clone());
        assert_eq!(bindings.term_bound_directly_to(y.clone()).expect("Some"), a.clone());
    }

    #[test]
    fn bound_variable_chain_to_compound() {
        /*
            X -> Y.
            Y -> cat(a).
            ?- X.
         */
        let bindings = Bindings::new();
        let x = Term::var("X", bindings.clone());
        let y = Term::var("Y", bindings.clone());
        let cat = Term::compound1("cat", Term::atom("a"));

        bindings.add_variable(x.clone(), y.clone());
        bindings.add_variable(y.clone(), cat.clone());
        assert_eq!(bindings.term_bound_directly_to(x.clone()).expect("Some"), y.clone());
        assert_eq!(bindings.term_bound_directly_to(y.clone()).expect("Some"), cat.clone());
    }
}

#[cfg(test)]
mod instantiation_tests {
    use crate::term::Term;
    use crate::term_builder::TermBuilder;

    #[test]
    fn unbound_variable() {
        /*
          ?- X.
       */
        let t = TermBuilder::new();
        assert_eq!(t.bindings().instantiate(t.x()), t.x());
    }

    #[test]
    fn bound_variable() {
        /*
           X -> a.
           ?- X.
        */
        let t = TermBuilder::new();
        t.bindings().add_variable(t.x(), t.a());
        assert_eq!(t.bindings().instantiate(t.x()), t.a());
    }

    #[test]
    fn bound_variable_chain_to_atom() {
        /*
           X -> Y.
           Y -> a.
           ?- X.
        */
        let t = TermBuilder::new();
        t.bindings().add_variable(t.x(), t.y());
        t.bindings().add_variable(t.y(), t.a());
        assert_eq!(t.bindings().instantiate(t.x()), t.a());
    }

    #[test]
    fn bound_variable_chain2_to_compound() {
        /*
            X -> Y.
            Y -> cat(a).
            ?- X.
         */
        let t = TermBuilder::new();
        let cat = Term::compound1("cat", t.a());
        t.bindings().add_variable(t.x(), t.y());
        t.bindings().add_variable(t.y(), cat.clone());
        assert_eq!(t.bindings().instantiate(t.x()), cat);
    }

    #[test]
    fn bound_variable_chain3_to_compound() {
        /*
            X -> Y.
            Y -> Z.
            Z -> cat(a).
            ?- X.
         */
        let t = TermBuilder::new();
        let cat = Term::compound1("cat", t.a());
        t.bindings().add_variable(t.x(), t.y());
        t.bindings().add_variable(t.y(), t.z());
        t.bindings().add_variable(t.z(), cat.clone());
        assert_eq!(t.bindings().instantiate(t.x()), cat);
    }
}
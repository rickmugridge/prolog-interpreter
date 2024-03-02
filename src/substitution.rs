use std::rc::Rc;
use crate::term::Term;
use crate::bindings::Bindings;
use crate::clause::Clause;

#[derive(Debug, Clone)]
pub struct Substitution {
    variables_source: Rc<Bindings>,
    substitutions: Rc<Bindings>,
}

impl Substitution {
    pub fn new(variables_source: Rc<Bindings>) -> Self {
        Self { variables_source, substitutions: Bindings::new() }
    }

    fn substitute_new_vars(&self, term: Rc<Term>) -> Rc<Term> {
        match term.clone().as_ref() {
            Term::Variable(variable) => {
                match self.substitutions.bound_directly_to(variable) {
                    None => {
                        let new_variable = Term::var_unnamed(self.variables_source.clone());
                        self.substitutions.add_variable(term, new_variable.clone());
                        new_variable
                    }
                    Some(t) => t
                }
            }
            Term::CompoundTerm(functor, args) => {
                Term::compound(functor, args.iter()
                    .map(|arg| self.substitute_new_vars(arg.clone()))
                    .collect())
            }
            _ => term,
        }
    }

    /*
                   match new_bindings.bound_directly_to(&variable) {
                    None => {
                        let new_variable = Term::var_unnamed(new_bindings.clone());
                        new_bindings.add_variable(term, new_variable.clone());
                        new_variable
                    }
                    Some(t) => t
                }

     */

    fn map(&self, term: Rc<Term>) -> Rc<Term> {
        if term.contains_variables() {
            self.substitute_new_vars(term.clone())
        } else {
            term
        }
    }

    pub fn map_clause(&self, clause: Rc<Clause>) -> Rc<Clause> {
        Clause::rule(self.map(clause.head.clone()),
                     clause.body.iter().map(|t|
                         self.map(t.clone())).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::substitution::Substitution;
    use crate::term_builder::TermBuilder;

    #[test]
    fn just_atom() {
        let t = TermBuilder::new();
        let s = Substitution::new(t.bindings());
        assert_eq!(s.map(t.a()), t.a());
    }

    #[test]
    fn just_variable() {
        let t = TermBuilder::new();
        let s = Substitution::new(t.bindings());
        assert_eq!(s.map(t.x()).to_string(), "X6");
    }

    #[test]
    fn two_variables() {
        let t = TermBuilder::new();
        let s = Substitution::new(t.bindings());
        assert_eq!(s.map(t.fxy()).to_string(), "f(X6, X7)");
    }

    #[test]
    fn same_variable_twice() {
        let t = TermBuilder::new();
        let s = Substitution::new(t.bindings());
        assert_eq!(s.map(t.fxx()).to_string(), "f(X6, X6)");
    }
}


/*#[cfg(test)]
mod test_substitute_new_vars {
    use crate::term::Term;
    use crate::bindings::Bindings;

    #[test]
    fn simple_ones() {
        let new_bindings = Bindings::new();
        let i = Term::int(3);
        assert_eq!(Term::substitute_new_vars(i.clone(), new_bindings.clone()), i);

        let a = Term::atom("a");
        assert_eq!(Term::substitute_new_vars(a.clone(), new_bindings.clone()), a);

        let f = Term::compound1("f", Term::atom("a"));
        assert_eq!(Term::substitute_new_vars(f.clone(), new_bindings.clone()), f);
    }

    #[test]
    fn just_variable() {
        let new_bindings = Bindings::new();
        let x = Term::var("X", new_bindings.clone());
        let result = Term::substitute_new_vars(x.clone(), new_bindings.clone());
        assert_eq!(result.to_string(), "X2");
    }

    #[test]
    fn compound_with_variables() {
        let new_bindings = Bindings::new();
        let y = Term::var("Y", new_bindings.clone());
        let f = Term::compound1("f", y.clone());
        let result = Term::substitute_new_vars(f.clone(), new_bindings.clone());
        assert_eq!(result.to_string(), "f(X2)");
    }
}
*/
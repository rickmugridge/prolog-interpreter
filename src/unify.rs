use std::ops::Deref;
use std::rc::Rc;
use crate::term::{Term};
use crate::bindings::Bindings;
use crate::variable::Variable;

// We assume that the outer-most call of unify() will provide a freshly-stacked Bindings,
// so that the top can be tossed if unification fails
pub fn unify(term1: Rc<Term>, term2: Rc<Term>, bindings: Rc<Bindings>) -> bool {
    match (term1.deref(), term2.deref()) {
        (Term::Atom(s1), Term::Atom(s2)) => s1 == s2,
        (Term::Int(i1), Term::Int(i2)) => i1 == i2,
        (Term::CompoundTerm(f1, args1), Term::CompoundTerm(f2, args2)) =>
            if f1 == f2 && args1.len() == args2.len() {
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if !unify(arg1.clone(), arg2.clone(), bindings.clone()) {
                        return false;
                    }
                }
                true
            } else {
                false
            },
        (Term::Variable(_), _) => unify_variable(term1, term2, bindings),
        (_, Term::Variable(_)) => unify(term2, term1, bindings), // todo Double check this is OK
        _ => false,
    }
}

// The first argument is always a Term::Variable()
fn unify_variable(term1: Rc<Term>, term2: Rc<Term>, bindings: Rc<Bindings>) -> bool {
    let t1 = bindings.instantiate(term1);
    let t2 = bindings.instantiate(term2);
    if let Term::Variable(Variable(v1, _)) = *t1 {
        if let Term::Variable(Variable(v2, _)) = *t2 {
            if v1 == v2 {
                return true;
            }
        }
        bindings.add(v1, t2);
        true
    } else { // t1 is not a variable
        unify(t1, t2, bindings)
    }
}

#[cfg(test)]
mod unify_variable_tests {
    use std::rc::Rc;
    use crate::term::Term;
    use crate::term_builder::TermBuilder;
    use crate::unify::unify_variable;

    fn unified(term1: Rc<Term>, term2: Rc<Term>, t: &TermBuilder, result: bool) {
        assert_eq!(unify_variable(term1, term2, t.bindings()), result);
    }

    #[test]
    fn var_and_atom() { // (X, a)
        let t = TermBuilder::new();
        unified(t.x(), t.a(), &t, true);
        assert_eq!(t.bindings().instantiate(t.x()), t.a());
    }

    #[test]
    fn var_and_atom_where_var_already_bound_to_that_atom() { // (X, a) where X -> a
        let t = TermBuilder::new();
        t.bindings().add_variable(t.x(), t.a());
        let len = t.bindings().len();
        unified(t.x(), t.a(), &t, true);
        assert_eq!(t.bindings().instantiate(t.x()), t.a());
        assert_eq!(t.bindings().len(), len); // ie, no change
    }

    #[test]
    fn var_and_atom_where_var_already_bound_to_other_atom() { // (X, a) where X -> b
        let t = TermBuilder::new();
        t.bindings().add_variable(t.x(), t.b());
        unified(t.a(), t.x(), &t, false);
    }

    #[test]
    fn var_and_same_var() { // (X, X)
        let t = TermBuilder::new();
        unified(t.x(), t.x(), &t, true);
        assert_eq!(t.bindings().instantiate(t.x()), t.x());
    }

    #[test]
    fn var_and_different_var() { // (X, Y)
        let t = TermBuilder::new();
        unified(t.x(), t.y(), &t, true);
        assert_eq!(t.bindings().instantiate(t.x()), t.y());
    }
}

#[cfg(test)]
mod unify_tests {
    use std::rc::Rc;
    use crate::term::Term;
    use crate::term_builder::TermBuilder;
    use crate::unify::unify;

    fn unified(t1: Rc<Term>, t2: Rc<Term>, t: &TermBuilder, result: bool) {
        assert_eq!(unify(t1, t2, t.bindings()), result);
    }

    #[test]
    fn atoms() {
        let t = TermBuilder::new();
        unified(t.a(), t.a(), &t, true);
        unified(t.a(), t.b(), &t, false);
        t.no_bindings();
    }

    #[test]
    fn ints() {
        let t = TermBuilder::new();
        unified(t.one(), t.one(), &t, true);
        unified(t.one(), t.two(), &t, false);
        t.no_bindings();
    }

    #[test]
    fn variables_only() {
        let t = TermBuilder::new();
        unified(t.x(), t.x(), &t, true);
        unified(t.x(), t.y(), &t, true);
        unified(t.y(), t.x(), &t, true);
        t.bound_to(t.x(), t.y());
    }

    #[test]
    fn variable_to_atom() {
        let t = TermBuilder::new();
        unified(t.x(), t.a(), &t, true);
        unified(t.a(), t.y(), &t, true);
        t.bound_to(t.x(), t.a());
        t.bound_to(t.y(), t.a());
    }

    #[test]
    fn compounds() {
        let t = TermBuilder::new();
        let fa2 = Term::compound1("f", t.a());
        unified(t.fa(), t.fa(), &t, true);
        unified(t.fa(), fa2, &t, true);
        unified(t.fa(), t.fb(), &t, false);
        t.no_bindings();
    }

    #[test]
    fn failing_simple_unification() {
        let t = TermBuilder::new();
        unified(t.a(), t.one(), &t, false);
        unified(t.one(), t.a(), &t, false);
        unified(t.a(), t.fa(), &t, false);
        unified(t.fa(), t.a(), &t, false);
        unified(t.one(), t.fa(), &t, false);
        unified(t.fa(), t.one(), &t, false);
        unified(t.fa(), t.fb(), &t, false);
        unified(t.fa(), t.faa(), &t, false);
        unified(t.fa(), t.ga(), &t, false);
        t.no_bindings();
    }

    #[test]
    fn bind_one_level() {
        let t = TermBuilder::new();
        unified(t.a(), t.x(), &t, true);
        t.bound_to(t.x(), t.a());

       unified(t.one(), t.y(), &t, true);
        t.bound_to(t.y(), t.one());

       unified(t.fab(), t.z(), &t, true);
        t.bound_to(t.z(), t.fab());
    }

    #[test]
    fn bind_multiple_variables() {
        /*
          f(a, b).
          ?- f(X, Y)
         */
        let t = TermBuilder::new();
        unified(t.fab(), t.fxy(), &t, true);
        t.bound_to(t.x(), t.a());
        t.bound_to(t.y(), t.b());
    }

    #[test]
    fn bind_multiple_variables_bidirectional() {
        /*
          f(b, Y).
          ?- f(X, a)
         */
        let t = TermBuilder::new();
        let term1 = Term::compound("f", vec![t.b(), t.y()]);
        unified(term1, t.fxa(), &t, true);
        t.bound_to(t.x(), t.b());
        t.bound_to(t.y(), t.a());
    }

    #[test]
    fn bind_triple() {
        /*
           f(a, Y).
           ?- f(X, X)
          */
        let t = TermBuilder::new();
        let term1 = Term::compound("f", vec![t.a(), t.y()]);
        unified(term1, t.fxx(), &t, true);
        t.bound_to(t.x(), t.a());
        t.bound_to(t.y(), t.a());
    }
}
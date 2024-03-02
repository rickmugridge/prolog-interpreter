use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::bindings::Bindings;
use crate::variable::Variable;

const LIST_COMPOUND: &str = "_list";
const EMPTY_LIST_COMPOUND: &str = "_emptyList";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Term {
    Atom(String),
    Int(isize),
    Variable(Variable),
    CompoundTerm(String, Vec<Rc<Term>>),
}

impl Term {
    pub fn int(i: isize) -> Rc<Term> {
        Rc::new(Term::Int(i))
    }

    pub fn atom(s: &str) -> Rc<Term> {
        Rc::new(Term::Atom(s.to_string()))
    }

    pub fn compound1(functor: &str, argument: Rc<Term>) -> Rc<Term> {
        Rc::new(Term::CompoundTerm(functor.to_string(), vec![argument]))
    }

    pub fn compound(functor: &str, arguments: Vec<Rc<Term>>) -> Rc<Term> {
        Rc::new(Term::CompoundTerm(functor.to_string(), arguments))
    }

    pub fn var(name: &str, bindings: Rc<Bindings>) -> Rc<Term> {
        Rc::new(Term::Variable(Variable::new_named(bindings, name)))
    }

    pub fn var_unnamed(bindings: Rc<Bindings>) -> Rc<Term> {
        Rc::new(Term::Variable(Variable::new(bindings)))
    }

    pub fn var_full(name: &str, i: isize) -> Rc<Term> {
        Rc::new(Term::Variable(Variable(i, Some(name.to_string()))))
    }

    pub fn empty_list() -> Rc<Term> {
        Term::atom(EMPTY_LIST_COMPOUND)
    }

    pub fn list(head: Rc<Term>, tail: Rc<Term>) -> Rc<Term> {
        Term::compound(LIST_COMPOUND, vec![head, tail])
    }

    pub fn make_list(list: Vec<Rc<Term>>) -> Rc<Term> {
        let mut result = Term::empty_list();
        for t in list.into_iter().rev() {
            result = Term::list(t, result);
        }
        result
    }

    pub fn contains_variables(&self) -> bool {
        match self {
            Term::Atom(_) => false,
            Term::Int(_) => false,
            Term::Variable(_) => true,
            Term::CompoundTerm(_, args) =>
                args.iter().any(|arg| arg.contains_variables())
        }
    }

    pub fn find_distinct_variables(terms: Vec<Rc<Term>>) -> HashSet<Rc<Term>> {
        let mut set: HashSet<Rc<Term>> = HashSet::new();
        let _: Vec<_> = terms.iter().map(|term| Term::find_variables_set(term.clone(), &mut set)).collect();
        set
    }

    fn find_variables_set(term: Rc<Term>, set: &mut HashSet<Rc<Term>>) {
        match term.clone().as_ref() {
            Term::Atom(_) => {}
            Term::Int(_) => {}
            Term::Variable(_) => { set.insert(term); }
            Term::CompoundTerm(_, args) =>
                args.iter()
                    .for_each(|arg| Term::find_variables_set(arg.clone(), set))
        }
    }

    fn fmt_list(args: &[Rc<Term>], f: &mut Formatter<'_>) -> std::fmt::Result {
        // Eg vec![atom(1), list(atom(2), vec![list(atom(3), emptyList])]
        std::fmt::Display::fmt(&args[0], f)?;
        match args[1].clone().as_ref() {
            Term::Atom(s) if s == EMPTY_LIST_COMPOUND => { Ok(()) }
            t @ Term::CompoundTerm(functor, args2) => {
                if functor == &LIST_COMPOUND.to_string() {
                    f.write_str(",")?;
                    Term::fmt_list(args2, f)
                } else {
                    f.write_str("|")?;
                    std::fmt::Display::fmt(&t, f)
                }
            }
            t => {
                f.write_str("|")?;
                std::fmt::Display::fmt(&t, f)
            }
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Atom(s) if s == EMPTY_LIST_COMPOUND => f.write_str("[]"),
            Term::Atom(s) => { f.write_str(s) }
            Term::Int(i) => f.write_str(&i.to_string()),
            Term::Variable(v) => std::fmt::Display::fmt(&v, f),
            Term::CompoundTerm(s, args)  if s == LIST_COMPOUND => {
                f.write_str("[")?;
                Term::fmt_list(args, f)?;
                f.write_str("]")
            }
            Term::CompoundTerm(s, args) => {
                f.write_str(s)?;
                f.write_str("(")?;
                for (i, arg) in args.iter().enumerate() {
                    std::fmt::Display::fmt(arg, f)?;
                    if i < args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")
            }
        }
    }
}

#[cfg(test)]
mod test_display {
    use crate::term::{Term};
    use crate::bindings::Bindings;

    #[test]
    fn simple_ones() {
        let bindings = Bindings::new();
        assert_eq!(Term::int(3).to_string(), "3");
        assert_eq!(Term::atom("a").to_string(), "a");
        assert_eq!(Term::var("X", bindings.clone()).to_string(), "X");
        assert_eq!(Term::var_unnamed(bindings.clone()).to_string(), "X2");
        assert_eq!(Term::var_unnamed(bindings).to_string(), "X3");
    }

    #[test]
    fn compound() {
        let bindings = Bindings::new();
        assert_eq!(Term::compound1("f", Term::atom("a")).to_string(), "f(a)");
        assert_eq!(Term::compound(
            "f",
            vec![Term::atom("a"), Term::atom("b"), Term::var("X", bindings)],
        ).to_string(), "f(a, b, X)");
    }

    #[test]
    fn empty_list() {
        assert_eq!(Term::empty_list().to_string(), "[]");
    }

    #[test]
    fn list() {
        let term = Term::make_list(vec![Term::int(1), Term::int(2), Term::int(3)]);
        assert_eq!(term.to_string(), "[1,2,3]");
    }

    #[test]
    fn bar_list() {
        let bindings = Bindings::new();
        let term = Term::list(Term::int(1), Term::var("X", bindings));
        assert_eq!(term.to_string(), "[1|X]");
    }

    #[test]
    fn bar_list2() {
        let bindings = Bindings::new();
        let term =
            Term::list(Term::int(1),
                       Term::list(Term::int(2), Term::var("X", bindings)),
            );
        assert_eq!(term.to_string(), "[1,2|X]");
    }
}

#[cfg(test)]
mod test_contains_variable {
    use crate::term::Term;
    use crate::bindings::Bindings;

    #[test]
    fn simple_ones() {
        let bindings = Bindings::new();
        assert!(!Term::int(3).contains_variables());
        assert!(!Term::atom("a").contains_variables());
        assert!(Term::var("X", bindings.clone()).contains_variables());
        assert!(Term::var_unnamed(bindings.clone()).contains_variables());
        assert!(Term::var_unnamed(bindings).contains_variables());
    }

    #[test]
    fn compound() {
        let bindings = Bindings::new();
        assert!(!Term::compound1("f", Term::atom("a")).contains_variables());
        assert!(Term::compound(
            "f",
            vec![Term::atom("a"), Term::atom("b"), Term::var("X", bindings)],
        ).contains_variables());
    }
}

#[cfg(test)]
mod test_find_distinct_variables {
    use std::collections::HashSet;
    use crate::term::Term;
    use crate::bindings::Bindings;

    #[test]
    fn none() {
        let i = Term::int(3);
        assert_eq!(Term::find_distinct_variables(vec![i.clone()]), HashSet::from([]));

        let a = Term::atom("a");
        assert_eq!(Term::find_distinct_variables(vec![a.clone()]), HashSet::from([]));

        let f = Term::compound1("f", Term::atom("a"));
        assert_eq!(Term::find_distinct_variables(vec![f.clone()]), HashSet::from([]));
    }

    #[test]
    fn just_variable() {
        let new_bindings = Bindings::new();
        let x = Term::var("X", new_bindings.clone());
        assert_eq!(Term::find_distinct_variables(vec![x.clone()]), HashSet::from([x]));
    }

    #[test]
    fn compound_with_variables() {
        let new_bindings = Bindings::new();
        let y = Term::var("Y", new_bindings.clone());
        let f = Term::compound("f", vec![y.clone(), y.clone()]);
        assert_eq!(Term::find_distinct_variables(vec![f.clone()]), HashSet::from([y]));
    }
}

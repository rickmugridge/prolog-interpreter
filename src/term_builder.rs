use std::rc::Rc;
use crate::term::Term;
use crate::bindings::Bindings;
use crate::clause::Clause;
use crate::database::Database;

pub struct TermBuilder {
    bindings: Rc<Bindings>,
    a: Rc<Term>,
    b: Rc<Term>,
    one: Rc<Term>,
    two: Rc<Term>,
    x: Rc<Term>,
    y: Rc<Term>,
    z: Rc<Term>,
    u: Rc<Term>,
    v: Rc<Term>,
    fa: Rc<Term>,
    // f(a)
    fb: Rc<Term>,
    // f(b)
    faa: Rc<Term>,
    // f(a,a)
    fab: Rc<Term>,
    // f(a,b)
    fxy: Rc<Term>,
    // f(X,Y)
    fxx: Rc<Term>,
    // f(X,Y)
    fxa: Rc<Term>,
    // f(X,a)
    ga: Rc<Term>, // g(a)
}

impl TermBuilder {
    pub fn new() -> Self {
        let bindings = Bindings::new();
        let a = Term::atom("a");
        let b = Term::atom("b");
        let one = Term::int(1);
        let two = Term::int(2);
        let x = Term::var("X", bindings.clone());
        let y = Term::var("Y", bindings.clone());
        let z = Term::var("Z", bindings.clone());
        let u = Term::var("U", bindings.clone());
        let v = Term::var("V", bindings.clone());
        let fa = Term::compound1("f", a.clone());
        let fb = Term::compound1("f", b.clone());
        let faa = Term::compound("f", vec![a.clone(), a.clone()]);
        let fab = Term::compound("f", vec![a.clone(), b.clone()]);
        let fxy = Term::compound("f", vec![x.clone(), y.clone()]);
        let fxx = Term::compound("f", vec![x.clone(), x.clone()]);
        let fxa = Term::compound("f", vec![x.clone(), a.clone()]);
        let ga = Term::compound("g", vec![a.clone()]);
        Self { bindings, a, b, one, two, x, y, z, u, v, fa, fb, faa, fab, fxy, fxx, fxa, ga }
    }

    pub fn _var(&self, name: &str) -> Rc<Term> {
        Term::var(name, self.bindings())
    }

    pub fn bindings(&self) -> Rc<Bindings> {
        self.bindings.clone()
    }

    pub fn bound_to(&self, term: Rc<Term>, bound_to: Rc<Term>) {
        assert_eq!(self.bindings().instantiate(term), bound_to);
    }

    pub fn database(&self, clauses: Vec<Rc<Clause>>) -> Database {
        Database::new(clauses, self.bindings.clone())
    }

    pub fn no_bindings(&self) {
        assert_eq!(self.bindings.len(), 0);
    }

    pub fn a(&self) -> Rc<Term> {
        self.a.clone()
    }

    pub fn b(&self) -> Rc<Term> {
        self.b.clone()
    }

    pub fn one(&self) -> Rc<Term> {
        self.one.clone()
    }

    pub fn two(&self) -> Rc<Term> {
        self.two.clone()
    }

    pub fn x(&self) -> Rc<Term> {
        self.x.clone()
    }

    pub fn y(&self) -> Rc<Term> {
        self.y.clone()
    }

    pub fn z(&self) -> Rc<Term> {
        self.z.clone()
    }

    pub fn u(&self) -> Rc<Term> {
        self.u.clone()
    }

    pub fn v(&self) -> Rc<Term> {
        self.v.clone()
    }

    pub fn fa(&self) -> Rc<Term> {
        self.fa.clone()
    }

    pub fn fb(&self) -> Rc<Term> {
        self.fb.clone()
    }

    pub fn faa(&self) -> Rc<Term> {
        self.faa.clone()
    }

    pub fn fab(&self) -> Rc<Term> {
        self.fab.clone()
    }

    pub fn fxy(&self) -> Rc<Term> {
        self.fxy.clone()
    }

    pub fn fxx(&self) -> Rc<Term> {
        self.fxx.clone()
    }

    pub fn fxa(&self) -> Rc<Term> {
        self.fxa.clone()
    }

    pub fn ga(&self) -> Rc<Term> {
        self.ga.clone()
    }
}
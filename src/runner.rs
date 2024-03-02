use std::rc::Rc;
use crate::bindings::Bindings;
use crate::database::Database;
use crate::parse_clauses::{clauses_parser, query_parser};
use crate::run::{Instantiation, run};
use crate::static_context::StaticContext;

pub struct Runner {
    bindings: Rc<Bindings>,
    database: Database,
}

impl Runner {
    pub fn new(src: &str) -> Self {
        let bindings = Bindings::new();
        let static_context = StaticContext::new(bindings.clone());
        let clauses = clauses_parser(src, static_context.clone()).expect("cannot be Err");
        let database = Database::new(clauses, bindings.clone());
        Self { bindings, database }
    }

    pub fn query<'a>(&'a self, query_src: &'a str) -> impl Iterator<Item=Instantiation> + Sized + 'a {
        let static_context = StaticContext::new(self.bindings.clone());
        let query = query_parser(query_src, static_context)
            .expect("cannot be Err");
        run(query, &self.database, self.bindings.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::rc::Rc;
    use crate::run::Instantiation;
    use crate::runner::Runner;
    use crate::term::Term;
    use crate::term_builder::TermBuilder;

    fn next(r: &mut (impl Iterator<Item=Instantiation> + Sized), hash_set: Vec<(String, Rc<Term>)>) {
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from_iter(hash_set)
        });
    }

    #[test]
    fn two_outcomes() {
        let src = "
         f(a,a).
         f(a,b).
         r(U,V) :- f(U,V).
         ";
        let query_src = "?- r(Y,X).";
        // yes, Y = a, X = a || Y = a, X = b.

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let t = TermBuilder::new();
        let y = Term::var_full("Y", 3);
        let x = Term::var_full("X", 4);
        next(&mut r, vec![
            (y.to_string(), t.a()),
            (x.to_string(), t.a()),
        ]);
        next(&mut r, vec![
            (y.to_string(), t.a()),
            (x.to_string(), t.b()),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn two_step() {
        let src = "
         f(a,a).
         r(U,V) :- f(U,V).
         s(M,N) :- r(M,N).
         ";
        let query_src = "?- s(X,a).";
        // yes, X = a.

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let t = TermBuilder::new();
        let x = Term::var_full("X", 5);
        next(&mut r, vec![
            (x.to_string(), t.a()),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn list_append_empty_lists() {
        let src = "
         append(X, [], X).
         append(X, [Head|Rest], [Head|Rest2]) :- append(X, Rest, Rest2).
         ";
        let query_src = "?- append([], [], Both).";
        // yes, Both = [].

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let both = Term::var_full("Both", 5);
        next(&mut r, vec![
            (both.to_string(), Term::empty_list()),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn list_append__list_to_empty_list() {
        let src = "
         append(X, [], X).
         append(X, [Head|Rest], [Head|Rest2]) :- append(X, Rest, Rest2).
         ";
        let query_src = "?- append([1],[],Both).";
        // yes, Both = [1].

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let x = Term::var_full("Both", 5);
        next(&mut r, vec![
            (x.to_string(), Term::make_list(vec![Term::int(1)])),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn list_append_empty_list_to_list() {
        let src = "
         append([], List, List).
         append([Head|Tail], List, [Head|Rest]) :- append(Tail, List, Rest).
         ";
        let query_src = "?- append([1],[2],Both).";
        // yes, Both = [1, 2].

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let both = Term::var_full("Both", 5);
        next(&mut r, vec![
            (both.to_string(), Term::make_list(vec![Term::int(1), Term::int(2)])),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn list_append() {
        let src = "
         append([], List, List).
         append([Head|Tail], List, [Head|Rest]) :- append(Tail, List, Rest).
         ";
        let query_src = "?- append([1, 2],[3, 4],Both).";
        // yes, Both = [1, 2, 3, 4].

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let both = Term::var_full("Both", 5);
        next(&mut r, vec![
            (both.to_string(), Term::make_list(vec![
                Term::int(1), Term::int(2), Term::int(3), Term::int(4)])),
        ]);
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn list_append_generator() {
        let src = "
         append([], List, List).
         append([Head|Tail], List, [Head|Rest]) :- append(Tail, List, Rest).
         ";
        let query_src = "?- append(X, Y, [1, 2]).";
        // yes, X = [], Y = [1, 2].
        // yes, X = [1], Y = [2].
        // yes, X = [1, 2], Y = [].

        let runner = Runner::new(src);
        let mut r = runner.query(query_src);
        let x = Term::var_full("X", 5);
        let y = Term::var_full("Y", 6);
        next(&mut r, vec![
            (x.to_string(), Term::make_list(vec![])),
            (y.to_string(), Term::make_list(vec![Term::int(1), Term::int(2)])),
        ]);
        next(&mut r, vec![
            (x.to_string(), Term::make_list(vec![Term::int(1)])),
            (y.to_string(), Term::make_list(vec![Term::int(2)])),
        ]);
        next(&mut r, vec![
            (x.to_string(), Term::make_list(vec![Term::int(1), Term::int(2)])),
            (y.to_string(), Term::make_list(vec![])),
        ]);
        assert_eq!(r.next().is_none(), true);
    }
}
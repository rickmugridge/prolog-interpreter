use std::collections::HashSet;
use std::iter;
use std::rc::Rc;
use crate::term::{Term};
use crate::bindings::Bindings;
use crate::clause::Clause;
use crate::database::Database;
use crate::substitution::Substitution;
use crate::unify::unify;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instantiation {
    pub(crate) vars: HashSet<(String, Rc<Term>)>,
}

pub fn run(query: Vec<Rc<Term>>, database: &Database, bindings: Rc<Bindings>) -> impl Iterator<Item=Instantiation> + '_ {
    let query_variables = Term::find_distinct_variables(query.clone());
    run_body(query, database, bindings)
        .map(move |temp_bindings| resolve_instantiations(&query_variables, temp_bindings.clone()))
}

pub fn run_query(query: Rc<Term>,
                 database: &Database,
                 outer_bindings: Rc<Bindings>) -> impl Iterator<Item=Rc<Bindings>> + '_ {
    database.matches()
        .filter_map(move |clause| {
            let bindings = Bindings::stack(outer_bindings.clone());
            let rewritten_clause = substitute(clause, bindings.clone());
            println!("?- {} on db clause: {}", query.clone(), rewritten_clause);
            let rewritten_clause_head = rewritten_clause.head.clone();
            let unified = unify(query.clone(), rewritten_clause_head, bindings.clone());
            if unified {
                println!("    -> Unified head: {}", bindings.clone());
                Some((rewritten_clause.body.clone(), bindings))
            } else {
                println!("    -> Failed to unify head");
                None
            }
        })
        .flat_map(|(body, bindings)| {
            run_body(body, database, bindings)
            /*            run_body22(database, &mut body.iter(), bindings) // todo cannot return value referencing temporary value
                        run_body22(database, &mut body.into_iter(), bindings) // todo does into_iter() help???
            */
        })
}

// todo later consider passing the body in as an Iterator or a slice
pub fn run_body<'a>(body: Vec<Rc<Term>>, database: &'a Database, bindings: Rc<Bindings>) -> Box<dyn Iterator<Item=Rc<Bindings>> + 'a> {
    if body.is_empty() {
        Box::new(iter::once(bindings.clone()))
    } else {
        println!("    -> Run_body: {:?}", body.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", "));
        let run_next = run_query(body[0].clone(), database, bindings);
        Box::new(run_next.flat_map(move |new_bindings| {
            let remaining_body: Vec<Rc<Term>> = body.iter().skip(1).cloned().collect();
            run_body(remaining_body, database, new_bindings)
        }))
    }
}

// todo The following tries to use an Iterator, but got stuck due to temporary value. Likewise with using a slice
// todo Could try creating the iterator in here, but likely problems with recursion
/*fn run_body22<'a, I>(database: &'a Database, body: &'a mut I, bindings: Rc<Bindings>) -> Box<dyn Iterator<Item=Rc<Bindings>> + 'a>
    where I: Iterator<Item=&'a Rc<Term>> {
    Box::new(iter::once(bindings.clone()))
}
*/

fn substitute(clause: &Rc<Clause>, outer_bindings: Rc<Bindings>) -> Rc<Clause> {
    let substitution = Substitution::new(outer_bindings.clone());
    substitution.map_clause((*clause).clone())
}

fn resolve_instantiations(query_variables: &HashSet<Rc<Term>>, bindings: Rc<Bindings>) -> Instantiation {
    let vars: HashSet<(String, Rc<Term>)> = query_variables.iter()
        .map(|variable| (
            variable.to_string(),
            bindings.instantiate(variable.clone())
        ))
        .collect();
    Instantiation { vars }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::term::{Term};
    use crate::clause::Clause;
    use crate::run::{Instantiation, run};
    use crate::term_builder::TermBuilder;

    #[test]
    fn no() {
        /*
          ?- a.
          => no
       */
        let t = TermBuilder::new();
        let database = &t.database(vec![]);
        let mut r = run(vec![t.a().clone()], database, t.bindings());
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn single_match_no_variables() {
        /*
           a.
           ?- a.
           => yes
        */
        let t = TermBuilder::new();
        let database = &t.database(vec![Clause::fact(t.a())]);
        let mut r = run(vec![t.a()], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation { vars: HashSet::from([]) });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn single_match_with_variable() {
        /*
         a.
         ?- X.
         => yes, X = a.
         */
        let t = TermBuilder::new();
        let database = &t.database(vec![Clause::fact(t.a())]);

        let mut r = run(vec![t.x()], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"),
                   Instantiation { vars: HashSet::from([(t.x().to_string(), t.a())]) });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn double_match_with_variable() {
        /*
         a.
         b.
         ?- X.
         => yes, X = a || X = b.
         */
        let t = TermBuilder::new();
        let database = &t.database(vec![
            Clause::fact(t.a()),
            Clause::fact(t.b()),
        ]);

        let mut r = run(vec![t.x()], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.a())])
        });
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.b())])
        });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn two_variables() {
        /*
         f(a,b).
         ?- f(X,Y).
         => yes, X = a, Y =b.
         */
        let t = TermBuilder::new();
        let database = &t.database(vec![
            Clause::fact(t.fab()),
        ]);

        let mut r = run(vec![t.fxy()], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([
                (t.x().to_string(), t.a()),
                (t.y().to_string(), t.b())
            ])
        });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn double_match_variable() {
        /*
         f(a,a).
         f(a,b).
         ?- f(X,X).
         => yes, X = a.
         */
        let t = TermBuilder::new();
        let database = &t.database(vec![
            Clause::fact(t.faa()),
            Clause::fact(t.fab()),
        ]);

        let mut r = run(vec![t.fxx()], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.a())])
        });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn double_partial_match_variable() {
        /*
         f(a,a).
         f(a,b).
         ?- f(a,X).
         => yes, X = a || X = b.
         */
        let t = TermBuilder::new();
        let database = &t.database(vec![
            Clause::fact(t.faa()),
            Clause::fact(t.fab()),
        ]);
        let fax = Term::compound("f", vec![t.a(), t.x()]);

        let mut r = run(vec![fax], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.a())])
        });
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.b())])
        });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn one_step_rule_binding_x() {
        /*
         f(a,a).
         f(a,b).
         r(U,V) :- f(U,V)
         ?- r(a,X).
         => yes, X = a || X = b.
         */
        let t = TermBuilder::new();
        let rule = Clause::rule(
            Term::compound("r", vec![t.u(), t.v()]),
            vec![Term::compound("f", vec![t.u(), t.v()])]);
        let database = &t.database(vec![
            Clause::fact(t.faa()),
            Clause::fact(t.fab()),
            rule,
        ]);

        let query = Term::compound("r", vec![t.a(), t.x()]);
        let mut r = run(vec![query], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.a())])
        });
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([(t.x().to_string(), t.b())])
        });
        assert_eq!(r.next().is_none(), true);
    }

    #[test]
    fn one_step_rule_binding_x_and_y() {
        /*
         f(a,a).
         f(a,b).
         r(U,V) :- f(U,V)
         ?- r(Y,X).
         => yes, Y = a, X = a || Y = a, X = b.
         */
        let t = TermBuilder::new();
        let rule = Clause::rule(
            Term::compound("r", vec![t.u(), t.v()]),
            vec![Term::compound("f", vec![t.u(), t.v()])]);
        let database = &t.database(vec![
            Clause::fact(t.faa()),
            Clause::fact(t.fab()),
            rule,
        ]);

        let query = Term::compound("r", vec![t.y(), t.x()]);
        let mut r = run(vec![query], database, t.bindings());
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([
                (t.y().to_string(), t.a()),
                (t.x().to_string(), t.a())
            ])
        });
        assert_eq!(r.next().expect("Was not Some"), Instantiation {
            vars: HashSet::from([
                (t.y().to_string(), t.a()),
                (t.x().to_string(), t.b())
            ])
        });
        assert_eq!(r.next().is_none(), true);
    }
}
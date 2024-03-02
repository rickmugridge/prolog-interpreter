use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;
use crate::clause::Clause;
use crate::lex::{lex, Lex};
use crate::parse_term::{parse_term, remaining};
use crate::static_context::StaticContext;
use crate::term::Term;

pub fn clauses_parser(src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Clause>>, String> {
    let tokens = lex(src.to_string())?;
    let mut tokens = tokens.iter().peekable();
    let result = parse_clauses(&mut tokens, src, static_context)?;
    let remainder: Vec<_> = tokens.collect();
    if remainder.is_empty() {
        Ok(result)
    } else {
        Err(format!("result is {:?} but remaining tokens: {:?}", result, remaining(remainder)))
    }
}

fn parse_clauses(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Clause>>, String> {
    let mut clauses: Vec<Rc<Clause>> = vec![];
    while tokens.peek().is_some() {
        let head = parse_term(tokens, src, static_context.clone())?;
        if let Some(token) = tokens.next() {
            match token {
                Lex::Implies => {
                    let body = parse_body(tokens, src, static_context.clone())?;
                    clauses.push(Clause::rule(head, body));
                }
                Lex::FullStop => {
                    clauses.push(Clause::fact(head));
                }
                t => return Err(format!("Expected :- or '.', but got {}", t))
            }
        } else {
            return Err("Expected :- or '.', but got nothing".to_string());
        }
    }
    Ok(clauses)
}

fn parse_body(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Term>>, String> {
    let mut body: Vec<Rc<Term>> = vec![];
    loop {
        let term = parse_term(tokens, src, static_context.clone())?;
        body.push(term);
        match tokens.next() {
            Some(Lex::Comma) => {}
            Some(Lex::FullStop) => { break; }
            Some(lex) => { return Err(format!("Expected a ',' or '.' but got a {} ", lex)); }
            None => { return Err("Expected a ',' or '.' following a term, but no more tokens.".to_string()); }
        }
    }
    Ok(body)
}

pub fn query_parser(src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Term>>, String> {
    let tokens = lex(src.to_string())?;
    let mut tokens = tokens.iter().peekable();
    let result = parse_query(&mut tokens, src, static_context)?;
    let remainder: Vec<_> = tokens.collect();
    if remainder.is_empty() {
        Ok(result)
    } else {
        Err(format!("result is {:?} but remaining tokens: {:?}", result, remaining(remainder)))
    }
}

pub fn parse_query(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Term>>, String> {
    if let Some(token) = tokens.next() {
        if token == &Lex::Query {
            return parse_body(tokens, src, static_context.clone());
        }
    }
    Err("Expected a query: '?- term1, term2.'".to_string())
}

#[cfg(test)]
pub mod tests {
    use crate::term_builder::TermBuilder;
    use super::*;

    #[test]
    fn fact_clause_with_atom() {
        let static_context = StaticContext::new_all();
        let result = clauses_parser("a.", static_context.clone()).expect("Ok");
        assert_eq!(result, vec![
            Clause::rule(Term::atom("a"), vec![])
        ]);
    }

    #[test]
    fn fact_clause_with_compound() {
        let static_context = StaticContext::new_all();
        let result = clauses_parser("a(1).", static_context.clone()).expect("Ok");
        assert_eq!(result, vec![
            Clause::rule(
                Term::compound("a", vec![Term::int(1)]),
                vec![])
        ]);
    }

    #[test]
    fn interesting_rule() {
        let static_context = StaticContext::new_all();
        let result = clauses_parser("a(X,a) :- b(X).", static_context.clone()).expect("Ok");
        let x = Term::var_full("X", 1);
        let a = Term::atom("a");
        let axa = Term::compound("a", vec![x.clone(), a]);
        let bx = Term::compound("b", vec![x]);
        assert_eq!(result, vec![
            Clause::rule(axa, vec![bx])
        ]);
    }

    #[test]
    fn several_rules_and_facts() {
        let t = TermBuilder::new();
        let static_context = StaticContext::new(t.bindings());
        let src = "a. f(b). f(X,Y) :- f(X,a).";
        let result = clauses_parser(src, static_context.clone()).expect("Ok");
        let x6 = Term::var_full("X", 6);
        let x7 = Term::var_full("Y", 7);
        let fxy = Term::compound("f", vec![x6.clone(), x7]);
        let fxa = Term::compound("f", vec![x6, t.a()]);
        assert_eq!(result, vec![
            Clause::rule(t.a(), vec![]),
            Clause::rule(t.fb(), vec![]),
            Clause::rule(fxy, vec![fxa]),
        ]);
    }
}
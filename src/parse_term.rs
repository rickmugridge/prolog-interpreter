use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;
use crate::lex::{lex, Lex};
use crate::static_context::StaticContext;
use crate::term::Term;

pub fn term_parser(src: &str, static_context: Rc<StaticContext>) -> Result<Rc<Term>, String> {
    let tokens = lex(src.to_string())?;
    let mut tokens = tokens.iter().peekable();
    let result = parse_term(&mut tokens, src, static_context)?;
    let remainder: Vec<_> = tokens.collect();
    if remainder.is_empty() {
        Ok(result)
    } else {
        Err(format!("result is {result} but remaining tokens: {:?}", remaining(remainder)))
    }
}


pub fn parse_term(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Rc<Term>, String> {
    if let Some(token) = tokens.next() {
        match token {
            Lex::Variable(s) => Ok(static_context.clone().var(s)),
            Lex::Integer(j) => Ok(Term::int(*j)),
            Lex::Atom(name) => parse_atom_or_compound(name, tokens, src, static_context),
            Lex::LeftSquare => parse_list(tokens, src, static_context),
            t => Err(format!("Did not expect a: '{t}'")),
        }
    } else {
        Err(format!("Did not expect to end so soon in: '{src}'"))
    }
}

fn parse_list(tokens: &mut Peekable<Iter<Lex>>,
              src: &str,
              static_context: Rc<StaticContext>) -> Result<Rc<Term>, String> {
    if let Some(Lex::RightSquare) = tokens.peek() {
        tokens.next();
        return Ok(Term::empty_list());
    }
    parse_non_empty_list(tokens, src, static_context)
}

fn parse_non_empty_list(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Rc<Term>, String> {
    let mut list: Vec<Rc<Term>> = vec![];
    loop {
        let item = parse_term(tokens, src, static_context.clone())?;
        list.push(item.clone());
        if let Some(token) = tokens.next() {
            match token {
                Lex::RightSquare => {
                    break;
                }
                Lex::Comma => {}
                Lex::Bar => {
                    let tail = parse_term(tokens, src, static_context.clone())?;
                    return match tokens.next() {
                        Some(Lex::RightSquare) => Ok(Term::list(item, tail)),
                        end => Err(format!("Expected ']', but got  {:?}", end)),
                    };
                }
                t => { return Err(format!("Expected ']', ',' or '|', but got  {t}")); }
            }
        } else {
            return Err("Expected ']', ',' or '|', but got nothing".to_string());
        }
    }
    Ok(Term::make_list(list))
}

fn parse_atom_or_compound(name: &String,
                          tokens: &mut Peekable<Iter<Lex>>,
                          src: &str,
                          static_context: Rc<StaticContext>) -> Result<Rc<Term>, String> {
    if let Some(Lex::Left) = tokens.peek() {
        tokens.next();
        let arguments = parse_arguments(tokens, src, static_context)?;
        Ok(Term::compound(name, arguments))
    } else {
        Ok(Term::atom(name))
    }
}

fn parse_arguments(tokens: &mut Peekable<Iter<Lex>>, src: &str, static_context: Rc<StaticContext>) -> Result<Vec<Rc<Term>>, String> {
    if let Some(Lex::Right) = tokens.peek() {
        tokens.next();
        return Ok(vec![]);
    }
    let mut arguments: Vec<Rc<Term>> = vec![];
    loop {
        let arg = parse_term(tokens, src, static_context.clone())?;
        arguments.push(arg);
        match tokens.peek() {
            Some(Lex::Right) => {
                tokens.next();
                break;
            }
            Some(Lex::Comma) => { tokens.next(); }
            t => { return Err(format!("Expected ',' or ') in arguments but got a {:?}", t)); }
        }
    }
    Ok(arguments)
}

pub fn remaining(remainder: Vec<&Lex>) -> String {
    let left = remainder.iter()
        .map(|r| r.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    left
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn atom() {
        let static_context = StaticContext::new_all();
        let result = term_parser("a", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::atom("a"));
    }

    #[test]
    fn integer() {
        let static_context = StaticContext::new_all();
        let result = term_parser("12", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::int(12));
    }

    #[test]
    fn variable() {
        let static_context = StaticContext::new_all();
        let result = term_parser("X", static_context).expect("Ok");
        assert_eq!(result, Term::var_full("X", 1));
    }

    #[test]
    fn nullary_compound() {
        let static_context = StaticContext::new_all();
        let result = term_parser("f()", static_context.clone()).expect("Ok");
        assert_eq!(result, Rc::new(Term::CompoundTerm("f".to_string(), vec![])));
    }

    #[test]
    fn binary_compound() {
        let static_context = StaticContext::new_all();
        let result = term_parser("f(1, 2)", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::compound("f", vec![Term::int(1), Term::int(2)]));
    }

    #[test]
    fn binary_compound_x() {
        let static_context = StaticContext::new_all();
        let result = term_parser("f(X, X)", static_context.clone()).expect("Ok");
        let x = Term::var_full("X", 1);
        assert_eq!(result, Term::compound("f", vec![x.clone(), x]));
    }

    #[test]
    fn nested_compound() {
        let static_context = StaticContext::new_all();
        let result = term_parser("f(g(1))", static_context.clone()).expect("Ok");
        assert_eq!(result,
                   Term::compound("f", vec![
                       Term::compound("g", vec![Term::int(1)])
                   ]));
    }

    #[test]
    fn empty_list() {
        let static_context = StaticContext::new_all();
        let result = term_parser("[]", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::empty_list());
    }

    #[test]
    fn list() {
        let static_context = StaticContext::new_all();
        let result = term_parser("[1,2,3]", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::make_list(vec![Term::int(1), Term::int(2), Term::int(3)]));
    }

    #[test]
    fn bar_list() {
        let static_context = StaticContext::new_all();
        let result = term_parser("[1|X]", static_context.clone()).expect("Ok");
        assert_eq!(result, Term::list(Term::int(1), Term::var_full("X", 1)));
    }
}
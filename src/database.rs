use std::rc::Rc;
use crate::bindings::Bindings;
use crate::clause::Clause;
use crate::substitution::Substitution;

pub struct Database {
    clauses: Vec<Rc<Clause>>,
    substitution: Rc<Substitution>,
}

impl Database {
    // todo Organise terms around f/2, etc for faster lookup
    pub fn new(clauses: Vec<Rc<Clause>>, variables_source: Rc<Bindings>) -> Self {
        Self { clauses, substitution: Rc::new(Substitution::new(variables_source)) }
    }

    pub fn matches(&self) -> impl Iterator<Item=&Rc<Clause>> {
        self.clauses.iter()
    }

    pub fn matches_substituted(&self) -> impl Iterator<Item=Rc<Clause>> + '_ {
        self.clauses.iter().map(|clause| self.substitution.map_clause(clause.clone()))
    }
}

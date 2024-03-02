mod term;

mod substitution;
mod term_builder;
mod variable;
mod clause;
mod bindings;
mod static_context;
mod unify;
mod run;
mod runner;
mod database;
mod lex;
mod parse_term;
mod parse_clauses;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    // use crate::abstract_syntax_tree::{Term};
    // use crate::run::{run, Database};

    #[test]
    fn enable_suite() {}
}

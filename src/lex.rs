use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Lex {
    Atom(String),
    Variable(String),
    String(String),
    Integer(isize),
    Float(f64),
    Boolean(bool),
    Left,
    Right,
    LeftSquare,
    RightSquare,
    Quote,
    FullStop,
    True,
    Implies,
    // :-
    Query,
    // ?-
    Comma,
    Bar,
}

pub fn lex(src: String) -> Result<Vec<Lex>, String> {
    let mut result: Vec<Lex> = vec![];
    let mut chars = src.chars();
    let mut next_char = chars.next();
    while let Some(ch) = next_char {
        match ch {
            ' ' | '\n' => { next_char = chars.next(); }
            '(' => {
                result.push(Lex::Left);
                next_char = chars.next();
            }
            ')' => {
                result.push(Lex::Right);
                next_char = chars.next();
            }
            '[' => {
                result.push(Lex::LeftSquare);
                next_char = chars.next();
            }
            ']' => {
                result.push(Lex::RightSquare);
                next_char = chars.next();
            }
            '.' => {
                result.push(Lex::FullStop);
                next_char = chars.next();
            }
            '\'' => {
                result.push(Lex::Quote);
                next_char = chars.next();
            }
            '<' => {
                next_char = chars.next();
                if let Some('=') = next_char {
                    result.push(Lex::Atom("<=".to_string()));
                    next_char = chars.next();
                } else {
                    result.push(Lex::Atom("<".to_string()));
                }
            }
            '>' => {
                next_char = chars.next();
                if let Some('=') = next_char {
                    result.push(Lex::Atom(">=".to_string()));
                    next_char = chars.next();
                } else {
                    result.push(Lex::Atom(">".to_string()));
                }
            }
            ':' => {
                next_char = chars.next();
                if let Some('-') = next_char {
                    result.push(Lex::Implies);
                    next_char = chars.next();
                } else {
                    result.push(Lex::Atom("-".to_string()));
                }
            }
            '?' => {
                next_char = chars.next();
                if let Some('-') = next_char {
                    result.push(Lex::Query);
                    next_char = chars.next();
                } else {
                    result.push(Lex::Atom("?".to_string()));
                }
            }
            '"' => {
                let mut string = String::new();
                next_char = chars.next();
                while let Some(ch) = next_char {
                    if ch == '"' {
                        break;
                    } else {
                        string.push(ch);
                        next_char = chars.next();
                    }
                }
                result.push(Lex::String(string));
                next_char = chars.next();
            }
            ',' => {
                result.push(Lex::Comma);
                next_char = chars.next();
            }
            '|' => {
                result.push(Lex::Bar);
                next_char = chars.next();
            }
            x if x.is_ascii_digit() || x == '.' => {
                let mut digit_string = String::new();
                digit_string.push(x);
                next_char = chars.next();
                while let Some(ch) = next_char {
                    if ch.is_ascii_digit() || ch == '.' {
                        digit_string.push(ch);
                        next_char = chars.next();
                    } else {
                        break;
                    }
                }
                result.push(parse_number(digit_string)?);
            }
            y => {
                let mut symbol = String::new();
                symbol.push(y);
                let is_variable = y.is_uppercase();
                next_char = chars.next();
                while let Some(ch) = next_char {
                    if ch.is_alphanumeric() || ch == '_' {
                        symbol.push(ch);
                        next_char = chars.next();
                    } else {
                        break;
                    }
                }
                if is_variable {
                    result.push(Lex::Variable(symbol));
                } else if symbol == "true" {
                    result.push(Lex::True);
                } else {
                    result.push(Lex::Atom(symbol));
                }
            }
        }
    }
    Ok(result)
}

impl fmt::Display for Lex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Lex::Left => write!(f, "("),
            Lex::Right => write!(f, ")"),
            Lex::LeftSquare => write!(f, "["),
            Lex::RightSquare => write!(f, "]"),
            Lex::Quote => write!(f, "'"),
            Lex::Atom(s) => write!(f, "{}", s),
            Lex::Integer(i) => write!(f, "{}", i),
            Lex::Float(x) => write!(f, "{}", x),
            Lex::Boolean(b) => write!(f, "{}", b),
            Lex::String(s) => write!(f, "{}", s),
            Lex::Variable(s) => write!(f, "{}", s),
            Lex::FullStop => write!(f, "."),
            Lex::True => write!(f, "true"),
            Lex::Implies => write!(f, ":-"),
            Lex::Query => write!(f, "?-"),
            Lex::Comma => write!(f, ", "),
            Lex::Bar => write!(f, "|"),
        }
    }
}

fn parse_number(digit_string: String) -> Result<Lex, String> {
    if digit_string.contains('.') {
        if let Ok(x) = digit_string.parse::<f64>() {
            Ok(Lex::Float(x))
        } else {
            Err("Invalid float".to_string())
        }
    } else if let Ok(i) = digit_string.parse::<isize>() {
        Ok(Lex::Integer(i))
    } else {
        Err("Invalid int".to_string())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn mixed() {
        assert_eq!(lex("(X? (y, 12) 0.4 true <= >= ') :- ?-[].|".to_string()), Ok(vec![
            Lex::Left,
            Lex::Variable("X".to_string()),
            Lex::Atom("?".to_string()),
            Lex::Left,
            Lex::Atom("y".to_string()),
            Lex::Comma,
            Lex::Integer(12),
            Lex::Right,
            Lex::Float(0.4),
            Lex::True,
            Lex::Atom("<=".to_string()),
            Lex::Atom(">=".to_string()),
            Lex::Quote,
            Lex::Right,
            Lex::Implies,
            Lex::Query,
            Lex::LeftSquare,
            Lex::RightSquare,
            Lex::FullStop,
            Lex::Bar,
        ]));
    }

    #[test]
    fn double_quote() {
        assert_eq!(lex("\"abc\"".to_string()), Ok(vec![Lex::String("abc".to_string())]));
        assert_eq!(lex("\"a --- c\"".to_string()), Ok(vec![Lex::String("a --- c".to_string())]));
    }
}
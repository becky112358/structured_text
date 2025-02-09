use std::fmt;
use std::io::{Error, ErrorKind, Result};

use crate::code::Code;

#[derive(Debug, PartialEq)]
pub struct Expression(String);

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Expression {
    pub fn peel(code: &mut Code) -> Result<Self> {
        let mut expression = String::new();
        let mut open_bracket_count = 0;
        let mut chars = code.chars().peekable();
        while let Some(c) = chars.next() {
            if c.is_alphanumeric()
                || c == '_'
                || c == '-'
                || c == '#'
                || c == '.'
                || c == '*'
                || c == '+'
                || c == ' '
            {
                expression.push(c);
            } else if c == '/' {
                if let Some('/') = chars.peek() {
                    break;
                } else {
                    expression.push(c);
                }
            } else if c == '(' {
                if let Some('*') = chars.peek() {
                    break;
                } else {
                    expression.push(c);
                    open_bracket_count += 1;
                }
            } else if c == ')' {
                if open_bracket_count > 0 {
                    expression.push(c);
                    open_bracket_count -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if expression.trim().is_empty() {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("No expression \n{code}"),
            ))
        } else {
            code.peel(expression.len())?;
            Ok(Self(expression.trim().to_string()))
        }
    }
}

#[cfg(test)]
#[path = "./test_expression.rs"]
mod test_expression;

use std::fmt;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
pub struct Expression(String);

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Expression {
    pub fn peel(remainder: &mut String) -> Result<Self> {
        let mut output = String::new();
        let mut open_bracket_count = 0;
        let mut chars = remainder.chars().peekable();
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
                output.push(c);
            } else if c == '/' {
                if let Some('/') = chars.peek() {
                    break;
                } else {
                    output.push(c);
                }
            } else if c == '(' {
                if let Some('*') = chars.peek() {
                    break;
                } else {
                    output.push(c);
                    open_bracket_count += 1;
                }
            } else if c == ')' {
                if open_bracket_count > 0 {
                    output.push(c);
                    open_bracket_count -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if output.trim().is_empty() {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("No expression \n{remainder}"),
            ))
        } else {
            *remainder = remainder[output.len()..].to_string();
            output = output.trim().to_string();
            Ok(Self(output))
        }
    }
}

#[cfg(test)]
#[path = "./test_expression.rs"]
mod test_expression;

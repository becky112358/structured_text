use std::fmt;
use std::io::{Error, ErrorKind, Result};

use crate::code::Code;

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub struct IdentifierSub(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for IdentifierSub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Identifier {
    pub fn peel(code: &mut Code) -> Result<Self> {
        Ok(Self(peel(code, |c| c.is_alphanumeric() || c == '_')?))
    }
}

impl IdentifierSub {
    pub fn peel(code: &mut Code) -> Result<Self> {
        Ok(Self(peel(code, |c| {
            c.is_alphanumeric() || c == '_' || c == '.'
        })?))
    }
}

fn peel(code: &mut Code, char_allowed: impl Fn(char) -> bool) -> Result<String> {
    let mut output = String::new();
    for c in code.chars() {
        if char_allowed(c) {
            output.push(c);
        } else {
            break;
        }
    }
    if output.is_empty() {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("No identifer or sub-identifier \n{code}"),
        ))
    } else {
        code.peel(output.len())?;
        Ok(output)
    }
}

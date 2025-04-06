use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::code::Code;

use super::Identifier;

#[derive(Debug, PartialEq)]
pub enum Member {
    Named(Identifier),
    Unnamed(u32),
}

impl fmt::Display for Member {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Named(inner) => write!(f, "{inner}"),
            Self::Unnamed(inner) => write!(f, "{inner}"),
        }
    }
}

impl Member {
    pub fn peel(code: &mut Code) -> Result<Self> {
        if let Ok(identifier) = Identifier::peel(code) {
            return Ok(Self::Named(identifier));
        }

        let mut number_as_string = String::new();
        for c in code.chars() {
            if c.is_ascii_digit() {
                number_as_string.push(c);
            } else {
                break;
            }
        }

        if number_as_string.is_empty() {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("No member\n{code}"),
            ))
        } else if let Ok(number) = u32::from_str(&number_as_string) {
            Ok(Self::Unnamed(number))
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("Failed to parse {number_as_string}"),
            ))
        }
    }
}

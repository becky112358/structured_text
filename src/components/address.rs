use std::fmt;
use std::io::{Error, ErrorKind, Result};

use crate::code::Code;

#[derive(Debug, PartialEq)]
pub enum Address {
    I,
    Q,
    M,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I => write!(f, "AT %I*"),
            Self::Q => write!(f, "AT %Q*"),
            Self::M => write!(f, "AT %M*"),
        }
    }
}

impl Address {
    pub fn peel(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix_uppercase("AT")?;
        code_clone = code_clone.trim_start();
        code_clone = code_clone.strip_prefix('%')?;

        let address = match code_clone.chars().next() {
            Some('I') => Address::I,
            Some('Q') => Address::Q,
            Some('M') => Address::M,
            Some(_) | None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find address type\n{code}"),
                ))
            }
        };
        code_clone.peel(1)?;

        *code = code_clone.strip_prefix('*')?;
        Ok(address)
    }
}

#[cfg(test)]
#[path = "./test_address.rs"]
mod test_address;

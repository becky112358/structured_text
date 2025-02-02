use std::fmt;
use std::io::{Error, ErrorKind, Result};

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
    pub fn peel(remainder: &mut String) -> Result<Self> {
        let at = match remainder.to_uppercase().find("AT") {
            Some(0) => 0,
            Some(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Does not start with \"AT\" \n{remainder}"),
                ))
            }
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find \"AT\" \n{remainder}"),
                ))
            }
        };
        let pc = match remainder.find('%') {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find '%' \n{remainder}"),
                ))
            }
        };
        let star = match remainder.find('*') {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find '*' \n{remainder}"),
                ))
            }
        };
        if star <= pc || pc <= at {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("AT %X* is incorrectly ordered \n{remainder}"),
            ));
        }
        let address_value = remainder[pc + 1..star].trim().to_uppercase();
        let address = match address_value.as_str() {
            "I" => Self::I,
            "Q" => Self::Q,
            "M" => Self::M,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Address should be I, Q, or M, got {address_value} \n{remainder}"),
                ))
            }
        };
        *remainder = remainder[star + 1..].to_string();
        Ok(address)
    }
}

#[cfg(test)]
#[path = "./test_address.rs"]
mod test_address;

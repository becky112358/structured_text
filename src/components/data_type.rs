use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::code::Code;

use super::Identifier;

#[derive(Debug, PartialEq)]
pub enum DataType {
    Array(ArrayRange, Box<DataType>),
    String(Option<u16>),
    ReferenceTo(Box<DataType>),
    PointerTo(Box<DataType>),
    ImplicitEnum(Vec<Identifier>),
    Flat(String),
}

#[derive(Debug, PartialEq)]
pub enum ArrayRange {
    LowerUpper(String, String),
    Star,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Array(range, flat) => match range {
                ArrayRange::LowerUpper(lower, upper) => {
                    write!(f, "ARRAY [{lower}..{upper}] OF {flat}")
                }
                ArrayRange::Star => write!(f, "ARRAY [*] OF {flat}"),
            },
            Self::String(inner) => match inner {
                Some(length) => write!(f, "STRING({length})"),
                None => write!(f, "STRING"),
            },
            Self::ReferenceTo(inner) => write!(f, "REFERENCE TO {inner}"),
            Self::PointerTo(inner) => write!(f, "POINTER TO {inner}"),
            Self::ImplicitEnum(inner) => {
                write!(f, "(")?;
                for (i, member) in inner.iter().enumerate() {
                    write!(f, "{member}")?;
                    if i + 1 < inner.len() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Self::Flat(inner) => write!(f, "{inner}"),
        }
    }
}

impl DataType {
    pub fn peel(code: &mut Code) -> Result<Self> {
        if let Ok(array) = Self::peel_array(code) {
            Ok(array)
        } else if let Ok(s) = Self::peel_string(code) {
            Ok(s)
        } else if let Ok(mut code_clone) = code.strip_prefix_uppercase("REFERENCE TO") {
            code_clone = code_clone.trim_start();
            let flat = Self::peel(&mut code_clone)?;
            *code = code_clone;
            Ok(Self::ReferenceTo(Box::new(flat)))
        } else if let Ok(mut code_clone) = code.strip_prefix_uppercase("POINTER TO") {
            code_clone = code_clone.trim_start();
            let flat = Self::peel(&mut code_clone)?;
            *code = code_clone;
            Ok(Self::PointerTo(Box::new(flat)))
        } else if let Ok(implicit_enum) = Self::peel_implicit_enum(code) {
            Ok(implicit_enum)
        } else {
            let mut data_type = String::new();
            for c in code.chars() {
                if c.is_alphanumeric() || c == '_' || c == '#' || c == '.' {
                    data_type.push(c);
                } else {
                    break;
                }
            }
            if data_type.is_empty() {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("No Data Type\n{code}"),
                ))
            } else {
                code.peel(data_type.len())?;
                Ok(Self::Flat(data_type))
            }
        }
    }

    fn peel_array(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix_uppercase("ARRAY")?.trim_start();

        let range_string = code_clone.strip_between_and_trim_inner("[", "]")?;
        let range = if range_string == "*" {
            ArrayRange::Star
        } else {
            match range_string.split_once("..") {
                Some((start, end)) => {
                    ArrayRange::LowerUpper(start.trim().to_string(), end.trim().to_string())
                }
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot parse array range\n{code}"),
                    ))
                }
            }
        };

        code_clone = code_clone
            .trim_start()
            .strip_prefix_uppercase("OF")?
            .trim_start();

        let flat = Self::peel(&mut code_clone)?;

        *code = code_clone;
        Ok(Self::Array(range, Box::new(flat)))
    }

    fn peel_string(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix_uppercase("STRING")?.trim_start();
        match code_clone.strip_between_and_trim_inner("(", ")") {
            Ok(inner) => {
                let length = match u16::from_str(&inner) {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Cannot parse STRING length\n{code}"),
                        ))
                    }
                };
                *code = code_clone;
                Ok(Self::String(Some(length)))
            }
            Err(_) => {
                *code = code_clone;
                Ok(Self::String(None))
            }
        }
    }

    fn peel_implicit_enum(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix('(')?.trim_start();
        let mut members = Vec::new();
        loop {
            let identifier = Identifier::peel(&mut code_clone)?;
            if let Ok(code_clone_stripped) = code_clone.trim_start().strip_prefix(',') {
                code_clone = code_clone_stripped.trim_start();
                members.push(identifier);
                continue;
            } else if let Ok(code_clone_stripped) = code_clone.trim_start().strip_prefix(')') {
                code_clone = code_clone_stripped;
                members.push(identifier);
                break;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse Implicit Enum \n{code}"),
                ));
            }
        }
        *code = code_clone;
        Ok(Self::ImplicitEnum(members))
    }
}

#[cfg(test)]
#[path = "./test_data_type.rs"]
mod test_data_type;

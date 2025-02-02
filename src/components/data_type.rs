use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

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
    pub fn peel(remainder: &mut String) -> Result<Self> {
        if let Ok(array) = Self::peel_array(remainder) {
            Ok(array)
        } else if remainder.to_uppercase().starts_with("STRING") {
            if !remainder["STRING".len()..].trim().starts_with("(") {
                *remainder = remainder["STRING".len()..].to_string();
                return Ok(Self::String(None));
            }

            let length_start = match remainder.find("(") {
                Some(i) => i,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Lost the '('!\n{remainder}"),
                    ))
                }
            };
            let length_stop = match remainder.find(")") {
                Some(i) => i,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot find ')'\n{remainder}"),
                    ))
                }
            };
            let length = match u16::from_str(remainder[length_start + 1..length_stop].trim()) {
                Ok(i) => i,
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot parse STRING length\n{remainder}"),
                    ))
                }
            };
            *remainder = remainder[length_stop + 1..].to_string();
            Ok(Self::String(Some(length)))
        } else if remainder.to_uppercase().starts_with("REFERENCE TO") {
            let mut remainder_clone = remainder["REFERENCE TO".len()..].trim_start().to_string();
            let flat = Self::peel(&mut remainder_clone)?;
            *remainder = remainder_clone;
            Ok(Self::ReferenceTo(Box::new(flat)))
        } else if remainder.to_uppercase().starts_with("POINTER TO") {
            let mut remainder_clone = remainder["POINTER TO".len()..].trim_start().to_string();
            let flat = Self::peel(&mut remainder_clone)?;
            *remainder = remainder_clone;
            Ok(Self::PointerTo(Box::new(flat)))
        } else if let Ok(implicit_enum) = Self::peel_implicit_enum(remainder) {
            Ok(implicit_enum)
        } else {
            let mut data_type = String::new();
            for c in remainder.chars() {
                if c.is_alphanumeric() || c == '_' || c == '#' || c == '.' {
                    data_type.push(c);
                } else {
                    break;
                }
            }
            if data_type.is_empty() {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("No Data Type\n{remainder}"),
                ))
            } else {
                *remainder = remainder[data_type.len()..].to_string();
                Ok(Self::Flat(data_type))
            }
        }
    }

    fn peel_array(remainder: &mut String) -> Result<Self> {
        if remainder.to_uppercase().starts_with("ARRAY") {
            let start = match remainder.find('[') {
                Some(i) => i,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot find '[' \n{remainder}"),
                    ))
                }
            };
            if remainder.len() <= start {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Finishes at '['\n{remainder}"),
                ));
            }
            let end = match remainder[(start + 1)..].find(']') {
                Some(i) => i + start + 1,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot find ']' \n{remainder}"),
                    ))
                }
            };
            if remainder.len() <= end {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Finishes at \"]\"\n{remainder}"),
                ));
            }
            let of = match remainder[(end + 1)..].to_uppercase().find("OF") {
                Some(i) => i + end + 1,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot find \"OF\" \n{remainder}"),
                    ))
                }
            };

            let range = if let Some(mid) = remainder[(start + 1)..end].find("..") {
                ArrayRange::LowerUpper(
                    remainder[(start + 1)..(start + 1 + mid)].trim().to_string(),
                    remainder[(start + 1 + mid + "..".len())..end]
                        .trim()
                        .to_string(),
                )
            } else if remainder[(start + 1)..end].trim() == "*" {
                ArrayRange::Star
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find array range \n{remainder}"),
                ));
            };

            let mut remainder_clone = remainder[of + "OF".len()..].trim_start().to_string();
            let flat = Self::peel(&mut remainder_clone)?;
            *remainder = remainder_clone;
            Ok(Self::Array(range, Box::new(flat)))
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with \"ARRAY\"\n{remainder}"),
            ))
        }
    }

    fn peel_implicit_enum(remainder: &mut String) -> Result<Self> {
        if !remainder.starts_with('(') {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Does not start with '('\n{remainder}",
            ));
        }
        let mut remainder_clone = remainder[1..].trim_start().to_string();
        let mut members = Vec::new();
        loop {
            let identifier = Identifier::peel(&mut remainder_clone)?;
            if let Some(remainder_clone_stripped) = remainder_clone.trim_start().strip_prefix(',') {
                remainder_clone = remainder_clone_stripped.trim_start().to_string();
                members.push(identifier);
                continue;
            } else if let Some(remainder_clone_stripped) =
                remainder_clone.trim_start().strip_prefix(')')
            {
                remainder_clone = remainder_clone_stripped.to_string();
                members.push(identifier);
                break;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse Implicit Enum \n{remainder}"),
                ));
            }
        }
        *remainder = remainder_clone.to_string();
        Ok(Self::ImplicitEnum(members))
    }
}

#[cfg(test)]
#[path = "./test_data_type.rs"]
mod test_data_type;

use std::io::{Error, ErrorKind, Result};

use crate::code::Code;
use crate::dazzle::{self, Dazzle};

use super::{Ether, Expression, Identifier, KEYWORDS};

#[derive(Debug, PartialEq)]
pub struct Value(ValueInner);

#[derive(Debug, PartialEq)]
enum ValueInner {
    Array(Array),
    Struct(Struct),
    String(String),
    Flat(String),
}

impl Dazzle for Value {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        self.0.dazzle(dazzler);
    }
}

impl Dazzle for ValueInner {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        match &self {
            Self::Array(inner) => inner.dazzle(dazzler),
            Self::Struct(inner) => inner.dazzle(dazzler),
            Self::String(inner) => {
                dazzler.indent_or_space(false);
                dazzler.f.push_str(&format!("'{inner}'"));
                dazzler.previous_character = dazzle::PreviousCharacter::Other;
            }
            Self::Flat(inner) => inner.dazzle(dazzler),
        }
    }
}

impl Value {
    pub fn peel(code: &mut Code) -> Result<Self> {
        Ok(Self(ValueInner::peel(code)?))
    }
}

impl ValueInner {
    fn peel(code: &mut Code) -> Result<Self> {
        if let Ok(a) = Array::peel(code) {
            Ok(Self::Array(a))
        } else if let Ok(s) = Struct::peel(code) {
            Ok(Self::Struct(s))
        } else if let Ok(mut code_clone) = code.strip_prefix('\'') {
            let mut value = String::new();
            let mut escape = false;
            for c in code_clone.chars() {
                if !escape && c == '\'' {
                    break;
                } else if !escape && c == '$' {
                    escape = true;
                    value.push(c);
                } else if escape {
                    escape = false;
                    value.push(c);
                } else {
                    value.push(c);
                }
            }
            code_clone.peel(value.len() + '\''.len_utf8())?;
            *code = code_clone;
            Ok(Self::String(value))
        } else {
            let mut value = String::new();
            for c in code.chars() {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '#' || c == '.' {
                    value.push(c);
                } else {
                    break;
                }
            }
            if value.is_empty() || KEYWORDS.contains(&value.to_uppercase().as_str()) {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("No value\n{code}"),
                ))
            } else {
                code.peel(value.len())?;
                Ok(Self::Flat(value))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct Array(Vec<Ether>, Vec<(Expression, Vec<Ether>)>);

impl Dazzle for Array {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        let start_with_newline = dazzler.previous_character == dazzle::PreviousCharacter::LineFeed;
        if start_with_newline {
            dazzler.indentation_count += 1;
        }
        dazzler.indent_or_space(true);
        dazzler.f.push('[');
        dazzler.indentation_count += 1;
        for ether in &self.0 {
            ether.dazzle(dazzler);
        }
        for (i, (value, ethers)) in self.1.iter().enumerate() {
            value.dazzle(dazzler);
            if i + 1 < self.1.len() {
                dazzler.f.push(',');
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
            }
            for ether in ethers {
                ether.dazzle(dazzler);
            }
        }
        dazzler.indentation_count -= 1;
        dazzler.indent_or_space(false);
        dazzler.f.push(']');
        if start_with_newline {
            dazzler.indentation_count -= 1;
        }
    }
}

impl Array {
    fn peel(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix('[')?;

        let ethers_start = Ether::peel(&mut code_clone)?;

        let mut array = Vec::new();
        loop {
            let member = Expression::peel(&mut code_clone)?;
            let mut ethers = Ether::peel(&mut code_clone)?;
            if let Ok(code_clone_stripped) = code_clone.strip_prefix(',') {
                code_clone = code_clone_stripped;
                ethers.extend(Ether::peel(&mut code_clone)?);
                array.push((member, ethers));
            } else {
                array.push((member, ethers));
                break;
            }
        }
        *code = code_clone.strip_prefix(']')?;
        Ok(Self(ethers_start, array))
    }
}

#[derive(Debug, PartialEq)]
struct Struct(Vec<(Identifier, Expression, Vec<Ether>)>);

impl Dazzle for Struct {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        if dazzler.should_split(self, dazzle_singleline) {
            self.dazzle_multiline(dazzler);
        } else {
            dazzle_singleline(self, dazzler);
        }
    }
}

fn dazzle_singleline(s: &Struct, dazzler: &mut dazzle::Dazzler) {
    '('.dazzle(dazzler);
    for (i, (identifier, value, ethers)) in s.0.iter().enumerate() {
        identifier.dazzle(dazzler);
        dazzler.f.push_str(" :=");
        dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
        value.dazzle(dazzler);
        if !&ethers.iter().all(|ether| *ether == Ether::LineFeed) {
            for ether in ethers {
                ether.dazzle(dazzler);
            }
        }
        if i + 1 < s.0.len() {
            dazzler.f.push(',');
            dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
        }
    }
    ')'.dazzle(dazzler);
}

impl Struct {
    fn dazzle_multiline(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indentation_count += 1;
        dazzler.if_not_linefeed_then_linefeed();
        dazzler.indent();
        dazzler.f.push_str("(\n");
        dazzler.indentation_count += 1;
        dazzler.previous_character = dazzle::PreviousCharacter::LineFeed;

        let max_identifier_length = self
            .0
            .iter()
            .map(|(i, _, _)| i.to_string().len())
            .max()
            .unwrap_or(0);

        let width_to_comment_start = self.get_width_to_comment_start(max_identifier_length)
            + dazzle::INDENT_WIDTH as usize * dazzler.indentation_count as usize;

        for (i, (identifier, expression, ethers)) in self.0.iter().enumerate() {
            identifier.dazzle(dazzler);
            for _ in 0..(max_identifier_length - identifier.to_string().len()) {
                dazzler.f.push(' ');
            }
            dazzler.f.push_str(" :=");
            dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
            expression.dazzle(dazzler);
            if i + 1 < self.0.len() {
                ','.dazzle(dazzler);
            }
            for (j, ether) in ethers.iter().enumerate() {
                if j == 0 && ether.is_comment() {
                    let width_current = match dazzler.f.lines().last() {
                        Some(line) => line.len(),
                        None => dazzler.f.len(),
                    };
                    for _ in width_current..width_to_comment_start {
                        dazzler.f.push(' ');
                    }
                }
                ether.dazzle(dazzler);
            }
            dazzler.if_not_linefeed_then_linefeed();
        }
        dazzler.indentation_count -= 1;
        dazzler.indent();
        dazzler.f.push(')');
        dazzler.indentation_count -= 1;
        dazzler.previous_character = dazzle::PreviousCharacter::Other;
    }

    fn get_width_to_comment_start(&self, max_identifier_length: usize) -> usize {
        let mut max_width = 0;
        for (i, (_, expression, ethers)) in self.0.iter().enumerate() {
            match ethers.first() {
                Some(ether) => {
                    if !ether.is_comment() {
                        continue;
                    }
                }
                None => continue,
            }
            let mut dazzler_line = dazzle::Dazzler {
                f: String::new(),
                previous_character: dazzle::PreviousCharacter::LineFeed,
                indentation_count: 0,
            };
            expression.dazzle(&mut dazzler_line);
            let mut this_width = match dazzler_line.f.rsplit_once('\n') {
                Some((_, last_line)) => last_line.len(),
                None => dazzler_line.f.len() + max_identifier_length + " := ".len(),
            };
            if i + 1 < self.0.len() {
                this_width += 1;
            }
            max_width = max_width.max(this_width);
        }
        max_width
    }

    fn peel(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.strip_prefix('(')?.trim_start();
        let mut values = Vec::new();
        loop {
            let identifier = Identifier::peel(&mut code_clone)?;
            code_clone = code_clone.trim_start().strip_prefix_str(":=")?.trim_start();
            let expression = Expression::peel(&mut code_clone)?;
            let mut ethers = Ether::peel(&mut code_clone)?;
            if let Ok(code_clone_stripped) = code_clone.strip_prefix(',') {
                code_clone = code_clone_stripped;
                ethers.extend(Ether::peel(&mut code_clone)?);
                values.push((identifier, expression, ethers));
                continue;
            } else if let Ok(code_clone_stripped) = code_clone.strip_prefix(')') {
                code_clone = code_clone_stripped;
                ethers.extend(Ether::peel(&mut code_clone)?);
                values.push((identifier, expression, ethers));
                break;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse struct value \n{code}"),
                ));
            }
        }
        *code = code_clone;
        Ok(Self(values))
    }
}

#[cfg(test)]
#[path = "./test_value.rs"]
mod test_value;

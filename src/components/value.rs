use std::io::{Error, ErrorKind, Result};

use crate::dazzle::{self, Dazzle};

use super::{Assignment, Ether, Identifier};

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
            Self::Flat(inner) => {
                dazzler.indent_or_space(false);
                dazzler.f.push_str(&inner.to_string());
                dazzler.previous_character = dazzle::PreviousCharacter::Other;
            }
        }
    }
}

impl Value {
    pub fn peel(remainder: &mut String) -> Result<Self> {
        Ok(Self(ValueInner::peel(remainder)?))
    }
}

impl ValueInner {
    fn peel(remainder: &mut String) -> Result<Self> {
        if let Ok(a) = Array::peel(remainder) {
            Ok(Self::Array(a))
        } else if let Ok(s) = Struct::peel(remainder) {
            Ok(Self::Struct(s))
        } else if remainder.starts_with("'") {
            let mut value = String::new();
            let mut escape = false;
            for c in remainder.chars().skip(1) {
                if !escape && c == '\'' {
                    break;
                } else if !escape && c == '$' {
                    escape = true;
                } else if escape {
                    escape = false;
                    value.push(c);
                } else {
                    value.push(c);
                }
            }
            *remainder = remainder[value.len() + 2..].to_string();
            Ok(Self::String(value))
        } else {
            let mut value = String::new();
            let mut array_accessor_count = 0;
            for c in remainder.chars() {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '#' || c == '.' {
                    value.push(c);
                } else if c == '[' {
                    array_accessor_count += 1;
                    value.push(c);
                } else if c == ']' && array_accessor_count > 0 {
                    array_accessor_count -= 1;
                    value.push(c);
                } else {
                    break;
                }
            }
            *remainder = remainder[value.len()..].to_string();
            value = value.to_string();
            Ok(Self::Flat(value))
        }
    }
}

#[derive(Debug, PartialEq)]
struct Array(Vec<Ether>, Vec<(Assignment, Vec<Ether>)>);

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
    fn peel(remainder: &mut String) -> Result<Self> {
        if !remainder.starts_with('[') {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Does not start with '['\n{remainder}",
            ));
        }
        let mut remainder_clone = remainder[1..].to_string();

        let ethers_start = Ether::peel(&mut remainder_clone)?;

        let mut array = Vec::new();
        loop {
            let member = Assignment::peel(&mut remainder_clone, ',', ']')?;
            let mut ethers = Ether::peel(&mut remainder_clone)?;
            if let Some(remainder_clone_stripped) = remainder_clone.strip_prefix(",") {
                remainder_clone = remainder_clone_stripped.to_string();
                ethers.extend(Ether::peel(&mut remainder_clone)?);
                array.push((member, ethers));
            } else {
                array.push((member, ethers));
                break;
            }
        }
        if let Some(remainder_clone_stripped) = remainder_clone.strip_prefix("]") {
            *remainder = remainder_clone_stripped.to_string();
            Ok(Self(ethers_start, array))
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot find ']' \n{remainder}"),
            ))
        }
    }
}

#[derive(Debug, PartialEq)]
struct Struct(Vec<(Identifier, Assignment, Vec<Ether>)>);

impl Dazzle for Struct {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        if self.dazzle_multiline(dazzler) {
            if dazzler.previous_character != dazzle::PreviousCharacter::LineFeed {
                dazzler.f.push('\n');
            }
            dazzler.indentation_count += 1;
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

            for (i, (identifier, assignment, ethers)) in self.0.iter().enumerate() {
                identifier.dazzle(dazzler);
                for _ in 0..(max_identifier_length - identifier.to_string().len()) {
                    dazzler.f.push(' ');
                }
                dazzler.f.push_str(" :=");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                assignment.dazzle(dazzler);
                if i + 1 < self.0.len() {
                    dazzler.f.push(',');
                    dazzler.previous_character = dazzle::PreviousCharacter::Other;
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
                if dazzler.previous_character != dazzle::PreviousCharacter::LineFeed {
                    dazzler.f.push('\n');
                    dazzler.previous_character = dazzle::PreviousCharacter::LineFeed;
                }
            }
            dazzler.indentation_count -= 1;
            dazzler.indent();
            dazzler.f.push(')');
            dazzler.indentation_count -= 1;
            dazzler.previous_character = dazzle::PreviousCharacter::Other;
        } else {
            dazzler.indent_or_space(false);
            dazzler.f.push('(');
            dazzler.previous_character = dazzle::PreviousCharacter::Other;
            for (i, (identifier, value, ethers)) in self.0.iter().enumerate() {
                identifier.dazzle(dazzler);
                dazzler.f.push_str(" :=");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                value.dazzle(dazzler);
                for ether in ethers {
                    ether.dazzle(dazzler);
                }
                if i + 1 < self.0.len() {
                    dazzler.f.push(',');
                }
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
            }
            dazzler.f.push(')');
            dazzler.previous_character = dazzle::PreviousCharacter::Other;
        }
    }
}

impl Struct {
    fn dazzle_multiline(&self, dazzler: &dazzle::Dazzler) -> bool {
        if dazzler.previous_character == dazzle::PreviousCharacter::LineFeed {
            return true;
        }

        let last_line = match dazzler.f.lines().last() {
            Some(line) => line.to_owned(),
            None => dazzler.f.clone(),
        };
        let mut dazzler_this = dazzle::Dazzler {
            f: last_line,
            previous_character: dazzle::PreviousCharacter::Other, // '('
            indentation_count: dazzler.indentation_count,
        };

        for (identifier, assignment, ethers) in &self.0 {
            identifier.dazzle(&mut dazzler_this);
            dazzler_this.previous_character = dazzle::PreviousCharacter::PendingSpace; // ' := '
            assignment.dazzle(&mut dazzler_this);
            dazzler_this.previous_character = dazzle::PreviousCharacter::Other; // ','
            for ether in ethers {
                ether.dazzle(&mut dazzler_this);
            }

            if dazzler_this.f.contains('\n') {
                return true;
            }
        }

        let length = dazzler_this.f.len() + "()".len() + self.0.len() * " := ,".len();
        length > crate::fmt::LINE_LENGTH_LIMIT as usize
    }

    fn get_width_to_comment_start(&self, max_identifier_length: usize) -> usize {
        let mut max_width = 0;
        for (i, (_, assignment, ethers)) in self.0.iter().enumerate() {
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
            assignment.dazzle(&mut dazzler_line);
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

    fn peel(remainder: &mut String) -> Result<Self> {
        if !remainder.starts_with('(') {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Does not start with '('\n{remainder}",
            ));
        }
        let mut remainder_clone = remainder[1..].trim_start().to_string();
        let mut values = Vec::new();
        loop {
            let identifier = Identifier::peel(&mut remainder_clone)?;
            if let Some(remainder_clone_stripped) = remainder_clone.trim_start().strip_prefix(":=")
            {
                remainder_clone = remainder_clone_stripped.trim_start().to_string();
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find \":=\" \n{remainder}"),
                ));
            }
            let assignment = Assignment::peel(&mut remainder_clone, ',', ')')?;
            let mut ethers = Ether::peel(&mut remainder_clone)?;
            if let Some(remainder_clone_stripped) = remainder_clone.trim_start().strip_prefix(',') {
                remainder_clone = remainder_clone_stripped.trim_start().to_string();
                ethers.extend(Ether::peel(&mut remainder_clone)?);
                values.push((identifier, assignment, ethers));
                continue;
            } else if let Some(remainder_clone_stripped) =
                remainder_clone.trim_start().strip_prefix(')')
            {
                remainder_clone = remainder_clone_stripped.to_string();
                ethers.extend(Ether::peel(&mut remainder_clone)?);
                values.push((identifier, assignment, ethers));
                break;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse struct value \n{remainder}"),
                ));
            }
        }
        *remainder = remainder_clone.to_string();
        Ok(Self(values))
    }
}

#[cfg(test)]
#[path = "./test_value.rs"]
mod test_value;

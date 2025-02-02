use std::io::{Error, ErrorKind, Result};

use crate::dazzle::{self, Dazzle};

use super::{Assignment, Identifier, IdentifierSub};

#[derive(Debug, PartialEq)]
pub struct FunctionCall(FunctionCallInner);

impl Dazzle for FunctionCall {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        self.0.dazzle(dazzler);
    }
}

impl FunctionCall {
    pub fn peel(remainder: &mut String) -> Result<Self> {
        Ok(Self(FunctionCallInner::peel(remainder)?))
    }
}

#[derive(Debug, PartialEq)]
struct FunctionCallInner {
    identifier: IdentifierSub,
    arguments: Vec<Argument>,
}

impl Dazzle for FunctionCallInner {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indent_or_space(true);
        self.identifier.dazzle(dazzler);
        dazzler.f.push('(');
        for (i, a) in self.arguments.iter().enumerate() {
            a.dazzle(dazzler);
            if i + 1 < self.arguments.len() {
                dazzler.f.push(',');
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
            }
        }
        dazzler.indent_or_space(false);
        dazzler.f.push(')');
        dazzler.previous_character = dazzle::PreviousCharacter::Other;
    }
}

impl FunctionCallInner {
    fn peel(remainder: &mut String) -> Result<Self> {
        let mut remainder_clone = remainder.clone();
        let identifier = IdentifierSub::peel(&mut remainder_clone)?;
        let mut remainder_clone = match remainder_clone.strip_prefix('(') {
            Some(remainder_clone_stripped) => remainder_clone_stripped.trim_start().to_string(),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find '('\n{remainder}"),
                ))
            }
        };

        let mut arguments = Vec::new();

        loop {
            let argument = Argument::peel(&mut remainder_clone)?;
            arguments.push(argument);
            remainder_clone = match remainder_clone.trim_start().strip_prefix(',') {
                Some(remainder_clone_stripped) => remainder_clone_stripped.trim_start().to_string(),
                None => break,
            };
        }

        remainder_clone = match remainder_clone.trim_start().strip_prefix(')') {
            Some(remainder_clone_stripped) => remainder_clone_stripped.to_string(),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find ')'\n{remainder}"),
                ))
            }
        };

        *remainder = remainder_clone.to_string();
        Ok(Self {
            identifier,
            arguments,
        })
    }
}

#[derive(Debug, PartialEq)]
enum Argument {
    Unnamed(Assignment),
    InputOrInout(Identifier, Assignment),
    Output(Identifier, Assignment),
}

impl Dazzle for Argument {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indent_or_space(false);
        match self {
            Self::Unnamed(assignment) => assignment.dazzle(dazzler),
            Self::InputOrInout(identifier, assignment) => {
                identifier.dazzle(dazzler);
                dazzler.f.push_str(" :=");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                assignment.dazzle(dazzler);
            }
            Self::Output(identifier, assignment) => {
                identifier.dazzle(dazzler);
                dazzler.f.push_str(" =>");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                assignment.dazzle(dazzler);
            }
        }
    }
}

impl Argument {
    fn peel(remainder: &mut String) -> Result<Self> {
        if let Ok((identifier, assignment)) = peel_identifier_and_assignment(remainder, ":=") {
            Ok(Self::InputOrInout(identifier, assignment))
        } else if let Ok((identifier, assignment)) = peel_identifier_and_assignment(remainder, "=>")
        {
            Ok(Self::Output(identifier, assignment))
        } else {
            Ok(Self::Unnamed(peel_assignment_only(remainder)?))
        }
    }
}

fn peel_identifier_and_assignment(
    remainder: &mut String,
    separator: &str,
) -> Result<(Identifier, Assignment)> {
    let mut remainder_clone = remainder.clone();
    let identifier = Identifier::peel(&mut remainder_clone)?;
    remainder_clone = match remainder_clone.trim_start().strip_prefix(separator) {
        Some(remainder_clone_stripped) => remainder_clone_stripped.trim_start().to_string(),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot find \"{separator}\"\n{remainder}"),
            ))
        }
    };
    let assignment = Assignment::peel(&mut remainder_clone, ',', ')')?;
    *remainder = remainder_clone.to_string();
    Ok((identifier, assignment))
}

fn peel_assignment_only(remainder: &mut String) -> Result<Assignment> {
    Assignment::peel(remainder, ',', ')')
}

#[cfg(test)]
#[path = "./test_function_call.rs"]
mod test_function_call;

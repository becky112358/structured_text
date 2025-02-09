use std::io::Result;

use crate::code::Code;
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
    pub fn peel(code: &mut Code) -> Result<Self> {
        Ok(Self(FunctionCallInner::peel(code)?))
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
    fn peel(code: &mut Code) -> Result<Self> {
        let mut code_clone = code.clone();
        let identifier = IdentifierSub::peel(&mut code_clone)?;
        let mut code_clone = code_clone.strip_prefix('(')?;

        let mut arguments = Vec::new();

        loop {
            let argument = Argument::peel(&mut code_clone)?;
            arguments.push(argument);
            code_clone = match code_clone.trim_start().strip_prefix(',') {
                Ok(code_clone_stripped) => code_clone_stripped.trim_start(),
                Err(_) => break,
            };
        }

        code_clone = code_clone.trim_start().strip_prefix(')')?;

        *code = code_clone;
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
    fn peel(code: &mut Code) -> Result<Self> {
        if let Ok((identifier, assignment)) = peel_identifier_and_assignment(code, ":=") {
            Ok(Self::InputOrInout(identifier, assignment))
        } else if let Ok((identifier, assignment)) = peel_identifier_and_assignment(code, "=>") {
            Ok(Self::Output(identifier, assignment))
        } else {
            Ok(Self::Unnamed(peel_assignment_only(code)?))
        }
    }
}

fn peel_identifier_and_assignment(
    code: &mut Code,
    separator: &str,
) -> Result<(Identifier, Assignment)> {
    let mut code_clone = code.clone();
    let identifier = Identifier::peel(&mut code_clone)?;
    code_clone = code_clone
        .trim_start()
        .strip_prefix_str(separator)?
        .trim_start();
    let assignment = Assignment::peel(&mut code_clone, ',', ')')?;
    *code = code_clone;
    Ok((identifier, assignment))
}

fn peel_assignment_only(code: &mut Code) -> Result<Assignment> {
    Assignment::peel(code, ',', ')')
}

#[cfg(test)]
#[path = "./test_function_call.rs"]
mod test_function_call;

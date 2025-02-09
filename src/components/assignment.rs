use std::io::{Error, ErrorKind, Result};

use crate::code::Code;
use crate::dazzle::{self, Dazzle};

use super::{Ether, Expression, FunctionCall, Value};

#[derive(Debug, PartialEq)]
pub enum Assignment {
    Value(Value),
    Expression(Expression),
    FunctionCall(FunctionCall),
}

impl Dazzle for Assignment {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        match self {
            Self::Value(inner) => inner.dazzle(dazzler),
            Self::Expression(inner) => inner.dazzle(dazzler),
            Self::FunctionCall(inner) => inner.dazzle(dazzler),
        }
    }
}

impl Assignment {
    pub fn peel(code: &mut Code, separator: char, terminator: char) -> Result<Self> {
        let mut code_clone = code.clone();
        if let Ok(value) = Value::peel(&mut code_clone) {
            if assignment_complete(&code_clone, separator, terminator) {
                *code = code_clone;
                return Ok(Self::Value(value));
            }
        }

        let mut code_clone = code.clone();
        if let Ok(expression) = Expression::peel(&mut code_clone) {
            if assignment_complete(&code_clone, separator, terminator) {
                *code = code_clone;
                return Ok(Self::Expression(expression));
            }
        }

        let mut code_clone = code.clone();
        if let Ok(function_call) = FunctionCall::peel(&mut code_clone) {
            if assignment_complete(&code_clone, separator, terminator) {
                *code = code_clone;
                return Ok(Self::FunctionCall(function_call));
            }
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse assignment \n{code}"),
        ))
    }
}

fn assignment_complete(code: &Code, separator: char, terminator: char) -> bool {
    code.trim_start().starts_with(separator)
        || code.trim_start().starts_with(terminator)
        || Ether::then_comment(code)
}

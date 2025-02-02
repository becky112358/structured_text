use std::io::{Error, ErrorKind, Result};

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
    pub fn peel(remainder: &mut String, separator: char, terminator: char) -> Result<Self> {
        let mut remainder_clone = remainder.clone();
        if let Ok(value) = Value::peel(&mut remainder_clone) {
            if assignment_complete(&remainder_clone, separator, terminator) {
                *remainder = remainder_clone;
                return Ok(Self::Value(value));
            }
        }

        let mut remainder_clone = remainder.clone();
        if let Ok(expression) = Expression::peel(&mut remainder_clone) {
            if assignment_complete(&remainder_clone, separator, terminator) {
                *remainder = remainder_clone;
                return Ok(Self::Expression(expression));
            }
        }

        let mut remainder_clone = remainder.clone();
        if let Ok(function_call) = FunctionCall::peel(&mut remainder_clone) {
            if assignment_complete(&remainder_clone, separator, terminator) {
                *remainder = remainder_clone;
                return Ok(Self::FunctionCall(function_call));
            }
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse assignment \n{remainder}"),
        ))
    }
}

fn assignment_complete(remainder: &str, separator: char, terminator: char) -> bool {
    remainder.trim().starts_with(separator)
        || remainder.trim().starts_with(terminator)
        || Ether::then_comment(remainder)
}

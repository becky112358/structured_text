use std::io::{Error, ErrorKind, Result};

use crate::code::Code;
use crate::dazzle::{self, Dazzle};

use super::{Ether, Identifier, Member, Value};

#[derive(Debug, PartialEq)]
pub struct Expression(ExpressionInner);

#[derive(Debug, PartialEq)]
enum ExpressionInner {
    BinaryOperator(Box<BinaryOperator>),
    Bracket(Vec<Ether>, Box<ExpressionInner>, Vec<Ether>),
    Dereference(Box<ExpressionInner>),
    Field(Box<ExpressionInner>, Member),
    FunctionCall(Box<FunctionCall>),
    Index(Box<ExpressionInner>, Vec<ExpressionInner>),
    Method(Box<ExpressionInner>, Box<FunctionCall>),
    Negative(Box<ExpressionInner>),
    Not(Vec<Ether>, Box<ExpressionInner>),
    Value(Value),
}

impl Dazzle for Expression {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        self.0.dazzle(dazzler);
    }
}

impl Dazzle for ExpressionInner {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        self.dazzle_inner(dazzler, false);
    }
}

impl ExpressionInner {
    fn dazzle_inner(&self, dazzler: &mut dazzle::Dazzler, indented: bool) {
        match self {
            Self::BinaryOperator(inner) => {
                inner.left.dazzle_inner(dazzler, indented);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                for ether in &inner.ethers0 {
                    ether.dazzle(dazzler);
                }
                inner.operator.dazzle(dazzler);
                for ether in &inner.ethers1 {
                    ether.dazzle(dazzler);
                }
                inner.right.dazzle_inner(dazzler, indented);
                if !indented {
                    dazzler.indentation_count -= 1;
                }
            }
            Self::Bracket(ethers0, inner, ethers1) => {
                '('.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                for ether in ethers0 {
                    ether.dazzle(dazzler);
                }
                inner.dazzle_inner(dazzler, true);
                for ether in ethers1 {
                    ether.dazzle(dazzler);
                }
                if !indented {
                    dazzler.indentation_count -= 1;
                }
                ')'.dazzle(dazzler);
            }
            Self::Dereference(inner) => {
                inner.dazzle_inner(dazzler, indented);
                '^'.dazzle(dazzler);
            }
            Self::Field(inner, field) => {
                inner.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                '.'.dazzle(dazzler);
                field.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count -= 1;
                }
            }
            Self::FunctionCall(inner) => inner.dazzle(dazzler),
            Self::Index(inner, index) => {
                inner.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                '['.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                for (i, is) in index.iter().enumerate() {
                    is.dazzle_inner(dazzler, true);
                    if i + 1 < index.len() {
                        ','.dazzle(dazzler);
                    }
                }
                if !indented {
                    dazzler.indentation_count -= 1;
                }
                ']'.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count -= 1;
                }
            }
            Self::Method(inner, method) => {
                inner.dazzle_inner(dazzler, indented);
                if !indented {
                    dazzler.indentation_count += 1;
                }
                '.'.dazzle(dazzler);
                method.dazzle(dazzler);
                if !indented {
                    dazzler.indentation_count -= 1;
                }
            }
            Self::Negative(inner) => {
                '-'.dazzle(dazzler);
                inner.dazzle_inner(dazzler, indented);
            }
            Self::Not(ethers, inner) => {
                "NOT".dazzle(dazzler);
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                if !indented {
                    dazzler.indentation_count += 1;
                }
                for ether in ethers {
                    ether.dazzle(dazzler);
                }
                inner.dazzle_inner(dazzler, indented);
                if !indented {
                    dazzler.indentation_count -= 1;
                }
            }
            Self::Value(inner) => inner.dazzle(dazzler),
        }
    }
}

impl Expression {
    pub fn peel(code: &mut Code) -> Result<Self> {
        Ok(Self(ExpressionInner::peel(code)?))
    }
}

impl ExpressionInner {
    fn peel(code: &mut Code) -> Result<Self> {
        let mut expression = Self::peel_start(code)?;

        let mut extended = true;
        while extended {
            extended = false;

            if let Ok(binary_operator) = BinaryOperator::peel_rhs(code) {
                expression = BinaryOperator::construct(expression, binary_operator);
                extended = true;
            }

            if let Ok(code_clone) = code.strip_prefix('^') {
                *code = code_clone;
                expression = Self::Dereference(Box::new(expression));
                extended = true;
            }

            if let Ok(mut code_clone) = code.trim_start().strip_prefix('.') {
                code_clone = code_clone.trim_start();
                if let Ok(identifier) = Value::peel(&mut code_clone) {
                    if let Ok((ethers, arguments)) = FunctionCall::peel_rhs(&mut code_clone) {
                        *code = code_clone;
                        let method = FunctionCall {
                            identifier: ExpressionInner::Value(identifier),
                            ethers,
                            arguments,
                        };
                        expression = expression.extend(method, |left, right| {
                            Self::Method(Box::new(left), Box::new(right))
                        });
                        extended = true;
                    }
                }
            }

            if let Ok(mut code_clone) = code.trim_start().strip_prefix('.') {
                code_clone = code_clone.trim_start();
                if let Ok(field) = Member::peel(&mut code_clone) {
                    *code = code_clone;
                    expression =
                        expression.extend(field, |left, right| Self::Field(Box::new(left), right));
                    extended = true;
                }
            }

            if let Ok(index) = Self::peel_index(code) {
                expression =
                    expression.extend(index, |left, right| Self::Index(Box::new(left), right));
                extended = true;
            }

            if let Ok((ethers, arguments)) = FunctionCall::peel_rhs(code) {
                expression = Self::FunctionCall(Box::new(FunctionCall {
                    identifier: expression,
                    ethers,
                    arguments,
                }));
                extended = true;
            }
        }

        Ok(expression)
    }

    fn peel_start(code: &mut Code) -> Result<Self> {
        if let Ok(mut code_clone) = code.strip_prefix_uppercase("NOT") {
            if let Ok(ethers) = Ether::peel(&mut code_clone) {
                if let Ok(expression) = Self::peel(&mut code_clone) {
                    *code = code_clone;
                    return Ok(Self::Not(ethers, Box::new(expression)));
                }
            }
        }

        if let Ok(mut code_clone) = code.strip_prefix('-') {
            if let Ok(expression) = Self::peel(&mut code_clone) {
                *code = code_clone;
                return Ok(Self::Negative(Box::new(expression)));
            }
        }

        if let Ok(mut code_clone) = code.strip_prefix('(') {
            if let Ok(ethers0) = Ether::peel(&mut code_clone) {
                if let Ok(expression) = Self::peel(&mut code_clone) {
                    if let Ok(ethers1) = Ether::peel(&mut code_clone) {
                        if let Ok(code_stripped) = code_clone.strip_prefix(')') {
                            *code = code_stripped;
                            return Ok(Self::Bracket(ethers0, Box::new(expression), ethers1));
                        }
                    }
                }
            }
        }

        let mut code_clone = code.clone();
        if let Ok(expression) = Value::peel(&mut code_clone) {
            *code = code_clone;
            return Ok(Self::Value(expression));
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("No expression\n{code}"),
        ))
    }

    fn peel_index(code: &mut Code) -> Result<Vec<Self>> {
        let mut code_clone = code.strip_prefix('[')?;
        let mut indices = vec![Self::peel(&mut code_clone)?];
        while let Ok(code_stripped) = code_clone.trim_start().strip_prefix(',') {
            code_clone = code_stripped;
            indices.push(Self::peel(&mut code_clone)?);
        }
        code_clone = code_clone.strip_prefix(']')?;
        *code = code_clone;
        Ok(indices)
    }

    fn extend<T>(self, right: T, outer: impl Fn(Self, T) -> Self) -> Self {
        match self {
            Self::BinaryOperator(inner) => Self::BinaryOperator(Box::new(BinaryOperator {
                left: inner.left,
                ethers0: inner.ethers0,
                operator: inner.operator,
                ethers1: inner.ethers1,
                right: inner.right.extend(right, outer),
            })),
            Self::Negative(inner) => Self::Negative(Box::new(inner.extend(right, outer))),
            Self::Not(ethers, inner) => Self::Not(ethers, Box::new(inner.extend(right, outer))),
            Self::Bracket(_, _, _)
            | Self::Dereference(_)
            | Self::Field(_, _)
            | Self::FunctionCall(_)
            | Self::Index(_, _)
            | Self::Method(_, _)
            | Self::Value(_) => outer(self, right),
        }
    }
}

#[derive(Debug, PartialEq)]
struct BinaryOperator {
    left: ExpressionInner,
    ethers0: Vec<Ether>,
    operator: Operator,
    ethers1: Vec<Ether>,
    right: ExpressionInner,
}

impl BinaryOperator {
    fn peel_rhs(code: &mut Code) -> Result<(Vec<Ether>, Operator, Vec<Ether>, ExpressionInner)> {
        let mut code_clone = code.clone();

        let ethers0 = Ether::peel(&mut code_clone)?;
        let operator = Operator::peel(&mut code_clone)?;
        let ethers1 = Ether::peel(&mut code_clone)?;
        let right = ExpressionInner::peel_start(&mut code_clone)?;

        *code = code_clone;
        Ok((ethers0, operator, ethers1, right))
    }

    fn construct(
        lhs: ExpressionInner,
        (ethers0, operator, ethers1, rhs): (Vec<Ether>, Operator, Vec<Ether>, ExpressionInner),
    ) -> ExpressionInner {
        if operator.seperates() {
            if matches!(lhs, ExpressionInner::BinaryOperator(_)) {
                let output = ExpressionInner::BinaryOperator(Box::new(Self {
                    left: lhs,
                    ethers0,
                    operator,
                    ethers1,
                    right: rhs,
                }));
                return output;
            }
        } else if let ExpressionInner::BinaryOperator(mut first_binary_operator) = lhs {
            first_binary_operator.right = ExpressionInner::BinaryOperator(Box::new(Self {
                left: first_binary_operator.right,
                ethers0,
                operator,
                ethers1,
                right: rhs,
            }));
            let output = ExpressionInner::BinaryOperator(first_binary_operator);
            return output;
        }

        if operator.seperates() {
            if matches!(lhs, ExpressionInner::Not(_, _)) {
                return ExpressionInner::BinaryOperator(Box::new(Self {
                    left: lhs,
                    ethers0,
                    operator,
                    ethers1,
                    right: rhs,
                }));
            }
        } else if let ExpressionInner::Not(ethers, inner) = lhs {
            return ExpressionInner::Not(
                ethers,
                Box::new(ExpressionInner::BinaryOperator(Box::new(Self {
                    left: *inner,
                    ethers0,
                    operator,
                    ethers1,
                    right: rhs,
                }))),
            );
        }

        match lhs {
            ExpressionInner::BinaryOperator(_) => unreachable!(),
            ExpressionInner::Not(_, _) => unreachable!(),
            ExpressionInner::Bracket(_, _, _)
            | ExpressionInner::Dereference(_)
            | ExpressionInner::Field(_, _)
            | ExpressionInner::FunctionCall(_)
            | ExpressionInner::Index(_, _)
            | ExpressionInner::Method(_, _)
            | ExpressionInner::Negative(_)
            | ExpressionInner::Value(_) => ExpressionInner::BinaryOperator(Box::new(Self {
                left: lhs,
                ethers0,
                operator,
                ethers1,
                right: rhs,
            })),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    And,
    Or,
    Xor,
    LessThanOrEqual,
    LessThan,
    EqualTo,
    NotEqualTo,
    GreaterThan,
    GreaterThanOrEqual,
}

impl Dazzle for Operator {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indent_or_space(true);
        match self {
            Self::Add => dazzler.f.push('+'),
            Self::Subtract => dazzler.f.push('-'),
            Self::Multiply => dazzler.f.push('*'),
            Self::Divide => dazzler.f.push('/'),
            Self::Mod => dazzler.f.push_str("MOD"),
            Self::And => dazzler.f.push_str("AND"),
            Self::Or => dazzler.f.push_str("OR"),
            Self::Xor => dazzler.f.push_str("XOR"),
            Self::LessThanOrEqual => dazzler.f.push_str("<="),
            Self::LessThan => dazzler.f.push('<'),
            Self::EqualTo => dazzler.f.push('='),
            Self::NotEqualTo => dazzler.f.push_str("<>"),
            Self::GreaterThan => dazzler.f.push('>'),
            Self::GreaterThanOrEqual => dazzler.f.push_str(">="),
        }
        dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
    }
}

impl Operator {
    fn peel(code: &mut Code) -> Result<Self> {
        if let Ok(code_clone) = code.strip_prefix('+') {
            *code = code_clone;
            Ok(Self::Add)
        } else if let Ok(code_clone) = code.strip_prefix('-') {
            *code = code_clone;
            Ok(Self::Subtract)
        } else if let Ok(code_clone) = code.strip_prefix('*') {
            *code = code_clone;
            Ok(Self::Multiply)
        } else if let Ok(code_clone) = code.strip_prefix('/') {
            *code = code_clone;
            Ok(Self::Divide)
        } else if let Ok(code_clone) = code.strip_prefix_uppercase("MOD") {
            *code = code_clone;
            Ok(Self::Mod)
        } else if let Ok(code_clone) = code.strip_prefix_uppercase("AND") {
            *code = code_clone;
            Ok(Self::And)
        } else if let Ok(code_clone) = code.strip_prefix_uppercase("OR") {
            *code = code_clone;
            Ok(Self::Or)
        } else if let Ok(code_clone) = code.strip_prefix_uppercase("XOR") {
            *code = code_clone;
            Ok(Self::Xor)
        } else if let Ok(code_clone) = code.strip_prefix_str("<=") {
            *code = code_clone;
            Ok(Self::LessThanOrEqual)
        } else if let Ok(code_clone) = code.strip_prefix_str("<>") {
            *code = code_clone;
            Ok(Self::NotEqualTo)
        } else if let Ok(code_clone) = code.strip_prefix('<') {
            *code = code_clone;
            Ok(Self::LessThan)
        } else if let Ok(code_clone) = code.strip_prefix('=') {
            *code = code_clone;
            Ok(Self::EqualTo)
        } else if let Ok(code_clone) = code.strip_prefix_str(">=") {
            *code = code_clone;
            Ok(Self::GreaterThanOrEqual)
        } else if let Ok(code_clone) = code.strip_prefix('>') {
            *code = code_clone;
            Ok(Self::GreaterThan)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("No operator\n{code}"),
            ))
        }
    }

    fn seperates(&self) -> bool {
        match self {
            Self::And | Self::Or | Self::Xor => true,
            Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Mod
            | Self::LessThanOrEqual
            | Self::LessThan
            | Self::EqualTo
            | Self::NotEqualTo
            | Self::GreaterThan
            | Self::GreaterThanOrEqual => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    identifier: ExpressionInner,
    ethers: Vec<Ether>,
    arguments: Arguments,
}

type Arguments = Vec<(Argument, Vec<Ether>)>;

impl Dazzle for FunctionCall {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        if dazzler.should_split(self, dazzle_singleline) {
            self.dazzle_multiline(dazzler);
        } else {
            dazzle_singleline(self, dazzler);
        }
    }
}

fn dazzle_singleline(function_call: &FunctionCall, dazzler: &mut dazzle::Dazzler) {
    dazzler.indent_or_space(false);
    function_call.identifier.dazzle(dazzler);
    '('.dazzle(dazzler);
    dazzler.previous_character = dazzle::PreviousCharacter::Other;
    if !&function_call
        .ethers
        .iter()
        .all(|ether| *ether == Ether::LineFeed)
    {
        for ether in &function_call.ethers {
            ether.dazzle(dazzler);
        }
    }
    for (i, (a, e)) in function_call.arguments.iter().enumerate() {
        a.dazzle(dazzler);
        if i + 1 < function_call.arguments.len() {
            dazzler.f.push(',');
            dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
        }
        if !&e.iter().all(|ether| *ether == Ether::LineFeed) {
            for ether in e {
                ether.dazzle(dazzler);
            }
        }
    }
    ')'.dazzle(dazzler);
}

impl FunctionCall {
    fn dazzle_multiline(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indent_or_space(false);
        self.identifier.dazzle(dazzler);
        '('.dazzle(dazzler);
        dazzler.indentation_count += 1;
        for ether in &self.ethers {
            ether.dazzle(dazzler);
        }
        dazzler.if_not_linefeed_then_linefeed();

        let max_identifier_length = self
            .arguments
            .iter()
            .map(|(argument, _)| match argument {
                Argument::Unnamed(_) => 0,
                Argument::InputOrInout(i, _) => i.to_string().len(),
                Argument::Output(i, _) => i.to_string().len(),
            })
            .max()
            .unwrap_or(0);

        for (argument, ethers) in self.arguments.iter() {
            match argument {
                Argument::Unnamed(identifierx) => {
                    if let Some(identifier) = identifierx {
                        identifier.dazzle(dazzler);
                        ','.dazzle(dazzler);
                    }
                }
                Argument::InputOrInout(left, rightx) => {
                    left.dazzle(dazzler);
                    for _ in 0..(max_identifier_length - left.to_string().len()) {
                        dazzler.f.push(' ');
                    }
                    dazzler.f.push_str(" :=");
                    dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                    if let Some(right) = rightx {
                        right.dazzle(dazzler);
                    }
                    ','.dazzle(dazzler);
                }
                Argument::Output(left, rightx) => {
                    left.dazzle(dazzler);
                    for _ in 0..(max_identifier_length - left.to_string().len()) {
                        dazzler.f.push(' ');
                    }
                    dazzler.f.push_str(" =>");
                    dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                    if let Some(right) = rightx {
                        right.dazzle(dazzler);
                    }
                    ','.dazzle(dazzler);
                }
            }
            for ether in ethers {
                ether.dazzle(dazzler);
            }
            dazzler.if_not_linefeed_then_linefeed();
        }

        dazzler.indentation_count -= 1;
        ')'.dazzle(dazzler);
    }

    fn peel_rhs(code: &mut Code) -> Result<(Vec<Ether>, Arguments)> {
        let mut code_clone = code.clone();
        let mut ethers = Ether::peel(&mut code_clone)?;
        let mut code_clone = code_clone.strip_prefix('(')?;
        ethers.extend(Ether::peel(&mut code_clone)?);

        let mut arguments = Vec::new();

        let mut found_comma = true;
        while found_comma {
            let argument = Argument::peel(&mut code_clone)?;
            let mut ethers_inner = Ether::peel(&mut code_clone)?;
            match code_clone.strip_prefix(',') {
                Ok(code_clone_stripped) => {
                    found_comma = true;
                    code_clone = code_clone_stripped;
                }
                Err(_) => found_comma = false,
            }
            if !argument.is_empty() || !ethers_inner.is_empty() {
                ethers_inner.extend(Ether::peel(&mut code_clone)?);
                arguments.push((argument, ethers_inner));
            }
        }

        code_clone = code_clone.strip_prefix(')')?;

        *code = code_clone;
        Ok((ethers, arguments))
    }
}

#[derive(Debug, PartialEq)]
enum Argument {
    Unnamed(Option<Expression>),
    InputOrInout(Identifier, Option<Expression>),
    Output(Identifier, Option<Expression>),
}

impl Dazzle for Argument {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        dazzler.indent_or_space(false);
        match self {
            Self::Unnamed(expressionx) => {
                if let Some(expression) = expressionx {
                    expression.dazzle(dazzler)
                }
            }
            Self::InputOrInout(identifier, expressionx) => {
                identifier.dazzle(dazzler);
                dazzler.f.push_str(" :=");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                if let Some(expression) = expressionx {
                    expression.dazzle(dazzler);
                }
            }
            Self::Output(identifier, expressionx) => {
                identifier.dazzle(dazzler);
                dazzler.f.push_str(" =>");
                dazzler.previous_character = dazzle::PreviousCharacter::PendingSpace;
                if let Some(expression) = expressionx {
                    expression.dazzle(dazzler);
                }
            }
        }
    }
}

impl Argument {
    fn peel(code: &mut Code) -> Result<Self> {
        if let Ok((identifier, expression)) = peel_identifier_and_expression(code, ":=") {
            Ok(Self::InputOrInout(identifier, expression))
        } else if let Ok((identifier, expression)) = peel_identifier_and_expression(code, "=>") {
            Ok(Self::Output(identifier, expression))
        } else {
            Ok(Self::Unnamed(peel_expression_only(code)))
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::Unnamed(None)
    }
}

fn peel_identifier_and_expression(
    code: &mut Code,
    separator: &str,
) -> Result<(Identifier, Option<Expression>)> {
    let mut code_clone = code.clone();
    let identifier = Identifier::peel(&mut code_clone)?;
    code_clone = code_clone
        .trim_start()
        .strip_prefix_str(separator)?
        .trim_start();
    let expression = match Expression::peel(&mut code_clone) {
        Ok(exp) => Some(exp),
        Err(_) => None,
    };
    *code = code_clone;
    Ok((identifier, expression))
}

fn peel_expression_only(code: &mut Code) -> Option<Expression> {
    match Expression::peel(code) {
        Ok(expression) => Some(expression),
        Err(_) => None,
    }
}

#[cfg(test)]
#[path = "./test_expression.rs"]
mod test_expression;

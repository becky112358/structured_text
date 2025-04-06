use std::io::{Error, ErrorKind, Result};

use crate::code::Code;
use crate::components::{
    Address, BeginMiddleEnd, Component as C, DataType, Ether, Expression, Identifier,
    IdentifierSub, Value,
};
use crate::implementation;

pub fn string_and_format_get_items(code: &mut Code, layout: &[Layout]) -> Result<Vec<C>> {
    let mut output = Vec::new();
    let mut code_clone = code.clone();

    for l in layout {
        let items = string_and_one_format_get_items(&mut code_clone, l)?;
        output.extend(items);
    }

    *code = code_clone;
    Ok(output)
}

fn string_and_one_format_get_items(code: &mut Code, layout: &Layout) -> Result<Vec<C>> {
    let mut output = Vec::new();

    let mut code_clone = code.clone();

    for ether in Ether::peel(&mut code_clone)? {
        output.push(C::Ether(ether));
    }

    match layout {
        Layout::Space => output.push(C::Space),
        Layout::LineFeed => {
            if !matches!(output.last(), Some(C::Ether(Ether::LineFeed))) {
                output.push(C::Ether(Ether::LineFeed));
            }
        }
        Layout::Uppercase(text) => {
            output.push(C::Uppercase(peel_uppercase(&mut code_clone, text)?))
        }
        Layout::Text(text) => output.push(C::Text(peel(&mut code_clone, text)?)),
        Layout::Identifier => output.push(C::Identifier(Identifier::peel(&mut code_clone)?)),
        Layout::IdentifierSub => {
            output.push(C::IdentifierSub(IdentifierSub::peel(&mut code_clone)?))
        }
        Layout::Address => output.push(C::Address(Address::peel(&mut code_clone)?)),
        Layout::DataType => output.push(C::DataType(DataType::peel(&mut code_clone)?)),
        Layout::Value => output.push(C::Value(Value::peel(&mut code_clone)?)),
        Layout::Expression => output.push(C::Expression(Expression::peel(&mut code_clone)?)),
        Layout::OneOf(options) => {
            let mut found = false;
            for option in *options {
                let mut code_clone_clone = code_clone.clone();
                if let Ok(item) = string_and_format_get_items(&mut code_clone_clone, option) {
                    output.extend(item);
                    code_clone = code_clone_clone;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find any of \n{options:?} \n{code_clone}"),
                ));
            }
        }
        Layout::Option(inner) => {
            if let Ok(items) = string_and_format_get_items(&mut code_clone, inner) {
                output.extend(items);
            } else {
                return Ok(Vec::new());
            }
        }
        Layout::BeginMiddleEnd(beginf, middlef, endf) => {
            let (begin_middle_end, ethers) = BeginMiddleEnd::peel(
                &mut code_clone,
                |c| string_and_format_get_items(c, beginf),
                |c| string_and_format_get_items(c, middlef),
                |c| string_and_format_get_items(c, endf),
            )?;
            output.push(C::BeginMiddleEnd(begin_middle_end));
            output.extend(ethers.into_iter().map(C::Ether).collect::<Vec<C>>());
        }
        Layout::Repeat(inner) => {
            let mut found = false;
            let mut code_clone_clone = code_clone.clone();
            while let Ok(items) = string_and_format_get_items(&mut code_clone_clone, inner) {
                output.push(C::Repeat(items));
                code_clone = code_clone_clone.clone();
                found = true;
            }
            if !found {
                return Ok(Vec::new());
            }
        }
        Layout::Implementation => output.extend(implementation::peel(&mut code_clone)?),
    }

    *code = code_clone;

    Ok(output)
}

#[derive(Debug)]
pub enum Layout {
    Space,
    LineFeed,
    Uppercase(&'static str),
    Text(&'static str),
    Identifier,
    IdentifierSub,
    Address,
    DataType,
    Value,
    Expression,
    OneOf(&'static [&'static [Layout]]),
    Option(&'static [Layout]),
    BeginMiddleEnd(&'static [Layout], &'static [Layout], &'static [Layout]),
    Repeat(&'static [Layout]),
    Implementation,
}

fn peel<'a>(code: &mut Code, text: &'a str) -> Result<&'a str> {
    *code = code.strip_prefix_str(text)?;
    Ok(text)
}

fn peel_uppercase<'a>(code: &mut Code, text: &'a str) -> Result<&'a str> {
    *code = code.strip_prefix_uppercase(text)?;
    Ok(text)
}

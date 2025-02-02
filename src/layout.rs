use std::io::{Error, ErrorKind, Result};

use crate::components::{
    Address, Assignment, Component as C, DataType, Ether, Identifier, IdentifierSub, Value,
};

pub fn string_and_format_get_items(remainder: &mut String, layout: &[Layout]) -> Result<Vec<C>> {
    let mut output = Vec::new();

    for l in layout {
        let items = string_and_one_format_get_items(remainder, l)?;
        output.extend(items);
    }

    Ok(output)
}

fn string_and_one_format_get_items(remainder: &mut String, layout: &Layout) -> Result<Vec<C>> {
    let mut output = Vec::new();

    let mut remainder_clone = remainder.clone();

    for ether in Ether::peel(&mut remainder_clone)? {
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
            output.push(C::Uppercase(peel_uppercase(&mut remainder_clone, text)?))
        }
        Layout::Text(text) => output.push(C::Text(peel(&mut remainder_clone, text)?)),
        Layout::Identifier => output.push(C::Identifier(Identifier::peel(&mut remainder_clone)?)),
        Layout::IdentifierSub => {
            output.push(C::IdentifierSub(IdentifierSub::peel(&mut remainder_clone)?))
        }
        Layout::Address => output.push(C::Address(Address::peel(&mut remainder_clone)?)),
        Layout::DataType => output.push(C::DataType(DataType::peel(&mut remainder_clone)?)),
        Layout::Value => output.push(C::Value(Value::peel(&mut remainder_clone)?)),
        Layout::Assignment => output.push(C::Assignment(Assignment::peel(
            &mut remainder_clone,
            ';',
            ';',
        )?)),
        Layout::OneOf(options) => {
            let mut found = false;
            for option in *options {
                let mut remainder_clone_clone = remainder_clone.clone();
                if let Ok(item) = string_and_format_get_items(&mut remainder_clone_clone, option) {
                    output.extend(item);
                    remainder_clone = remainder_clone_clone;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find any of \n{options:?} \n{remainder_clone}"),
                ));
            }
        }
        Layout::Option(inner) => {
            if let Ok(items) = string_and_format_get_items(&mut remainder_clone, inner) {
                output.extend(items);
            } else {
                return Ok(Vec::new());
            }
        }
        Layout::BeginMiddleEnd(beginf, middlef, endf) => {
            let mut begind = string_and_format_get_items(&mut remainder_clone, beginf)?;

            let mut middled_start_ethers = Vec::new();
            let mut new_line = false;
            for ether in Ether::peel(&mut remainder_clone)? {
                if !new_line {
                    begind.push(C::Ether(ether.clone()));
                } else {
                    middled_start_ethers.push(C::Ether(ether.clone()));
                }
                if matches!(ether, Ether::LineFeed) {
                    new_line = true;
                }
            }
            if !matches!(begind.last(), Some(C::Ether(Ether::LineFeed))) {
                begind.push(C::Ether(Ether::LineFeed));
            }

            let mut middled = vec![middled_start_ethers];
            let mut remainder_clone_clone = remainder_clone.clone();
            while let Ok(mut items) =
                string_and_format_get_items(&mut remainder_clone_clone, middlef)
            {
                for ether in Ether::peel(&mut remainder_clone_clone)? {
                    items.push(C::Ether(ether));
                }
                if !matches!(items.last(), Some(C::Ether(Ether::LineFeed))) {
                    items.push(C::Ether(Ether::LineFeed));
                }
                middled.push(items);
                remainder_clone = remainder_clone_clone.clone();
            }

            let mut endd = string_and_format_get_items(&mut remainder_clone, endf)?;
            let mut output_after_ethers = Vec::new();
            let mut new_line = false;
            for ether in Ether::peel(&mut remainder_clone)? {
                if !new_line {
                    endd.push(C::Ether(ether.clone()));
                } else {
                    output_after_ethers.push(C::Ether(ether.clone()));
                }
                if matches!(ether, Ether::LineFeed) {
                    new_line = true;
                }
            }
            if !matches!(endd.last(), Some(C::Ether(Ether::LineFeed))) {
                endd.push(C::Ether(Ether::LineFeed));
            }
            output.push(C::BeginMiddleEnd(begind, middled, endd));
            output.extend(output_after_ethers);
        }
        Layout::Repeat(inner) => {
            let mut found = false;
            let mut remainder_clone_clone = remainder_clone.clone();
            while let Ok(items) = string_and_format_get_items(&mut remainder_clone_clone, inner) {
                output.push(C::Repeat(items));
                remainder_clone = remainder_clone_clone.clone();
                found = true;
            }
            if !found {
                return Ok(Vec::new());
            }
        }
    }

    *remainder = remainder_clone;

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
    Assignment,
    OneOf(&'static [&'static [Layout]]),
    Option(&'static [Layout]),
    BeginMiddleEnd(&'static [Layout], &'static [Layout], &'static [Layout]),
    Repeat(&'static [Layout]),
}

fn peel<'a>(remainder: &mut String, text: &'a str) -> Result<&'a str> {
    if let Some(remainder_stripped) = remainder.strip_prefix(text) {
        *remainder = remainder_stripped.to_string();
        Ok(text)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot find {text} in \n{remainder}"),
        ))
    }
}

fn peel_uppercase<'a>(remainder: &mut String, text: &'a str) -> Result<&'a str> {
    if remainder.to_uppercase().starts_with(text) {
        *remainder = remainder[text.len()..].to_string();
        Ok(text)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot find {text} (case-insensitive) in \n{remainder}"),
        ))
    }
}

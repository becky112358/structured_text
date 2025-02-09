use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::code::Code;
use crate::components::{
    Address, Assignment, Component as C, DataType, Ether, Identifier, IdentifierSub, Value,
};

struct Layout(&'static [L]);
#[derive(Debug)]
pub struct Declaration(pub Vec<C>);

impl FromStr for Declaration {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let mut code = Code::from(input);

        if let Ok(declaration) = string_and_format_get_declaration(&mut code, &ENUM) {
            Ok(declaration)
        } else if let Ok(declaration) = string_and_format_get_declaration(&mut code, &STRUCT) {
            Ok(declaration)
        } else if let Ok(declaration) = string_and_format_get_declaration(&mut code, &UNION) {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_format_get_declaration(&mut code, &GLOBAL_VARIABLE_LIST)
        {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_format_get_declaration(&mut code, &PROGRAM_ORGANISATION_UNIT)
        {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_format_get_declaration(&mut code, &PROPERTY_GET_OR_SET)
        {
            Ok(declaration)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot parse \n{input}"),
            ))
        }
    }
}

fn string_and_format_get_declaration(code: &mut Code, layout: &Layout) -> Result<Declaration> {
    let mut items = string_and_format_get_items(code, layout.0)?;
    for ether in Ether::peel(code)? {
        items.push(C::Ether(ether));
    }
    let declaration = Declaration(items);

    if code.end_of_file() {
        Ok(declaration)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse \n{code}"),
        ))
    }
}

fn string_and_format_get_items(code: &mut Code, layout: &[L]) -> Result<Vec<C>> {
    let mut output = Vec::new();

    for l in layout {
        let items = string_and_one_format_get_items(code, l)?;
        output.extend(items);
    }

    Ok(output)
}

fn string_and_one_format_get_items(code: &mut Code, layout: &L) -> Result<Vec<C>> {
    let mut output = Vec::new();

    let mut code_clone = code.clone();

    for ether in Ether::peel(&mut code_clone)? {
        output.push(C::Ether(ether));
    }

    match layout {
        L::Space => output.push(C::Space),
        L::LineFeed => {
            if !matches!(output.last(), Some(C::Ether(Ether::LineFeed))) {
                output.push(C::Ether(Ether::LineFeed));
            }
        }
        L::Uppercase(text) => output.push(C::Uppercase(peel_uppercase(&mut code_clone, text)?)),
        L::Text(text) => output.push(C::Text(peel(&mut code_clone, text)?)),
        L::Identifier => output.push(C::Identifier(Identifier::peel(&mut code_clone)?)),
        L::IdentifierSub => output.push(C::IdentifierSub(IdentifierSub::peel(&mut code_clone)?)),
        L::Address => output.push(C::Address(Address::peel(&mut code_clone)?)),
        L::DataType => output.push(C::DataType(DataType::peel(&mut code_clone)?)),
        L::Value => output.push(C::Value(Value::peel(&mut code_clone)?)),
        L::Assignment => output.push(C::Assignment(Assignment::peel(&mut code_clone, ';', ';')?)),
        L::OneOf(options) => {
            let mut found = false;
            for option in *options {
                if let Ok(item) = string_and_format_get_items(&mut code_clone, option) {
                    output.extend(item);
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
        L::Option(inner) => {
            if let Ok(items) = string_and_format_get_items(&mut code_clone, inner) {
                output.extend(items);
            } else {
                return Ok(Vec::new());
            }
        }
        L::BeginMiddleEnd(beginf, middlef, endf) => {
            let mut begind = string_and_format_get_items(&mut code_clone, beginf)?;

            let mut middled_start_ethers = Vec::new();
            let mut new_line = false;
            for ether in Ether::peel(&mut code_clone)? {
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
            let mut code_clone_clone = code_clone.clone();
            while let Ok(mut items) = string_and_format_get_items(&mut code_clone_clone, middlef) {
                for ether in Ether::peel(&mut code_clone_clone)? {
                    items.push(C::Ether(ether));
                }
                if !matches!(items.last(), Some(C::Ether(Ether::LineFeed))) {
                    items.push(C::Ether(Ether::LineFeed));
                }
                middled.push(items);
                code_clone = code_clone_clone.clone();
            }

            let mut endd = string_and_format_get_items(&mut code_clone, endf)?;
            let mut output_after_ethers = Vec::new();
            let mut new_line = false;
            for ether in Ether::peel(&mut code_clone)? {
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
        L::Repeat(inner) => {
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
    }

    *code = code_clone;

    Ok(output)
}

const VARIABLE_DECLARATION: &[L] = &[L::OneOf(&[
    &[
        L::Identifier,
        L::Option(&[L::Repeat(&[L::Text(","), L::Space, L::Identifier])]),
        L::Option(&[L::Space, L::Address]),
        L::Space,
        L::Text(":"),
        L::Space,
        L::DataType,
        L::Option(&[L::Space, L::Text(":="), L::Space, L::Assignment]),
        L::Text(";"),
    ],
    &[L::Text(";")],
])];

#[rustfmt::skip]
const ENUM: Layout = Layout(&[
    L::BeginMiddleEnd(
        &[
            L::Uppercase("TYPE"), L::Space, L::Identifier, L::Space, L::Text(":"), L::LineFeed,
            L::Text("("),
        ],
        &[L::Identifier, L::Option(&[L::Space, L::Text(":="), L::Space, L::Value]), L::Option(&[L::Text(",")])],
        &[
            L::Text(")"), L::Text(";"), L::LineFeed,
            L::Uppercase("END_TYPE"),
        ],
    ),
]);

#[rustfmt::skip]
const STRUCT: Layout = Layout(&[
    L::BeginMiddleEnd(
        &[
            L::Uppercase("TYPE"), L::Space, L::Identifier, L::Space, L::Option(&[L::Uppercase("EXTENDS"), L::Space, L::Identifier, L::Space]), L::Text(":"), L::LineFeed,
            L::Uppercase("STRUCT"),
        ],
        VARIABLE_DECLARATION,
        &[
            L::Uppercase("END_STRUCT"), L::LineFeed,
            L::Uppercase("END_TYPE"),
        ],
    ),
]);

#[rustfmt::skip]
const UNION: Layout = Layout(&[
    L::BeginMiddleEnd(
        &[
            L::Uppercase("TYPE"), L::Space, L::Identifier, L::Space, L::Text(":"), L::LineFeed,
            L::Uppercase("UNION"),
        ],
        &[L::Identifier, L::Space, L::Text(":"), L::Space, L::DataType, L::Text(";")],
        &[
            L::Uppercase("END_UNION"), L::LineFeed,
            L::Uppercase("END_TYPE"),
        ],
    ),
]);

const GLOBAL_VARIABLE_LIST: Layout = Layout(&[L::BeginMiddleEnd(
    &[
        L::Uppercase("VAR_GLOBAL"),
        L::Option(&[L::OneOf(&[
            &[L::Space, L::Uppercase("CONSTANT")],
            &[L::Space, L::Uppercase("PERSISTENT")],
        ])]),
    ],
    VARIABLE_DECLARATION,
    &[L::Uppercase("END_VAR")],
)]);

#[rustfmt::skip]
const PROGRAM_ORGANISATION_UNIT: Layout = Layout(&[
    L::OneOf(&[
        &[L::Uppercase("PROGRAM"), L::Space, L::Identifier],
        &[L::Uppercase("FUNCTION_BLOCK"), L::Space, L::Option(&[L::Uppercase("PUBLIC"), L::Space]), L::Identifier, L::Option(&[L::Space, L::Uppercase("EXTENDS"), L::Space, L::IdentifierSub])],
        &[
            L::OneOf(&[&[L::Uppercase("METHOD")], &[L::Uppercase("PROPERTY")]]),
            L::Space,
            L::Option(&[
                L::OneOf(&[&[L::Uppercase("PRIVATE")], &[L::Uppercase("PROTECTED")], &[L::Uppercase("PUBLIC")], &[L::Uppercase("INTERNAL")]]),
                L::Space,
            ]),
            L::Identifier,
            L::Option(&[L::Space, L::Text(":"), L::Space, L::DataType, L::Option(&[L::Text(";")])]),
        ],
        &[
            L::Uppercase("FUNCTION"),
            L::Space,
            L::Identifier,
            L::Option(&[L::Space, L::Text(":"), L::Space, L::DataType]),
        ],
    ]),
    L::LineFeed,
    L::Repeat(&[L::BeginMiddleEnd(
        &[
            L::OneOf(&[
                &[L::Uppercase("VAR_INPUT")],
                &[L::Uppercase("VAR_IN_OUT")],
                &[L::Uppercase("VAR_OUTPUT")],
                &[L::Uppercase("VAR CONSTANT")],
                &[L::Uppercase("VAR PERSISTENT")],
                &[L::Uppercase("VAR RETAIN")],
                &[L::Uppercase("VAR_INST")],
                &[L::Uppercase("VAR")],
                &[L::Uppercase("VAR_GLOBAL")],
            ]),
        ],
        VARIABLE_DECLARATION,
        &[L::Uppercase("END_VAR")],
    )]),
]);

const PROPERTY_GET_OR_SET: Layout = Layout(&[L::Repeat(&[L::BeginMiddleEnd(
    &[L::OneOf(&[
        &[L::Uppercase("VAR CONSTANT")],
        &[L::Uppercase("VAR PERSISTENT")],
        &[L::Uppercase("VAR RETAIN")],
        &[L::Uppercase("VAR_INST")],
        &[L::Uppercase("VAR")],
    ])],
    VARIABLE_DECLARATION,
    &[L::Uppercase("END_VAR")],
)])]);

#[derive(Debug)]
enum L {
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
    OneOf(&'static [&'static [L]]),
    Option(&'static [L]),
    BeginMiddleEnd(&'static [L], &'static [L], &'static [L]),
    Repeat(&'static [L]),
}

fn peel<'a>(code: &mut Code, text: &'a str) -> Result<&'a str> {
    *code = code.strip_prefix_str(text)?;
    Ok(text)
}

fn peel_uppercase<'a>(code: &mut Code, text: &'a str) -> Result<&'a str> {
    *code = code.strip_prefix_uppercase(text)?;
    Ok(text)
}

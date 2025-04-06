use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::code::Code;
use crate::components::{Component as C, Ether};
use crate::layout::{string_and_format_get_items, Layout as L};

#[derive(Debug)]
pub struct Implementation(pub Vec<C>);

impl FromStr for Implementation {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        let mut code = Code::from(input);
        match peel(&mut code) {
            Ok(implementation) => {
                if code.end_of_file() {
                    Ok(Self(implementation))
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot parse\n{code}"),
                    ))
                }
            }
            Err(_) => {
                if code.trim_start().end_of_file() {
                    Ok(Self(Vec::new()))
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot parse\n{code}"),
                    ))
                }
            }
        }
    }
}

pub fn peel(code: &mut Code) -> Result<Vec<C>> {
    let mut implementation = Vec::new();
    let mut code_clone = code.clone();

    while let Ok(items) = string_get_implementation_items(&mut code_clone) {
        implementation.extend(items);
    }
    for ether in Ether::peel(&mut code_clone)? {
        implementation.push(C::Ether(ether));
    }

    if implementation.is_empty() {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("No implementation\n{code}"),
        ))
    } else {
        *code = code_clone;
        Ok(implementation)
    }
}

fn string_get_implementation_items(code: &mut Code) -> Result<Vec<C>> {
    if let Ok(output) =
        string_and_format_get_items(code, &[L::Uppercase("RETURN"), L::Text(";"), L::LineFeed])
    {
        Ok(output)
    } else if let Ok(output) =
        string_and_format_get_items(code, &[L::Uppercase("EXIT"), L::Text(";"), L::LineFeed])
    {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, ASSIGNMENT) {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, IF) {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, CASE) {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, FOR) {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, WHILE) {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, REPEAT_UNTIL) {
        Ok(output)
    } else if let Ok(output) =
        string_and_format_get_items(code, &[L::Expression, L::Text(";"), L::LineFeed])
    {
        Ok(output)
    } else if let Ok(output) = string_and_format_get_items(code, &[L::Text(";"), L::LineFeed]) {
        Ok(output)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse\n{code}"),
        ))
    }
}

const ASSIGNMENT: &[L] = &[
    L::Expression,
    L::Space,
    L::OneOf(&[
        &[L::Text(":=")],
        &[L::Uppercase("R=")],
        &[L::Uppercase("S=")],
        &[L::Uppercase("REF=")],
    ]),
    L::Space,
    L::Expression,
    L::Option(&[L::Repeat(&[
        L::Space,
        L::OneOf(&[
            &[L::Text(":=")],
            &[L::Uppercase("R=")],
            &[L::Uppercase("S=")],
        ]),
        L::Space,
        L::Expression,
    ])]),
    L::Text(";"),
    L::LineFeed,
];

const IF: &[L] = &[L::BeginMiddleEnd(
    &[
        L::Uppercase("IF"),
        L::Space,
        L::Expression,
        L::Space,
        L::Uppercase("THEN"),
    ],
    &[L::Implementation],
    &[
        L::Option(&[L::Repeat(&[L::BeginMiddleEnd(
            &[
                L::Uppercase("ELSIF"),
                L::Space,
                L::Expression,
                L::Space,
                L::Uppercase("THEN"),
            ],
            &[L::Implementation],
            &[],
        )])]),
        L::Option(&[L::BeginMiddleEnd(
            &[L::Uppercase("ELSE")],
            &[L::Implementation],
            &[],
        )]),
        L::Uppercase("END_IF"),
    ],
)];

const CASE: &[L] = &[L::BeginMiddleEnd(
    &[
        L::Uppercase("CASE"),
        L::Space,
        L::Expression,
        L::Space,
        L::Uppercase("OF"),
    ],
    &[
        L::BeginMiddleEnd(
            &[L::Expression, L::Space, L::Text(":")],
            &[L::Implementation],
            &[],
        ),
        L::Option(&[L::BeginMiddleEnd(
            &[L::Uppercase("ELSE")],
            &[L::Implementation],
            &[],
        )]),
    ],
    &[L::Uppercase("END_CASE")],
)];

const FOR: &[L] = &[L::BeginMiddleEnd(
    &[
        L::Uppercase("FOR"),
        L::Space,
        L::Identifier,
        L::Space,
        L::Text(":="),
        L::Space,
        L::Expression,
        L::Space,
        L::Uppercase("TO"),
        L::Space,
        L::Expression,
        L::Space,
        L::Option(&[L::Uppercase("BY"), L::Space, L::Value, L::Space]),
        L::Uppercase("DO"),
    ],
    &[L::Implementation],
    &[L::Uppercase("END_FOR")],
)];

const WHILE: &[L] = &[L::BeginMiddleEnd(
    &[
        L::Uppercase("WHILE"),
        L::Space,
        L::Expression,
        L::Space,
        L::Uppercase("DO"),
    ],
    &[L::Implementation],
    &[L::Uppercase("END_WHILE")],
)];

const REPEAT_UNTIL: &[L] = &[L::BeginMiddleEnd(
    &[L::Uppercase("REPEAT")],
    &[L::Implementation],
    &[
        L::Uppercase("UNTIL"),
        L::Space,
        L::Expression,
        L::Space,
        L::Uppercase("END_REPEAT"),
    ],
)];

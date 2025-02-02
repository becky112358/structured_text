use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::components::{Component as C, Ether};
use crate::layout::{string_and_format_get_items, Layout as L};

#[derive(Debug)]
pub struct Declaration(pub Vec<C>);

impl FromStr for Declaration {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self> {
        if let Ok(declaration) = string_and_layout_get_declaration(input, ENUM) {
            Ok(declaration)
        } else if let Ok(declaration) = string_and_layout_get_declaration(input, STRUCT) {
            Ok(declaration)
        } else if let Ok(declaration) = string_and_layout_get_declaration(input, UNION) {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_layout_get_declaration(input, GLOBAL_VARIABLE_LIST)
        {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_layout_get_declaration(input, PROGRAM_ORGANISATION_UNIT)
        {
            Ok(declaration)
        } else if let Ok(declaration) =
            string_and_layout_get_declaration(input, PROPERTY_GET_OR_SET)
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

fn string_and_layout_get_declaration(input: &str, layout: &[L]) -> Result<Declaration> {
    let mut remainder = input.to_string();
    let mut items = string_and_format_get_items(&mut remainder, layout)?;
    for ether in Ether::peel(&mut remainder)? {
        items.push(C::Ether(ether));
    }
    let declaration = Declaration(items);

    if remainder.is_empty() {
        Ok(declaration)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse \n{remainder}"),
        ))
    }
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
const ENUM: &[L] = &[
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
];

#[rustfmt::skip]
const STRUCT: &[L] = &[
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
];

#[rustfmt::skip]
const UNION: &[L] = &[
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
];

const GLOBAL_VARIABLE_LIST: &[L] = &[L::BeginMiddleEnd(
    &[
        L::Uppercase("VAR_GLOBAL"),
        L::Option(&[L::OneOf(&[
            &[L::Space, L::Uppercase("CONSTANT")],
            &[L::Space, L::Uppercase("PERSISTENT")],
        ])]),
    ],
    VARIABLE_DECLARATION,
    &[L::Uppercase("END_VAR")],
)];

#[rustfmt::skip]
const PROGRAM_ORGANISATION_UNIT: &[L] = &[
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
];

const PROPERTY_GET_OR_SET: &[L] = &[L::Repeat(&[L::BeginMiddleEnd(
    &[L::OneOf(&[
        &[L::Uppercase("VAR CONSTANT")],
        &[L::Uppercase("VAR PERSISTENT")],
        &[L::Uppercase("VAR RETAIN")],
        &[L::Uppercase("VAR_INST")],
        &[L::Uppercase("VAR")],
    ])],
    VARIABLE_DECLARATION,
    &[L::Uppercase("END_VAR")],
)])];

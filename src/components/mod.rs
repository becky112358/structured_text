mod address;
pub use address::Address;
mod expression;
pub use expression::Expression;
mod begin_middle_end;
pub use begin_middle_end::BeginMiddleEnd;
mod data_type;
pub use data_type::DataType;
mod ether;
pub use ether::Ether;
mod identifier;
pub use identifier::{Identifier, IdentifierSub};
mod keywords;
pub(super) use keywords::KEYWORDS;
mod member;
pub(super) use member::Member;
mod value;
pub use value::Value;

use crate::dazzle;

#[derive(Debug)]
pub enum Component {
    Ether(Ether),
    Space,
    Address(Address),
    DataType(DataType),
    Expression(Expression),
    Identifier(Identifier),
    IdentifierSub(IdentifierSub),
    Text(&'static str),
    Uppercase(&'static str),
    Value(Value),
    BeginMiddleEnd(BeginMiddleEnd),
    Repeat(Vec<Component>),
    Filler(u8),
}

impl dazzle::Dazzle for Component {
    fn dazzle(&self, arguments: &mut dazzle::Dazzler) {
        match self {
            Self::Ether(inner) => inner.dazzle(arguments),
            Self::Space => match arguments.previous_character {
                dazzle::PreviousCharacter::Top
                | dazzle::PreviousCharacter::LineFeed
                | dazzle::PreviousCharacter::PendingSpace => (),
                dazzle::PreviousCharacter::Other => {
                    arguments.previous_character = dazzle::PreviousCharacter::PendingSpace
                }
            },
            Self::Address(inner) => inner.dazzle(arguments),
            Self::DataType(inner) => inner.dazzle(arguments),
            Self::Expression(inner) => inner.dazzle(arguments),
            Self::Identifier(inner) => inner.dazzle(arguments),
            Self::IdentifierSub(inner) => inner.dazzle(arguments),
            Self::Text(inner) => inner.dazzle(arguments),
            Self::Uppercase(inner) => inner.dazzle(arguments),
            Self::Value(inner) => inner.dazzle(arguments),
            Self::BeginMiddleEnd(inner) => inner.dazzle(arguments),
            Self::Repeat(inners) => {
                for inner in inners {
                    inner.dazzle(arguments);
                }
            }
            Self::Filler(count) => {
                for _ in 0..*count {
                    arguments.f.push(' ');
                }
            }
        }
    }
}

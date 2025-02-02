mod address;
pub use address::Address;
mod assignment;
pub use assignment::Assignment;
mod data_type;
pub use data_type::DataType;
mod ether;
pub use ether::Ether;
mod expression;
pub use expression::Expression;
mod function_call;
pub use function_call::FunctionCall;
mod identifier;
pub use identifier::{Identifier, IdentifierSub};
mod value;
pub use value::Value;

#[derive(Debug)]
pub enum Component {
    Ether(Ether),
    Space,
    Uppercase(&'static str),
    Text(&'static str),
    Identifier(Identifier),
    IdentifierSub(IdentifierSub),
    Address(Address),
    DataType(DataType),
    Value(Value),
    Assignment(Assignment),
    BeginMiddleEnd(Vec<Component>, Vec<Vec<Component>>, Vec<Component>),
    Repeat(Vec<Component>),
    Filler(u8),
}

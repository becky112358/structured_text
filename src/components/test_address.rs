use super::*;

#[test]
fn ok() {
    let mut input = Code::from("AT %Q* : INT;");
    assert_eq!(Address::Q, Address::peel(&mut input).unwrap());
}

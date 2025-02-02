use super::*;

#[test]
fn ok() {
    let mut input = String::from("AT %Q* : INT;");
    assert_eq!(Address::Q, Address::peel(&mut input).unwrap());
}

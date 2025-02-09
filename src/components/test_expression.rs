use super::*;

#[test]
fn equation_then_comment() {
    let mut input = Code::from("(3 + 8) * 2 / 4 (* maths! *) ;");

    assert_eq!(
        Expression::peel(&mut input).unwrap(),
        Expression(String::from("(3 + 8) * 2 / 4"))
    );
}

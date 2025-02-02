use super::*;

#[test]
fn top() {
    let input_output = String::from(
        "{pragma0}
{pragma1}
// comment0
(* comment1 *)
(* comment2 *)
",
    );

    let mut input = input_output.clone();
    let mut dazzler = dazzle::Dazzler::default();
    for ether in Ether::peel(&mut input).unwrap() {
        ether.dazzle(&mut dazzler);
    }

    assert_eq!(dazzler.f, input_output);
}

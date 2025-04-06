use super::*;

#[test]
fn top() {
    let input_output = "{pragma0}
{pragma1}
// comment0
(* comment1 *)
(* comment2 *)
";

    let mut input = Code::from(input_output);
    let output = String::from(input_output);

    let mut dazzler = dazzle::Dazzler::default();
    for ether in Ether::peel(&mut input).unwrap() {
        ether.dazzle(&mut dazzler);
    }

    assert_eq!(dazzler.f, output);
}

#[test]
fn nested_comment() {
    let mut input = Code::from("(* Hello! (* I am a nested comment! *) Goodbye! *)");

    let output = Ether::peel(&mut input);

    assert!(output.is_ok());

    assert!(input.end_of_file());
}

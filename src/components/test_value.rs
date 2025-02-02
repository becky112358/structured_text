use super::*;

#[test]
fn array() {
    let mut input = String::from("[0, 4, SOME_CONSTANT, -2]");
    assert_eq!(
        Value::peel(&mut input).unwrap(),
        Value(ValueInner::Array(Array(
            vec![],
            vec![
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("0")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("4")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("SOME_CONSTANT")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("-2")))),
                    vec![]
                ),
            ]
        ))),
    );
}

#[test]
fn array_and_array_accessor() {
    let mut input_output = String::from("[0, 4, SOME_ARRAY[pp], -2, x[3]]");
    assert_eq!(
        Value::peel(&mut input_output).unwrap(),
        Value(ValueInner::Array(Array(
            vec![],
            vec![
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("0")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("4")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("SOME_ARRAY[pp]")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("-2")))),
                    vec![]
                ),
                (
                    Assignment::Value(Value(ValueInner::Flat(String::from("x[3]")))),
                    vec![]
                ),
            ]
        ))),
    );

    let mut dazzler = dazzle::Dazzler::default();
    Value::peel(&mut input_output).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, input_output);
}

#[test]
fn array_of_strings() {
    let mut input = String::from("['hello', 'world!']");
    assert!(Value::peel(&mut input).is_ok());
}

#[test]
fn array_of_subidentifiers() {
    let mut input = String::from("[item.value0, item.value1, item.value2]");
    assert!(Value::peel(&mut input).is_ok());
}

#[test]
fn structure() {
    let mut input = String::from("(this := [A, B, C_D], that := 12.3)");
    assert_eq!(
        Value(ValueInner::Struct(Struct(vec![
            (
                Identifier(String::from("this")),
                Assignment::Value(Value(ValueInner::Array(Array(
                    vec![],
                    vec![
                        (
                            Assignment::Value(Value(ValueInner::Flat(String::from("A")))),
                            vec![]
                        ),
                        (
                            Assignment::Value(Value(ValueInner::Flat(String::from("B")))),
                            vec![]
                        ),
                        (
                            Assignment::Value(Value(ValueInner::Flat(String::from("C_D")))),
                            vec![]
                        ),
                    ]
                )))),
                vec![],
            ),
            (
                Identifier(String::from("that")),
                Assignment::Value(Value(ValueInner::Flat(String::from("12.3")))),
                vec![],
            ),
        ]))),
        Value::peel(&mut input).unwrap(),
    );
}

#[test]
fn structure_with_array_of_one_dimension() {
    let mut input = String::from("(blah := 0.1, bleb := [x])");
    assert!(Value::peel(&mut input).is_ok());
}

#[test]
fn structure_with_comments() {
    let mut input = String::from(
        "(
    zeroth := 0.0, // zeroth
    first := 1.0, // first
    second := 2.2, (* second and a bit, actually *)
    third := 3.14, // pi! (approximately)
    fourth := 4 // fourth
)",
    );

    let output = String::from(
        "    (
        zeroth := 0.0,  // zeroth
        first  := 1.0,  // first
        second := 2.2,  (* second and a bit, actually *)
        third  := 3.14, // pi! (approximately)
        fourth := 4     // fourth
    )",
    );

    let mut dazzler = dazzle::Dazzler::default();
    Value::peel(&mut input).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, output);
}

#[test]
fn structure_very_long() {
    let mut input = String::from("(abc := 'abc', then_a_number := 123, xyz := 'xyz', then_a_constant := MY_CONSTANT, nested_struct := (a := 0, b := 1), keep_it_up := [0, 1, 2], dont_stop_me_now := 'Im having such a good time')");
    let output = String::from(
        "    (
        abc              := 'abc',
        then_a_number    := 123,
        xyz              := 'xyz',
        then_a_constant  := MY_CONSTANT,
        nested_struct    := (a := 0, b := 1),
        keep_it_up       := [0, 1, 2],
        dont_stop_me_now := 'Im having such a good time'
    )",
    );

    let mut dazzler = dazzle::Dazzler::default();
    Value::peel(&mut input).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, output);
}

#[test]
fn string() {
    let mut input = String::from("'Trees!';");
    assert_eq!(
        Value(ValueInner::String(String::from("Trees!"))),
        Value::peel(&mut input).unwrap(),
    );
    assert_eq!(input, String::from(";"));
}

#[test]
fn string_with_escapes() {
    let mut input =
        String::from("'Let$'s escape! We shall be $$free$$ like the wind in the trees.'");
    assert_eq!(
        Value(ValueInner::String(String::from(
            "Let's escape! We shall be $free$ like the wind in the trees."
        ))),
        Value::peel(&mut input).unwrap(),
    );
}

#[test]
fn flat() {
    let mut input = String::from("3.14;");
    assert_eq!(
        Value(ValueInner::Flat(String::from("3.14"))),
        Value::peel(&mut input).unwrap(),
    );
    assert_eq!(input, String::from(";"));
}

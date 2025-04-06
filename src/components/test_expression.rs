use super::*;

#[test]
fn call_method() {
    let input = String::from("my_functionblock[first + 3].method(x := 0, y => z)");
    let mut code = Code::from(&input);

    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut code).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, input);
    assert!(code.end_of_file());
}

#[test]
fn equation_then_comment() {
    let mut input = Code::from("(3 + 8) * 2 / 4 (* maths! *) ;");

    let expected_output = String::from("(3 + 8) * 2 / 4");

    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut input).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, expected_output);
}

#[test]
fn access_super() {
    let input = String::from("SUPER^.do_thing(123)");
    let mut code = Code::from(&input);

    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut code).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, input);
    assert!(code.end_of_file());
}

#[test]
fn dereference() {
    let input = String::from("byte_buffer[3 + x]^");
    let mut code = Code::from(&input);

    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut code).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, input);
    assert!(code.end_of_file());
}

#[test]
fn not_x_and_not_y() {
    let mut code = Code::from("not x and not y");

    let output_string = String::from("NOT x AND NOT y");
    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut code).unwrap().dazzle(&mut dazzler);
    assert_eq!(dazzler.f, output_string);
}

#[test]
fn condition_left_to_right() {
    let input = String::from("a + b AND c - d OR e OR g <> h AND q - 3 < r");
    let mut code = Code::from(&input);

    let output = Expression(ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
        left: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
            left: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                left: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                    left: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                        left: ExpressionInner::Value(Value::peel(&mut Code::from("a")).unwrap()),
                        ethers0: vec![],
                        operator: Operator::Add,
                        ethers1: vec![],
                        right: ExpressionInner::Value(Value::peel(&mut Code::from("b")).unwrap()),
                    })),
                    ethers0: vec![],
                    operator: Operator::And,
                    ethers1: vec![],
                    right: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                        left: ExpressionInner::Value(Value::peel(&mut Code::from("c")).unwrap()),
                        ethers0: vec![],
                        operator: Operator::Subtract,
                        ethers1: vec![],
                        right: ExpressionInner::Value(Value::peel(&mut Code::from("d")).unwrap()),
                    })),
                })),
                ethers0: vec![],
                operator: Operator::Or,
                ethers1: vec![],
                right: ExpressionInner::Value(Value::peel(&mut Code::from("e")).unwrap()),
            })),
            ethers0: vec![],
            operator: Operator::Or,
            ethers1: vec![],
            right: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                left: ExpressionInner::Value(Value::peel(&mut Code::from("g")).unwrap()),
                ethers0: vec![],
                operator: Operator::NotEqualTo,
                ethers1: vec![],
                right: ExpressionInner::Value(Value::peel(&mut Code::from("h")).unwrap()),
            })),
        })),
        ethers0: vec![],
        operator: Operator::And,
        ethers1: vec![],
        right: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
            left: ExpressionInner::BinaryOperator(Box::new(BinaryOperator {
                left: ExpressionInner::Value(Value::peel(&mut Code::from("q")).unwrap()),
                ethers0: vec![],
                operator: Operator::Subtract,
                ethers1: vec![],
                right: ExpressionInner::Value(Value::peel(&mut Code::from("3")).unwrap()),
            })),
            ethers0: vec![],
            operator: Operator::LessThan,
            ethers1: vec![],
            right: ExpressionInner::Value(Value::peel(&mut Code::from("r")).unwrap()),
        })),
    })));

    assert_eq!(Expression::peel(&mut code).unwrap(), output);
}

#[test]
fn lots() {
    let mut code = Code::from("(((((A AND B AND NOT C AND D < 2) OR (A AND B OR C)) AND E > 5) OR F) OR (G AND H) OR J OR ((K > 4.2) AND NOT L) OR NOT M)");
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn with_comments() {
    let mut code = Code::from(
        "(A AND B AND C) // A && B && C
    OR (C AND D AND E) // C && D && E
    OR (P < Q AND NOT Z)",
    );
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_no_arguments() {
    let mut input = Code::from("no.arguments()");
    assert!(Expression::peel(&mut input).is_ok());
}

#[test]
fn function_call_unnamed_inputs() {
    let mut code =
        Code::from("REAL_TO_INT(Main.blah.zzz[abc].something.final_thing * 3.14 / 100.0)");
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_named_inputs() {
    let mut code = Code::from("function_call(var0 := SOME_CONSTANT, var1 := 2.3)");
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_input_and_output() {
    let mut code = Code::from("function_call(var0 := SOME_CONSTANT, var1 => local_variable)");
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_unassigned_output() {
    let mut code = Code::from("TON(Q => )");
    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_unit_test() {
    let mut code = Code::from(
        "AssertEquals_BOOL(
    Expected := FALSE, // we expect false
    Actual   := (x.y AND thing.0 AND (a.b = c.d)),
    Message  := 'Explanation'
)",
    );

    assert!(Expression::peel(&mut code).is_ok());
    assert!(code.end_of_file());
}

#[test]
fn function_call_short() {
    let input = String::from(
        "my_function(a,
            b,
            c,
        )",
    );

    let output = String::from("my_function(a, b, c)");

    let mut dazzler = dazzle::Dazzler::default();
    Expression::peel(&mut Code::from(&input))
        .unwrap()
        .dazzle(&mut dazzler);
    assert_eq!(dazzler.f, output);
}

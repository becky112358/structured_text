use super::*;

#[test]
fn function_call_with_named_inputs() {
    let mut input = String::from("function_call(var0 := SOME_CONSTANT, var1 := 2.3)");
    assert!(FunctionCall::peel(&mut input).is_ok());
}

#[test]
fn function_call_with_input_and_output() {
    let mut input = String::from("function_call(var0 := SOME_CONSTANT, var1 => local_variable)");
    assert!(FunctionCall::peel(&mut input).is_ok());
}

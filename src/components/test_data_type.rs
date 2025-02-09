use super::*;

#[test]
fn simple() {
    let mut input = Code::from("xyz;");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::Flat(String::from("xyz")),
    );
}

#[test]
fn array() {
    let mut input = Code::from("ARRAY [2..8] OF xyz;");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::Array(
            ArrayRange::LowerUpper(String::from("2"), String::from("8")),
            Box::new(DataType::Flat(String::from("xyz")))
        ),
    );
}

#[test]
fn array_of_array() {
    let mut input = Code::from("ARRAY [-2..12] OF ARRAY [3..8] OF xyz;");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::Array(
            ArrayRange::LowerUpper(String::from("-2"), String::from("12")),
            Box::new(DataType::Array(
                ArrayRange::LowerUpper(String::from("3"), String::from("8")),
                Box::new(DataType::Flat(String::from("xyz")))
            ))
        ),
    );
}

#[test]
fn array_of_array_with_ofs() {
    let mut input = Code::from("ARRAY[0..numberOfValues] OF ARRAY[0..numberOfValues] OF ULINT");
    assert!(DataType::peel(&mut input).is_ok());
}

#[test]
fn array_messy() {
    let mut input = Code::from("array [3..LOTS] Of   xyz  ;");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::Array(
            ArrayRange::LowerUpper(String::from("3"), String::from("LOTS")),
            Box::new(DataType::Flat(String::from("xyz")))
        ),
    );
}

#[test]
fn array_of_star_length() {
    let mut input = Code::from("ARRAY [*] OF UINT");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::Array(
            ArrayRange::Star,
            Box::new(DataType::Flat(String::from("UINT")))
        ),
    );
}

#[test]
fn string() {
    let mut input = Code::from("STRING := 'hello';");
    assert_eq!(DataType::peel(&mut input).unwrap(), DataType::String(None));
}

#[test]
fn string_with_length() {
    let mut input = Code::from("STRING( 248 ) := 'hello';");
    assert_eq!(
        DataType::peel(&mut input).unwrap(),
        DataType::String(Some(248)),
    );
}

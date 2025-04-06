use super::*;

#[test]
fn underscores() {
    let input = String::from(
        "IF NOT __ISVALIDREF(xyz) THEN
    RETURN;
END_IF",
    );

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assignment_to_minus_bracket_equation() {
    let input = String::from("output.0 := -(b0.x * b1.z - b1.x * b0.z);");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assignment_to_string() {
    let input = String::from("data:='$n';");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assignment_spacey() {
    let input = String::from("thing0.thing1    .thing2 := thing3;");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assignment_to_function() {
    let input = String::from("a := fun(var0 := x0, var1 := x1.y);");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assignment_to_function_with_array_accessor() {
    let input = String::from("my_value := LREAL_TO_REAL(something[x + 1]);");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn assign_assign() {
    let input = String::from("a R= b := c S= d > 0;");

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn if_only() {
    let mut input = String::from(
        "IF x < 2 THEN
    y[s] := p;
END_IF
",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn if_elsif() {
    let mut input = String::from(
        "IF x < 2 THEN
    y[s] := p;
ELSIF x = 3 THEN
    y[t] := q;
END_IF
",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn if_else() {
    let mut input = String::from(
        "IF x < 2 THEN
    y[s] := p;
ELSE
    t[u] := r;
END_IF
",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn if_elsif_else() {
    let mut input = String::from(
        "IF x < 2 THEN
    y[s] := p;
ELSIF x = 3 THEN
    y[t] := q;
ELSE
    t[u] := r;
END_IF
",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn if_with_function_and_maths() {
    let mut input = String::from(
        "IF ABS(angle) > 180.0 THEN
    // do something intelligent
ELSIF angle < 180.0 THEN
    // do something else
ELSE
    // final case
END_IF",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn if_value() {
    let mut input = String::from(
        "IF x THEN
    y := do_thing(a:=b, c:=d.e); // useful comment
END_IF",
    );

    assert!(Implementation::from_str(&mut input).is_ok());
}

#[test]
fn case() {
    let input = String::from(
        "CASE x OF
    1 :
        do_thing();
    2 :
        x := y + z;
    3 :
        ;
    ELSE
        // blah blah blah
        a := 3.9;
END_CASE
",
    );

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn case_with_comments() {
    let input = String::from(
        "CASE ints_for_enums OF
    0 : // Comment because ints are being used in place of enums...
        IF do_thing THEN
            do_thing := FALSE;
            ints_for_enums := 10;
        END_IF

    10 : (* Comment which is only necessary because we are not using enums *)
        my_function.do_thing();
        my_functionblock.field.method(0).and_another_method().
                         field.yet_another_method(10000);

        something(x := y);
        ints_for_enums := 20;

    20 :
        complete := a.b(
                        c := d,
                        e => f,
                        g => h);

        IF complete THEN
            ints_for_enums := 30;
        END_IF

    30 : // blah blah blah
        timer_on(IN := TRUE, PT :=T#500MS);
        IF timer_on.Q THEN
            TON_Timer(IN := FALSE);
            complete := FALSE;
            ints_for_enums := 0;
        END_IF

END_CASE
",
    );

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn for_loop_lowercase() {
    let input = String::from(
        "for   x   := 0   TO n DO
    y :=  y + 3  ;
    function_call(y, 8)  ;
END_FOR",
    );

    let output_string = String::from(
        "FOR x := 0 TO n DO
    y := y + 3;
    function_call(y, 8);
END_FOR
",
    );

    let output = align(&input).unwrap();

    assert_eq!(output, output_string);
}

#[test]
fn for_loop_with_calculated_end() {
    let input = String::from(
        "FOR x := abc TO find_the_end.for(z * 0.5) DO
    CONCAT(some_string, another_string);
END_FOR
",
    );

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn trailing_semicolons() {
    let input = String::from(
        "FOR x := 1 TO ABC DO
    ;
END_FOR
;
",
    );

    assert!(Implementation::from_str(&input).is_ok());
}

#[test]
fn lone_statements() {
    let input = String::from(
        "x := 3;
z := 2;

function_call(input := x, output => a);

abc := (z + 2 = 3) OR y;
",
    );

    assert_eq!(align(&input).unwrap(), input);
}

#[test]
fn blank_lines_short() {
    let input = String::from(
        "FOR x := 1 TO 5 DO
    IF x < 3 THEN
        y := 2;
    END_IF

    IF z > 5 THEN
        ;
    END_IF
END_FOR
",
    );

    assert_eq!(align(&input).unwrap(), input);
}

#[test]
fn blank_lines() {
    let input = String::from(
        "FOR x := 1 TO ABC DO
    IF x < 3 THEN
        y := 2;

END_IF


        IF p + q = 5 THEN
    function_call();
    ELSIF p + q + z = 8 THEN
        another_function_call();


            
            ELSE
                p := -3;
                    END_IF

END_FOR


FOR y := 1 TO CBA DO
    IF z < 3 THEN
        // do a thing
        do_thing();
    ELSE
        IF z < 7 THEN
            // do another thing
            do_another_thing();

        ELSIF z > 12 THEN
        (*comment*)
            maybe_do_something_useful();
        END_IF



    END_IF

END_FOR",
    );

    let output = String::from(
        "FOR x := 1 TO ABC DO
    IF x < 3 THEN
        y := 2;

    END_IF

    IF p + q = 5 THEN
        function_call();
    ELSIF p + q + z = 8 THEN
        another_function_call();

    ELSE
        p := -3;
    END_IF

END_FOR

FOR y := 1 TO CBA DO
    IF z < 3 THEN
        // do a thing
        do_thing();
    ELSE
        IF z < 7 THEN
            // do another thing
            do_another_thing();

        ELSIF z > 12 THEN
            (* comment *)
            maybe_do_something_useful();
        END_IF

    END_IF

END_FOR
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

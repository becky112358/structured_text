use super::*;

#[test]
fn enumerator() {
    let input = String::from(
        "{attribute 'qualified_only'}
{ attribute 'strict'   }
TYPE Numbers    :
(
Zero:=0,
 One := 1,
  Two,
   Three   :=   3   ,
\tCrazy := 5,
    Six,
    Seven
);
END_TYPE
",
    );

    let output = String::from(
        "{attribute 'qualified_only'}
{attribute 'strict'}
TYPE Numbers :
(
    Zero  := 0,
    One   := 1,
    Two,
    Three := 3,
    Crazy := 5,
    Six,
    Seven
);
END_TYPE
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn structure() {
    let input = String::from(
        "TYPE BigGroup :
STRUCT

// Section 1
    x0 : REAL;
    x1 : REAL;
    x2 : REAL;

      // Section 2
    y0 : BOOL;
    y1 : BOOL;
    y2 : BOOL;
    y3 : BOOL;




  (* Section 3 *)
    z0 : INT;
    z1 : INT;
    z2 : INT;



    // Trailing semicolon!
    ;


      (*Section 4*)
      a0 : MyDataType;
      a1 : DifferentDataType;



   END_STRUCT
END_TYPE

",
    );

    let output = String::from(
        "TYPE BigGroup :
STRUCT
    // Section 1
    x0 : REAL;
    x1 : REAL;
    x2 : REAL;

    // Section 2
    y0 : BOOL;
    y1 : BOOL;
    y2 : BOOL;
    y3 : BOOL;

    (* Section 3 *)
    z0 : INT;
    z1 : INT;
    z2 : INT;

    // Trailing semicolon!
    ;

    (* Section 4 *)
    a0 : MyDataType;
    a1 : DifferentDataType;
END_STRUCT
END_TYPE
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn global_variable_list() {
    let input = String::from(
        "{attribute 'qualified_only'}

 VAR_GLOBAL

    // Insightful comment
    // Less insightful burbling

    var0   : BOOL := TRUE;     // Getting carried away
    var1   : BOOL := FALSE;    // Too many comments!
    var2   : BOOL := TRUE;


END_VAR
",
    );

    let output = String::from(
        "{attribute 'qualified_only'}
VAR_GLOBAL
    // Insightful comment
    // Less insightful burbling

    var0 : BOOL := TRUE;  // Getting carried away
    var1 : BOOL := FALSE; // Too many comments!
    var2 : BOOL := TRUE;
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn program() {
    let input = String::from(
        "PROGRAM Something
VAR
    one AT %I* : INT;
    two AT %I* : INT;
    three AT %I* : INT;
    four AT %I* : INT;
    a AT %Q* : BOOL;
    b AT %Q* : BOOL;
    c AT %Q* : BOOL;
    xyz : REAL;
    longer_name : STRING;
END_VAR
",
    );

    let output = String::from(
        "PROGRAM Something
VAR
    one   AT %I* : INT;
    two   AT %I* : INT;
    three AT %I* : INT;
    four  AT %I* : INT;
    a     AT %Q* : BOOL;
    b     AT %Q* : BOOL;
    c     AT %Q* : BOOL;
    xyz          : REAL;
    longer_name  : STRING;
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn function_block() {
    let input = String::from(
        "FUNCTION_BLOCK Something
VAR_INPUT
  one    : REAL ;
    two : REAL;
   three:BOOL;
END_VAR
VAR_OUTPUT
      x  :   REAL;
    y : REAL;
END_VAR
VAR
    holder : REAL := 1.5;
    another : REAL := 12.8;
    something_else : INT := -3;
    temporary : BOOL := FALSE;



END_VAR



",
    );

    let output = String::from(
        "FUNCTION_BLOCK Something
VAR_INPUT
    one            : REAL;
    two            : REAL;
    three          : BOOL;
END_VAR
VAR_OUTPUT
    x              : REAL;
    y              : REAL;
END_VAR
VAR
    holder         : REAL := 1.5;
    another        : REAL := 12.8;
    something_else : INT  := -3;
    temporary      : BOOL := FALSE;
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn function_block_extends() {
    let input = String::from(
        "FUNCTION_BLOCK Longer EXTENDS Shorter.ReallyShortBit_xyz
VAR
    blah          : Blah;
    fantastic_numbers: TheBestEver := (x := 1, y := 3.14);

    something : BOOL;
END_VAR",
    );

    assert!(align(&input).is_ok());
}

#[test]
fn implicit_enum() {
    let input = String::from(
        "FUNCTION_BLOCK MyFunctionBlock
VAR
    implicit_enum : (StateZero, StateOne, StateTwo, StateThree);
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), input);
}

#[test]
fn method() {
    let input = String::from(
        "METHOD Wow
VAR
    xyz : Something := (x := 1, y := 2, z := 3);
    abc : AnotherThing := (a := -1, y := -2, z := 3);
    blah : BOOL;
    questionable : Surprise := (surp := TRUE, rise := OVER_THERE);
END_VAR
",
    );

    let output = String::from(
        "METHOD Wow
VAR
    xyz          : Something    := (x := 1, y := 2, z := 3);
    abc          : AnotherThing := (a := -1, y := -2, z := 3);
    blah         : BOOL;
    questionable : Surprise     := (surp := TRUE, rise := OVER_THERE);
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), output);
}

#[test]
fn variable_assignment_to_function() {
    let input = String::from(
        "FUNCTION_BLOCK MyFunctionBlock
VAR
    var0 : USINT                := add_three_numbers(1, 2, 3);
    var1 : AnotherFunctionBlock := AnotherFunctionBlock(init_xyz := 'xyz', init_abc := [0.1, -0.9]);
    var2 : REAL                 := Something.method(8, 9, 10);
    var3 : ARRAY [1..5] OF REAL := function_without_arguments();
END_VAR
",
    );

    assert_eq!(align(&input).unwrap(), input);
}

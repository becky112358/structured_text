use super::*;

#[test]
fn file_minimal() {
    const FILE: &str = "<><Declaration><![CDATA[content]]></Declaration>";
    assert!(File::from_str(FILE).is_ok());
    assert_eq!(FILE, File::from_str(FILE).unwrap().to_string());

    let file = File::from_str(FILE).unwrap();

    assert_eq!("<><Declaration><![CDATA[", file.chaff0);
    assert_eq!("content", file.declaration);
    assert_eq!("]]></Declaration>", file.chaff1);
    assert!(file.chunks.is_empty());

    for content in file.into_iter() {
        assert_eq!((Content::Declaration, "content"), content);
    }
}

#[test]
fn file_longer() {
    const FILE: &str = r#"<xml>
  <Declaration blah>
    <![CDATA[content line 0
content line 1
content line 2
]]>
  </Declaration>
  <Implementation>
    <![CDATA[short implementation
]]>
  </Implementation>
  <Declaration>
    <![CDATA[bit more
]]>
  </Declaration>
  <Implementation>
    <![CDATA[last bit
]]>
  </Implementation>"#;

    let file = File::from_str(FILE).unwrap();
    let mut contents = file.into_iter();
    assert_eq!(
        contents.next().unwrap(),
        (
            Content::Declaration,
            "content line 0\ncontent line 1\ncontent line 2\n"
        )
    );
    assert_eq!(
        contents.next().unwrap(),
        (Content::Implementation, "short implementation\n")
    );
    assert_eq!(
        contents.next().unwrap(),
        (Content::Declaration, "bit more\n")
    );
    assert_eq!(
        contents.next().unwrap(),
        (Content::Implementation, "last bit\n")
    )
}

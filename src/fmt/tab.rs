use std::io::{Error, ErrorKind, Result};

const TAB: char = '\t';
const TAB_WIDTH: usize = 4;

pub(super) fn replace_with_whitespace(input: &str) -> Result<String> {
    if !input.contains(TAB) {
        return Ok(input.to_string());
    }

    let mut output = String::new();

    for line in input.lines() {
        let mut line_no_tab = line.to_string();
        while let Some(i) = line_no_tab.find(TAB) {
            let spaces = match i % TAB_WIDTH {
                0 => "    ",
                1 => "   ",
                2 => "  ",
                3 => " ",
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("{i} % {} gave {}", TAB_WIDTH, i % TAB_WIDTH),
                    ))
                }
            };
            line_no_tab = line_no_tab.replacen(TAB, spaces, 1);
        }
        output = format!("{output}{line_no_tab}\n");
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            String::from("text    text  text  text            text    text\n"),
            replace_with_whitespace("text\ttext  text\ttext\t\t\ttext \ttext\n").unwrap()
        );
    }
}

use std::io::Result;

pub(super) fn trim(input: &str) -> Result<String> {
    let mut output = String::new();
    for line in input.lines() {
        output = format!("{output}{}\n", line.trim_end());
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            String::from("line0\nline1\nline2\nline3\n\n\n"),
            trim("line0    \nline1\t\nline2\nline3        \n\n\n").unwrap()
        );
    }
}

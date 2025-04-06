use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::str::Chars;

#[derive(Clone, Debug)]
pub struct Code<'a> {
    content: &'a str,
    cursor: usize,
}

impl fmt::Display for Code<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.content[self.cursor..])
    }
}

impl<'a> Code<'a> {
    pub fn chars(&self) -> Chars<'a> {
        self.content[self.cursor..].chars()
    }

    pub fn end_of_file(&self) -> bool {
        self.cursor == self.content.len()
    }

    pub fn from(content: &'a str) -> Self {
        Self { content, cursor: 0 }
    }

    pub fn peel(&mut self, by: usize) -> Result<()> {
        if self.cursor + by <= self.content.len() {
            self.cursor += by;
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Not enough code remaining to peel by {by}"),
            ))
        }
    }

    pub fn strip_between_and_trim_inner(&mut self, start: &str, end: &str) -> Result<String> {
        self.strip_between_and_trim_inner_common(start, end, false)
    }

    pub fn strip_between_nestable_and_trim_inner(
        &mut self,
        start: &str,
        end: &str,
    ) -> Result<String> {
        self.strip_between_and_trim_inner_common(start, end, true)
    }

    fn strip_between_and_trim_inner_common(
        &mut self,
        start: &str,
        end: &str,
        nestable: bool,
    ) -> Result<String> {
        if !self.content[self.cursor..].starts_with(start) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {start}\n{self}"),
            ));
        }

        let mut index_end = self.cursor
            + start.len()
            + self.content[self.cursor + start.len()..]
                .find(end)
                .ok_or(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find {end}\n{self}"),
                ))?;

        if nestable {
            let mut index_start_inner = self.cursor;
            while let Some(inner_inner) =
                self.content[index_start_inner + start.len()..index_end].find(start)
            {
                index_end = index_end
                    + end.len()
                    + self.content[index_end + end.len()..]
                        .find(end)
                        .ok_or(Error::new(
                            ErrorKind::InvalidData,
                            format!("Cannot find {end}\n{self}"),
                        ))?;
                index_start_inner = index_start_inner + start.len() + inner_inner;
            }
        }

        let inner = self.content[self.cursor + start.len()..index_end]
            .trim()
            .to_string();
        self.cursor = index_end + end.len();
        Ok(inner)
    }

    pub fn strip_from_and_trim_inner(&mut self, start: &str) -> Result<String> {
        if !self.content[self.cursor..].starts_with(start) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {start}\n{self}"),
            ));
        }

        let inner = self.content[self.cursor + start.len()..self.content.len()]
            .trim()
            .to_string();
        self.cursor = self.content.len();
        Ok(inner)
    }

    pub fn strip_prefix(&self, c: char) -> Result<Self> {
        let mut output = self.to_owned();
        if self.content[self.cursor..].starts_with(c) {
            output.cursor += c.len_utf8();
            Ok(output)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {c}\n{self}"),
            ))
        }
    }

    pub fn strip_prefix_str(&self, text: &str) -> Result<Self> {
        let mut output = self.to_owned();
        if self.content[self.cursor..].starts_with(text) {
            output.cursor += text.len();
            Ok(output)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {text}\n{self}"),
            ))
        }
    }

    pub fn strip_prefix_uppercase(&self, text: &str) -> Result<Self> {
        let mut output = self.to_owned();
        if self.content[self.cursor..].to_uppercase().starts_with(text) {
            output.cursor += text.len();
            Ok(output)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {text} (case-insensitiive)\n{self}"),
            ))
        }
    }

    pub fn trim_start(&self) -> Self {
        let mut cursor = self.cursor;
        for c in self.content[self.cursor..].chars() {
            if c.is_whitespace() {
                cursor += c.len_utf8();
            } else {
                break;
            }
        }
        Self {
            content: self.content,
            cursor,
        }
    }
}

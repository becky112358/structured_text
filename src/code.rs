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

    pub fn starts_with(&self, c: char) -> bool {
        self.content[self.cursor..].starts_with(c)
    }

    pub fn starts_with_str(&self, text: &str) -> bool {
        self.content[self.cursor..].starts_with(text)
    }

    pub fn strip_between_and_trim_inner(&mut self, start: &str, end: &str) -> Result<String> {
        if !self.content[self.cursor..].starts_with(start) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Does not start with {start}\n{self}"),
            ));
        }

        let index_end = match self.content[self.cursor + start.len()..].find(end) {
            Some(ie) => ie,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find {end}\n{self}"),
                ))
            }
        };

        let inner = self.content[self.cursor + start.len()..self.cursor + start.len() + index_end]
            .trim()
            .to_string();
        self.cursor += start.len() + index_end + end.len();
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
        let mut output = self.to_owned();
        for c in self.content[self.cursor..].chars() {
            if c.is_whitespace() {
                output.cursor += c.len_utf8();
            } else {
                break;
            }
        }
        output
    }
}

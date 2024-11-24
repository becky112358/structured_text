use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

const CONTENT_START: &str = "<![CDATA[";
const CONTENT_END: &str = "]]>";

pub struct File {
    chaff0: String,
    declaration: String,
    chaff1: String,
    chunks: Vec<Chunk>,
}

struct Chunk {
    what: Content,
    content: String,
    chaff: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Content {
    Declaration,
    Implementation,
}

impl FromStr for File {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let index = match input.find(CONTENT_START) {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Expected a Structured Text file but cannot find `{CONTENT_START}`"),
                ))
            }
        };

        let (chaff0, remainder) = match input.split_at_checked(index + CONTENT_START.len()) {
            Some((c, r)) => (c, r),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("File does not seem to contain content after `{CONTENT_START}`"),
                ))
            }
        };

        let index = match remainder.find(CONTENT_END) {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Expected a Structured Text file but cannot find `{CONTENT_END}`"),
                ))
            }
        };

        let (declaration, remainder) = match remainder.split_at_checked(index) {
            Some((d, r)) => (d, r),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("File does not seem to contain XML after `{CONTENT_END}`"),
                ))
            }
        };

        let index = match remainder.find(CONTENT_START) {
            Some(i) => i,
            None => {
                return Ok(Self {
                    chaff0: chaff0.to_string(),
                    declaration: declaration.to_string(),
                    chaff1: remainder.to_string(),
                    chunks: Vec::new(),
                });
            }
        };

        let (chaff1, remainder) = match remainder.split_at_checked(index + CONTENT_START.len()) {
            Some((c, r)) => (c, r),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("File does not seem to contain content after `{CONTENT_START}`"),
                ))
            }
        };

        let mut chaff_previous = chaff1.to_owned();
        let mut chunks = Vec::new();
        let mut remainder = remainder;
        while let Some(index) = remainder.find(CONTENT_END) {
            let (content, r) = match remainder.split_at_checked(index) {
                Some((c, r)) => (c, r),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("File does not seem to contain XML after `{CONTENT_END}`"),
                    ))
                }
            };

            let index = match r.find(CONTENT_START) {
                Some(i) => i,
                None => {
                    add_chunk(
                        &mut chunks,
                        &chaff_previous,
                        content.to_string(),
                        r.to_string(),
                    )?;
                    return Ok(Self {
                        chaff0: chaff0.to_string(),
                        declaration: declaration.to_string(),
                        chaff1: chaff1.to_string(),
                        chunks,
                    });
                }
            };

            let (chaff, r) = match r.split_at_checked(index + CONTENT_START.len()) {
                Some((c, r)) => (c, r),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("File does not seem to contain content after `{CONTENT_START}`"),
                    ))
                }
            };

            add_chunk(
                &mut chunks,
                &chaff_previous,
                content.to_string(),
                chaff.to_string(),
            )?;

            remainder = r;
            chaff_previous = chaff.to_string();
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Expected a Structured Text file but cannot find final `{CONTENT_END}`"),
        ))
    }
}

fn add_chunk(
    chunks: &mut Vec<Chunk>,
    chaff_before: &str,
    content: String,
    chaff_after: String,
) -> Result<()> {
    let what = if chaff_before.contains("<Declaration>") {
        Content::Declaration
    } else if chaff_before.contains("<Implementation>") {
        Content::Implementation
    } else {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot find `<Declaration>` or `<Implementation>` in {chaff_before}"),
        ));
    };

    chunks.push(Chunk {
        what,
        content,
        chaff: chaff_after,
    });

    Ok(())
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.chaff0)?;
        write!(f, "{}", self.declaration)?;
        write!(f, "{}", self.chaff1)?;
        for chunk in &self.chunks {
            write!(f, "{}{}", chunk.content, chunk.chaff)?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a File {
    type Item = (Content, &'a str);
    type IntoIter = FileIntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            file: self,
            index: 0,
        }
    }
}

pub struct FileIntoIter<'a> {
    file: &'a File,
    index: usize,
}

impl<'a> Iterator for FileIntoIter<'a> {
    type Item = (Content, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            self.index += 1;
            Some((Content::Declaration, &self.file.declaration))
        } else if self.index - 1 < self.file.chunks.len() {
            let i = self.index - 1;
            self.index += 1;
            Some((self.file.chunks[i].what, &self.file.chunks[i].content))
        } else {
            None
        }
    }
}

impl File {
    pub fn for_each_chunk(&mut self, cb: fn(&str) -> Result<String>) -> Result<()> {
        let declaration = cb(&self.declaration)?;
        self.declaration = declaration;
        for chunk in self.chunks.iter_mut() {
            let content = cb(&chunk.content)?;
            chunk.content = content;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "./test_structured_text.rs"]
mod test_structured_text;

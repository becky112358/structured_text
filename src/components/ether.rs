use std::io::{Error, ErrorKind, Result};

use crate::dazzle::{self, Dazzle, PreviousCharacter};

#[derive(Clone, Debug, PartialEq)]
pub enum Ether {
    LineFeed,
    PragmaOrComment(PragmaOrComment),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PragmaOrComment(PragmaOrCommentInner);

#[derive(Clone, Debug, PartialEq)]
enum PragmaOrCommentInner {
    Pragma(String),
    CommentSingleLine(String),
    CommentMultiLine(String),
}

impl Dazzle for Ether {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        match self {
            Self::LineFeed => {
                dazzler.f.push('\n');
                dazzler.previous_character = PreviousCharacter::LineFeed;
            }
            Self::PragmaOrComment(PragmaOrComment(PragmaOrCommentInner::Pragma(inner))) => {
                if dazzler.previous_character != PreviousCharacter::LineFeed {
                    dazzler.f.push('\n');
                }
                dazzler.f.push_str(&format!("{{{inner}}}"));
                dazzler.previous_character = PreviousCharacter::Other;
            }
            Self::PragmaOrComment(PragmaOrComment(PragmaOrCommentInner::CommentSingleLine(
                inner,
            ))) => {
                dazzler.indent_or_space(true);
                dazzler.f.push_str(&format!("// {inner}"));
                dazzler.previous_character = PreviousCharacter::LineFeed;
            }
            Self::PragmaOrComment(PragmaOrComment(PragmaOrCommentInner::CommentMultiLine(
                inner,
            ))) => {
                dazzler.indent_or_space(true);
                dazzler.f.push_str(&format!("(* {inner} *)"));
                dazzler.previous_character = PreviousCharacter::Other;
            }
        }
    }
}

impl Ether {
    pub fn peel(remainder: &mut String) -> Result<Vec<Self>> {
        let mut pragmas_and_comments = Vec::new();
        let mut remainder_clone = remainder.to_string();
        peel_new_line(&mut pragmas_and_comments, &mut remainder_clone);
        peel_new_line(&mut pragmas_and_comments, &mut remainder_clone);
        remainder_clone = remainder_clone.trim_start().to_string();
        while peel_single(&mut pragmas_and_comments, &mut remainder_clone)? {
            peel_new_line(&mut pragmas_and_comments, &mut remainder_clone);
            peel_new_line(&mut pragmas_and_comments, &mut remainder_clone);
            remainder_clone = remainder_clone.trim_start().to_string();
        }
        *remainder = remainder_clone;
        Ok(pragmas_and_comments)
    }

    pub fn then_comment(remainder: &str) -> bool {
        remainder.trim().starts_with("//") || remainder.trim().starts_with("(*")
    }
}

fn peel_new_line(pragmas_and_comments: &mut Vec<Ether>, remainder: &mut String) -> bool {
    match remainder.split_once('\n') {
        Some((before, after)) => {
            for c in before.chars() {
                if !c.is_ascii_whitespace() {
                    return false;
                }
            }
            pragmas_and_comments.push(Ether::LineFeed);
            *remainder = after.to_string();
            true
        }
        None => false,
    }
}

fn peel_single(pragmas_and_comments: &mut Vec<Ether>, remainder: &mut String) -> Result<bool> {
    if remainder.starts_with('{') {
        let pragma_end = match remainder.find('}') {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find pragma end \n{remainder}"),
                ))
            }
        };
        let pragma_inner = remainder[1..pragma_end].trim().to_string();
        if pragma_end == remainder.len() - 1 {
            *remainder = String::new();
        } else {
            *remainder = remainder[pragma_end + 1..].to_string();
        }

        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::Pragma(pragma_inner),
        )));
        return Ok(true);
    }

    if remainder.starts_with("(*") {
        let comment_end = match remainder.find("*)") {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot find comment end \n{remainder}"),
                ))
            }
        };
        let comment_inner = remainder["(*".len()..comment_end].trim().to_string();
        if comment_end == remainder.len() - "*)".len() {
            *remainder = String::new();
        } else {
            *remainder = remainder[comment_end + "*)".len()..].to_string();
        }

        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::CommentMultiLine(comment_inner),
        )));
        return Ok(true);
    }

    if remainder.starts_with("//") {
        let comment;
        match remainder.find('\n') {
            Some(i) => {
                comment = remainder["//".len()..i].trim().to_string();
                *remainder = remainder[i + 1..].to_string();
            }
            None => {
                comment = remainder["//".len()..].trim().to_string();
                *remainder = String::new();
            }
        };

        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::CommentSingleLine(comment),
        )));
        pragmas_and_comments.push(Ether::LineFeed);
        return Ok(true);
    }

    Ok(false)
}

impl Ether {
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            Self::PragmaOrComment(
                PragmaOrComment(PragmaOrCommentInner::CommentSingleLine(_))
                    | PragmaOrComment(PragmaOrCommentInner::CommentMultiLine(_))
            )
        )
    }
}

#[cfg(test)]
#[path = "./test_ether.rs"]
mod test_ether;

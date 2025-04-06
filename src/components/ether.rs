use std::io::Result;

use crate::code::Code;
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
                dazzler.if_not_linefeed_then_linefeed();
                dazzler.f.push_str(&format!("{{{inner}}}"));
                dazzler.previous_character = PreviousCharacter::Other;
            }
            Self::PragmaOrComment(PragmaOrComment(PragmaOrCommentInner::CommentSingleLine(
                inner,
            ))) => {
                dazzler.indent_or_space(true);
                dazzler.f.push_str("//");
                dazzler.previous_character = PreviousCharacter::PendingSpace;
                if !inner.is_empty() {
                    inner.dazzle(dazzler);
                }
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
    pub fn peel(code: &mut Code) -> Result<Vec<Self>> {
        let mut pragmas_and_comments = Vec::new();
        let mut code_clone = code.clone();
        peel_new_line(&mut pragmas_and_comments, &mut code_clone)?;
        peel_new_line(&mut pragmas_and_comments, &mut code_clone)?;
        code_clone = code_clone.trim_start();
        while peel_single(&mut pragmas_and_comments, &mut code_clone)? {
            peel_new_line(&mut pragmas_and_comments, &mut code_clone)?;
            peel_new_line(&mut pragmas_and_comments, &mut code_clone)?;
            code_clone = code_clone.trim_start();
        }
        *code = code_clone;
        Ok(pragmas_and_comments)
    }
}

fn peel_new_line(pragmas_and_comments: &mut Vec<Ether>, code: &mut Code) -> Result<()> {
    let mut whitespace_length = 0;
    for c in code.chars() {
        if c == '\n' {
            whitespace_length += '\n'.len_utf8();
            code.peel(whitespace_length)?;
            pragmas_and_comments.push(Ether::LineFeed);
            return Ok(());
        } else if c.is_ascii_whitespace() {
            whitespace_length += c.len_utf8();
        } else {
            return Ok(());
        }
    }
    Ok(())
}

fn peel_single(pragmas_and_comments: &mut Vec<Ether>, code: &mut Code) -> Result<bool> {
    if let Ok(pragma) = code.strip_between_nestable_and_trim_inner("{", "}") {
        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::Pragma(pragma),
        )));
        return Ok(true);
    }

    if let Ok(comment) = code.strip_between_nestable_and_trim_inner("(*", "*)") {
        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::CommentMultiLine(comment),
        )));
        return Ok(true);
    }

    if let Ok(comment) = code.strip_between_and_trim_inner("//", "\n") {
        pragmas_and_comments.push(Ether::PragmaOrComment(PragmaOrComment(
            PragmaOrCommentInner::CommentSingleLine(comment),
        )));
        pragmas_and_comments.push(Ether::LineFeed);
        return Ok(true);
    } else if let Ok(comment) = code.strip_from_and_trim_inner("//") {
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

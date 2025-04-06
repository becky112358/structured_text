use std::fmt;

pub const INDENT_WIDTH: u8 = 4;

pub trait Dazzle {
    fn dazzle(&self, dazzler: &mut Dazzler);
}

#[derive(Clone)]
pub struct Dazzler {
    pub f: String,
    pub previous_character: PreviousCharacter,
    pub indentation_count: u8,
}

#[derive(Clone, PartialEq)]
pub enum PreviousCharacter {
    Top,
    LineFeed,
    PendingSpace,
    Other,
}

impl Default for Dazzler {
    fn default() -> Self {
        Self {
            f: String::new(),
            previous_character: PreviousCharacter::Top,
            indentation_count: 0,
        }
    }
}

impl<D: fmt::Display> Dazzle for D {
    fn dazzle(&self, dazzler: &mut Dazzler) {
        dazzler.indent_or_space(false);
        dazzler.f.push_str(&self.to_string());
        dazzler.previous_character = PreviousCharacter::Other;
    }
}

impl Dazzler {
    pub fn indent_or_space(&mut self, finish_with_newline_or_space: bool) {
        match self.previous_character {
            PreviousCharacter::Top => (),
            PreviousCharacter::LineFeed => {
                self.indent();
            }
            PreviousCharacter::PendingSpace => {
                self.f.push(' ');
                self.previous_character = PreviousCharacter::Other;
            }
            PreviousCharacter::Other => {
                if finish_with_newline_or_space {
                    self.f.push(' ');
                }
            }
        }
    }

    pub fn indent(&mut self) {
        for _ in 0..self.indentation_count {
            self.f.push_str("    ");
        }
        self.previous_character = PreviousCharacter::Other;
    }

    pub fn if_not_linefeed_then_linefeed(&mut self) {
        match self.previous_character {
            PreviousCharacter::Top | PreviousCharacter::LineFeed => (),
            PreviousCharacter::PendingSpace | PreviousCharacter::Other => {
                self.f.push('\n');
                self.previous_character = PreviousCharacter::LineFeed;
            }
        }
    }

    pub fn should_split<T>(&self, t: &T, dazzle_singleline: fn(&T, &mut Dazzler)) -> bool {
        let last_line = match self.f.lines().last() {
            Some(line) => line.to_owned(),
            None => self.f.clone(),
        };
        let mut dazzler = Dazzler {
            f: last_line,
            previous_character: self.previous_character.clone(),
            indentation_count: self.indentation_count,
        };

        dazzle_singleline(t, &mut dazzler);

        dazzler.f.contains('\n') || dazzler.f.len() > crate::fmt::LINE_LENGTH_LIMIT as usize
    }
}

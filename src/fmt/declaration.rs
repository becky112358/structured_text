use std::io::Result;
use std::str::FromStr;

use crate::components::{BeginMiddleEnd, Component as C, Ether};
use crate::dazzle::{self, Dazzle, PreviousCharacter};
use crate::declaration::Declaration;

pub(super) fn align(input: &str) -> Result<String> {
    Ok(Declaration::from_str(input)?.make_pretty())
}

impl Declaration {
    fn make_pretty(&mut self) -> String {
        self.trim_line_feeds();

        let mut max_width = 0;
        self.extend_to_width(":", &mut max_width, false);
        self.extend_to_width(":", &mut max_width, true);

        let mut max_width = 0;
        self.extend_to_width(":=", &mut max_width, false);
        self.extend_to_width(":=", &mut max_width, true);

        let mut max_width = 0;
        self.extend_to_width("//", &mut max_width, false);
        self.extend_to_width("//", &mut max_width, true);

        let mut dazzler = dazzle::Dazzler::default();
        for component in &self.0 {
            component.dazzle(&mut dazzler);
        }
        dazzler.f
    }

    fn trim_line_feeds(&mut self) {
        trim_line_feeds(&mut self.0, false);
    }

    fn extend_to_width(&mut self, aligner: &str, width_max: &mut u8, found_max: bool) {
        let mut dazzler = dazzle::Dazzler::default();
        extend_to_width(
            &mut self.0,
            &mut dazzler,
            aligner,
            false,
            width_max,
            found_max,
        );
    }
}

fn trim_line_feeds(components: &mut Vec<C>, allow_multiple: bool) {
    let mut line_feed_count = 1;

    let mut i = 0;
    loop {
        if i >= components.len() {
            break;
        }

        match &mut components[i] {
            C::Ether(Ether::LineFeed) => {
                if i == 0
                    || (allow_multiple && line_feed_count >= 2)
                    || (!allow_multiple && line_feed_count >= 1)
                {
                    components.remove(i);
                } else {
                    line_feed_count += 1;
                    i += 1;
                }
            }
            C::Ether(_)
            | C::Space
            | C::Uppercase(_)
            | C::Text(_)
            | C::Identifier(_)
            | C::IdentifierSub(_)
            | C::Address(_)
            | C::DataType(_)
            | C::Value(_)
            | C::Expression(_)
            | C::Filler(_) => {
                line_feed_count = 0;
                i += 1;
            }
            C::Repeat(inners) => {
                trim_line_feeds(inners, false);
                i += 1;
            }
            C::BeginMiddleEnd(BeginMiddleEnd { begin, middle, end }) => {
                trim_line_feeds(begin, false);
                let n_middles = middle.len();
                for (im, m) in middle.iter_mut().enumerate() {
                    trim_line_feeds(m, im + 1 < n_middles);
                }
                trim_line_feeds(end, allow_multiple);
                i += 1;
            }
        }
    }
}

fn extend_to_width(
    components: &mut Vec<C>,
    dazzler: &mut dazzle::Dazzler,
    aligner: &str,
    inside_repeat: bool,
    width_max: &mut u8,
    found_max: bool,
) {
    let mut i = 0;
    loop {
        if i >= components.len() {
            break;
        }
        let next_is_line_feed =
            i + 1 < components.len() && matches!(components[i + 1], C::Ether(Ether::LineFeed));
        let component = &mut components[i];
        match component {
            C::Ether(inner) => {
                let mut insert = None;
                if inside_repeat
                    && *aligner == *"//"
                    && dazzler.previous_character != PreviousCharacter::LineFeed
                    && inner.is_comment()
                    && next_is_line_feed
                {
                    if let Some(last_line) = dazzler.f.lines().last() {
                        let width = last_line.len() as u8;
                        if !found_max {
                            if width > *width_max {
                                *width_max = width;
                            }
                        } else if width < *width_max {
                            let difference = *width_max - width;
                            insert = Some(difference);
                        }
                    }
                }
                component.dazzle(dazzler);
                if let Some(difference) = insert {
                    components.insert(i, C::Filler(difference));
                    i += 1;
                }
            }
            C::Space
            | C::Uppercase(_)
            | C::Identifier(_)
            | C::IdentifierSub(_)
            | C::DataType(_)
            | C::Value(_)
            | C::Expression(_) => component.dazzle(dazzler),
            C::Text(inner) => {
                let text = inner.to_string();
                component.dazzle(dazzler);
                if inside_repeat && *text == *aligner {
                    if let Some(last_line) = dazzler.f.lines().last() {
                        let width = last_line.len() as u8;
                        if !found_max {
                            if width > *width_max {
                                *width_max = width;
                            }
                        } else if width < *width_max {
                            let difference = *width_max - width;
                            if aligner.eq(":") && i >= 2 {
                                if let C::Address(_) = components[i - 2] {
                                    components.insert(i - 2, C::Filler(difference));
                                } else {
                                    components.insert(i, C::Filler(difference));
                                }
                            } else {
                                components.insert(i, C::Filler(difference));
                            }
                            i += 1;
                        }
                    }
                }
            }
            C::Address(_) => component.dazzle(dazzler),
            C::BeginMiddleEnd(BeginMiddleEnd { begin, middle, end }) => {
                for b in begin {
                    b.dazzle(dazzler);
                }
                for m in middle {
                    extend_to_width(m, dazzler, aligner, true, width_max, found_max);
                }
                for e in end {
                    e.dazzle(dazzler);
                }
            }
            C::Repeat(ref mut inner) => {
                extend_to_width(inner, dazzler, aligner, true, width_max, found_max);
            }
            C::Filler(n) => {
                for _ in 0..*n {
                    dazzler.f.push(' ');
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
#[path = "./test_declaration.rs"]
mod test_declaration;

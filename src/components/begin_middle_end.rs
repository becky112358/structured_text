use std::io::Result;

use crate::code::Code;
use crate::dazzle::{self, Dazzle};

use super::{Component as C, Ether};

#[derive(Debug)]
pub struct BeginMiddleEnd {
    pub begin: Vec<C>,
    pub middle: Vec<Vec<C>>,
    pub end: Vec<C>,
}

impl Dazzle for BeginMiddleEnd {
    fn dazzle(&self, dazzler: &mut dazzle::Dazzler) {
        for b in &self.begin {
            b.dazzle(dazzler);
        }
        dazzler.if_not_linefeed_then_linefeed();
        dazzler.indentation_count += 1;
        for mi in &self.middle {
            for m in mi {
                m.dazzle(dazzler);
            }
            dazzler.if_not_linefeed_then_linefeed();
        }
        dazzler.indentation_count -= 1;
        for e in &self.end {
            e.dazzle(dazzler);
        }
        dazzler.if_not_linefeed_then_linefeed();
    }
}

impl BeginMiddleEnd {
    pub fn peel(
        code: &mut Code,
        layout_begin: impl Fn(&mut Code) -> Result<Vec<C>>,
        layout_middle: impl Fn(&mut Code) -> Result<Vec<C>>,
        layout_end: impl Fn(&mut Code) -> Result<Vec<C>>,
    ) -> Result<(Self, Vec<Ether>)> {
        let mut code_clone = code.clone();

        let mut begin = layout_begin(&mut code_clone)?;

        let mut middle_start_ethers = Vec::new();
        let mut new_line = false;
        for ether in Ether::peel(&mut code_clone)? {
            if !new_line {
                begin.push(C::Ether(ether.clone()));
            } else {
                middle_start_ethers.push(C::Ether(ether.clone()));
            }
            if matches!(ether, Ether::LineFeed) {
                new_line = true;
            }
        }

        let mut middle = vec![middle_start_ethers];
        let mut code_clone_clone = code_clone.clone();
        while let Ok(mut items) = layout_middle(&mut code_clone_clone) {
            for ether in Ether::peel(&mut code_clone_clone)? {
                items.push(C::Ether(ether));
            }
            middle.push(items);
            code_clone = code_clone_clone.clone();
        }

        let mut end = layout_end(&mut code_clone)?;
        let mut output_after_ethers = Vec::new();
        let mut new_line = false;
        for ether in Ether::peel(&mut code_clone)? {
            if !new_line {
                end.push(C::Ether(ether.clone()));
            } else {
                output_after_ethers.push(ether.clone());
            }
            if matches!(ether, Ether::LineFeed) {
                new_line = true;
            }
        }

        *code = code_clone;

        Ok((Self { begin, middle, end }, output_after_ethers))
    }
}

use std::io::Result;
use std::str::FromStr;

use crate::dazzle::{self, Dazzle};
use crate::implementation::Implementation;

pub(super) fn align(input: &str) -> Result<String> {
    let implementation = Implementation::from_str(input)?;

    let mut dazzler = dazzle::Dazzler::default();
    for c in &implementation.0 {
        c.dazzle(&mut dazzler);
    }
    Ok(dazzler.f)
}

#[cfg(test)]
#[path = "./test_implementation.rs"]
mod test_implementation;

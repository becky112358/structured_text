use std::fs;
use std::io::Result;
use std::path::Path;
use std::str::FromStr;

use super::{structured_text, visit_dirs};

mod declaration;
mod implementation;
mod tab;
mod trailing_whitespace;

pub const LINE_LENGTH_LIMIT: u8 = 120;

pub fn fmt() -> Result<()> {
    visit_dirs(Path::new("."), fmt_file)?;
    Ok(())
}

fn fmt_file(path: &Path) -> Result<()> {
    let file = fs::read_to_string(path)?;

    let mut structured_text = structured_text::File::from_str(&file)?;

    structured_text.for_each_chunk(trailing_whitespace::trim_end)?;
    structured_text.for_each_chunk(tab::replace_with_whitespace)?;
    if structured_text
        .for_each_declaration(declaration::align)
        .is_err()
    {
        println!("Failed to format {path:?} (declaration)");
    }
    if structured_text
        .for_each_implementation(implementation::align)
        .is_err()
    {
        println!("Failed to format {path:?} (implementation)");
    }

    fs::write(path, structured_text.to_string())?;

    Ok(())
}

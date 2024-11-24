use std::fs;
use std::io::Result;
use std::path::Path;
use std::str::FromStr;

use super::{structured_text, visit_dirs};

mod trailing_whitespace;

pub fn fmt() -> Result<()> {
    visit_dirs(Path::new("."), fmt_file)?;
    Ok(())
}

fn fmt_file(path: &Path) -> Result<()> {
    let file = fs::read_to_string(path)?;

    let mut structured_text = structured_text::File::from_str(&file)?;

    structured_text.for_each_chunk(trailing_whitespace::trim)?;

    fs::write(path, structured_text.to_string())?;

    Ok(())
}

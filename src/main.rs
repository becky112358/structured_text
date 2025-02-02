use std::fs;
use std::io::Result;
use std::path::Path;

mod components;
mod dazzle;
mod declaration;
mod fmt;
mod layout;
mod structured_text;

fn main() {
    fmt::fmt().unwrap();
}

pub fn visit_dirs(dir: &Path, cb: fn(&Path) -> Result<()>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, cb)?;
        } else {
            let extension = match path.extension() {
                Some(os_str) => os_str,
                None => continue,
            };

            match extension.to_str() {
                Some("TcPOU") | Some("TcDUT") | Some("TcTLEO") | Some("TcGVL") => (),
                Some(_) | None => continue,
            }

            cb(&path)?;
        }
    }
    Ok(())
}

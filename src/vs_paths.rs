//! Internal module for code handling paths in the
//! VS installation.
use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

/// Constructs a verified object representing the path to the sub directory.
pub(crate) fn sub_directory(parent: &Path, dir: &str) -> std::io::Result<PathBuf> {
    let sub_dir = parent.join(dir);
    if !sub_dir.is_dir() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "{} does not contain the {} directory.",
                parent.to_string_lossy(),
                dir
            ),
        ));
    }
    Ok(sub_dir)
}

//! Atomic, targeted writes.
//!
//! Every write to a user's project goes through here: a temporary file in
//! the same directory, flushed, then renamed over the target
//! (`AGENTS.md` §7). Rename is atomic on every platform we ship, so an
//! agent, an editor or a crash never sees a half-written `book.toml` — it
//! sees either the old file or the new one.

use std::io::Write as _;
use std::path::Path;

use booker_core::{Error, Result};

/// What a save did. Callers report it, and the file watcher uses it to know
/// there is nothing to reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// The bytes on disk already matched. Nothing was touched — which is
    /// what keeps "opening a project changes nothing on disk" true.
    Unchanged,
    Wrote,
}

impl Written {
    pub fn changed(self) -> bool {
        matches!(self, Written::Wrote)
    }
}

/// The prefix and suffix of the temporary file every atomic write goes
/// through.
///
/// They are public because the file watcher has to recognise them: the
/// temporary file is created and renamed away inside the project folder,
/// so it produces change events like any other file, and a reader who was
/// told the book changed because of one would be told a lie. The two
/// places must agree, so the names live here, next to the code that makes
/// them (`is_write_temporary`).
pub const TEMP_PREFIX: &str = ".booker-";
pub const TEMP_SUFFIX: &str = ".tmp";

/// Whether this path is the temporary file of an atomic write of ours
/// rather than a file of the book.
pub fn is_write_temporary(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX))
}

/// Write `contents` to `path`, atomically, and only if it would change the
/// file.
pub fn write_if_changed(path: &Path, contents: &str) -> Result<Written> {
    if let Ok(existing) = std::fs::read_to_string(path) {
        if existing == contents {
            return Ok(Written::Unchanged);
        }
    }
    write_atomic(path, contents.as_bytes())?;
    Ok(Written::Wrote)
}

/// Write `contents` to `path` through a temporary file and a rename.
pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(directory).map_err(|error| Error::Project {
        path: directory.to_path_buf(),
        message: format!("could not create the folder: {error}"),
    })?;

    let mut file = tempfile::Builder::new()
        .prefix(TEMP_PREFIX)
        .suffix(TEMP_SUFFIX)
        .tempfile_in(directory)
        .map_err(|error| Error::Project {
            path: path.to_path_buf(),
            message: format!("could not create a temporary file next to it: {error}"),
        })?;
    file.write_all(contents).map_err(|error| Error::Project {
        path: path.to_path_buf(),
        message: format!("could not write: {error}"),
    })?;
    file.as_file().sync_all().map_err(|error| Error::Project {
        path: path.to_path_buf(),
        message: format!("could not flush to disk: {error}"),
    })?;
    file.persist(path).map_err(|error| Error::Project {
        path: path.to_path_buf(),
        message: format!("could not replace the file: {}", error.error),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writing_the_same_bytes_does_not_touch_the_file() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("book.toml");
        assert_eq!(
            write_if_changed(&path, "title = \"Mia\"\n").unwrap(),
            Written::Wrote
        );
        let before = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .expect("a modification time");

        assert_eq!(
            write_if_changed(&path, "title = \"Mia\"\n").unwrap(),
            Written::Unchanged
        );
        let after = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .expect("a modification time");
        assert_eq!(before, after, "an unchanged save must not rewrite the file");
    }

    #[test]
    fn nothing_is_left_behind_next_to_the_file() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("book.toml");
        write_if_changed(&path, "title = \"Mia\"\n").unwrap();
        write_if_changed(&path, "title = \"Mia P.\"\n").unwrap();
        let entries: Vec<_> = std::fs::read_dir(directory.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entries, vec!["book.toml".to_string()]);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "title = \"Mia P.\"\n"
        );
    }

    #[test]
    fn a_folder_that_does_not_exist_yet_is_created() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("content").join("01.md");
        write_atomic(&path, b"# One\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# One\n");
    }
}

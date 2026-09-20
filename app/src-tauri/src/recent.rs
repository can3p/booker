//! The folders this window has opened before.
//!
//! This is the one thing the application remembers that is not in a book,
//! which is why it is the one place a file is written outside a project.
//! It lives in the per-user configuration folder the platform gives us, as
//! a plain JSON array of paths.
//!
//! Everything here is best-effort on purpose: a list of recent folders is a
//! convenience, and no failure to read or write it may stop somebody from
//! opening a book. A corrupt file reads as an empty list.

use std::path::{Path, PathBuf};

/// How many folders to remember. Long enough to cover the books somebody is
/// actually working on, short enough that the menu stays readable.
const KEEP: usize = 10;

/// The file name inside the application's configuration folder.
const FILE: &str = "recent-projects.json";

/// Read the list, newest first. Anything unreadable is an empty list.
pub fn read(config_dir: &Path) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(config_dir.join(FILE)) else {
        return Vec::new();
    };
    let paths: Vec<PathBuf> = serde_json::from_str(&text).unwrap_or_default();
    // A folder that has since been deleted or renamed is not offered: a
    // menu entry that cannot work is worse than one that is not there.
    paths.into_iter().filter(|path| path.is_dir()).collect()
}

/// Put `root` at the front of the list and write it back.
///
/// Returns the list as it now stands, so a caller can hand it straight to
/// the UI without reading the file again.
pub fn remember(config_dir: &Path, root: &Path) -> Vec<PathBuf> {
    let mut paths = read(config_dir);
    paths.retain(|existing| existing != root);
    paths.insert(0, root.to_path_buf());
    paths.truncate(KEEP);

    // Best effort: if this cannot be written, the window still works, it
    // just forgets. Nothing here is worth interrupting somebody for.
    if std::fs::create_dir_all(config_dir).is_ok() {
        if let Ok(text) = serde_json::to_string_pretty(&paths) {
            let _ = booker_project::write_atomic(&config_dir.join(FILE), text.as_bytes());
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two real folders, since the list drops any that are not there.
    fn two_folders(dir: &Path) -> (PathBuf, PathBuf) {
        let first = dir.join("mia");
        let second = dir.join("the-moon-jar");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        (first, second)
    }

    #[test]
    fn remembers_the_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let (first, second) = two_folders(dir.path());

        remember(&config, &first);
        let list = remember(&config, &second);

        assert_eq!(list, vec![second, first]);
    }

    #[test]
    fn opening_the_same_folder_again_moves_it_to_the_front_rather_than_duplicating_it() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let (first, second) = two_folders(dir.path());

        remember(&config, &first);
        remember(&config, &second);
        let list = remember(&config, &first);

        assert_eq!(list, vec![first, second]);
    }

    #[test]
    fn forgets_a_folder_that_is_no_longer_there() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let (first, second) = two_folders(dir.path());

        remember(&config, &first);
        remember(&config, &second);
        std::fs::remove_dir_all(&first).unwrap();

        assert_eq!(read(&config), vec![second]);
    }

    #[test]
    fn a_corrupt_file_reads_as_an_empty_list_rather_than_failing() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(config.join(FILE), "{ this is not the file we wrote").unwrap();

        assert!(read(&config).is_empty());
    }

    #[test]
    fn keeps_only_the_last_ten() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");

        let mut last = Vec::new();
        for index in 0..KEEP + 5 {
            let folder = dir.path().join(format!("book-{index}"));
            std::fs::create_dir_all(&folder).unwrap();
            last = remember(&config, &folder);
        }

        assert_eq!(last.len(), KEEP);
        assert!(last[0].ends_with(format!("book-{}", KEEP + 4)));
    }
}

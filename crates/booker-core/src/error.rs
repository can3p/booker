use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that cross a crate boundary.
///
/// A user's project content must never produce a panic (`AGENTS.md` §6), so
/// anything that can be wrong with a project is a variant here, and carries
/// enough to tell the user which file it came from.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{path}: {message}")]
    Project { path: PathBuf, message: String },

    #[error("{path}: could not parse: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("project format version {found} is newer than this build understands ({supported})")]
    FormatTooNew { found: u32, supported: u32 },

    #[error("no page {requested}; the document has {total}")]
    NoSuchPage { requested: u32, total: u32 },

    #[error("layout failed: {0}")]
    Layout(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

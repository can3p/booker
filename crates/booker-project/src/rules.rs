//! The diagnostic rules this crate can raise.
//!
//! IDs are stable for years (`AGENTS.md` §6): they end up in `[check]`
//! configuration, in ignore comments, and in what a person or an agent says
//! to each other about a book. Add, never renumber, never reuse.
//!
//! | Rule | Severity | Means |
//! |---|---|---|
//! | `BK-FORMAT-001` | warning | no `book.toml`; defaults were used |
//! | `BK-FORMAT-002` | error | `book.toml` is not valid TOML |
//! | `BK-FORMAT-003` | error | the format version is newer than this build |
//! | `BK-FORMAT-004` | error | a value could not be understood |
//! | `BK-FORMAT-005` | warning | an unknown key, kept unchanged |
//! | `BK-REF-001` | error | an image file is missing |
//! | `BK-REF-002` | error | a chapter listed in `book.toml` is absent |
//! | `BK-REF-003` | warning | the project has no content at all |
//! | `BK-REF-004` | error | a chapter file exists but could not be read |
//! | `BK-FORMAT-006` | error | `theme` names no built-in theme |
//! | `BK-REF-005` | error | a link to `#id` where nothing has that id |
//! | `BK-REF-006` | error | two elements share an id |
//! | `BK-DOC-001` | warning | an attribute key nothing reads |
//! | `BK-DOC-002` | warning | Markdown kept but not laid out yet |
//! | `BK-TEXT-001` | warning | a chapter renders nothing |
//!
//! The last six arrived with Wave 2 (`docs/waves/wave-2.md`, contract 6).
//! The document rules live here rather than in `booker-doc` because a
//! diagnostic needs a file, and the parser only ever sees text.

/// `book.toml` is missing. A folder with one `book.md` in it is still a
/// book (`PLAN.md` §5.1), so this is a warning and defaults are used.
pub const NO_BOOK_TOML: &str = "BK-FORMAT-001";
/// `book.toml` could not be parsed as TOML.
pub const INVALID_TOML: &str = "BK-FORMAT-002";
/// The project was written by a newer Booker than this one.
pub const FORMAT_TOO_NEW: &str = "BK-FORMAT-003";
/// A value is of the wrong kind, or cannot be understood.
pub const BAD_VALUE: &str = "BK-FORMAT-004";
/// A key this build does not know. It is preserved unchanged.
pub const UNKNOWN_KEY: &str = "BK-FORMAT-005";

/// An image referenced by a chapter is not on disk.
pub const MISSING_IMAGE: &str = "BK-REF-001";
/// A chapter listed in `book.toml` is not on disk.
pub const MISSING_CHAPTER: &str = "BK-REF-002";
/// The project contains no Markdown at all.
pub const NO_CONTENT: &str = "BK-REF-003";
/// A chapter file is there but could not be read.
pub const UNREADABLE_CHAPTER: &str = "BK-REF-004";

/// `theme` names a theme that is not built in. The default is used.
pub const UNKNOWN_THEME: &str = "BK-FORMAT-006";
/// A link points at `#id`, and no heading, div or span has that id.
pub const UNRESOLVED_LINK: &str = "BK-REF-005";
/// Two elements in the book have the same id, so a link to it is ambiguous.
pub const DUPLICATE_ID: &str = "BK-REF-006";
/// An attribute key that nothing in this build reads. It is kept.
pub const UNKNOWN_ATTRIBUTE: &str = "BK-DOC-001";
/// Markdown this build keeps but cannot lay out yet: footnotes, mathematics,
/// raw HTML. Until Wave 2 it vanished from the PDF without a word.
pub const NOT_LAID_OUT: &str = "BK-DOC-002";
/// A chapter file that produces nothing on the page.
pub const EMPTY_CHAPTER: &str = "BK-TEXT-001";

//! Markdown, plus Booker's attributes and directives, to the document model.
//!
//! Owned by Wave 0 track E. Every node must carry the byte range it came
//! from: click-to-source, and every diagnostic that will ever point at a
//! line, depend on it.

pub use booker_core::{Diagnostic, Error, Result, SourceLocation};

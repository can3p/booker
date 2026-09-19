//! Markdown, plus Booker's attributes and directives, to the document model.
//!
//! Owned by Wave 0 track E. Every node must carry the byte range it came
//! from: click-to-source, and every diagnostic that will ever point at a
//! line, depend on it.
//!
//! ```
//! use booker_doc::{Block, Document};
//!
//! let source = "# The Garden\n\nMia opened the *little* green door.\n";
//! let document = Document::parse(source);
//! let heading = document.headings()[0];
//! assert_eq!(heading.span.slice(source), Some("# The Garden\n"));
//!
//! let location = document.location(source, "content/01.md", heading.span);
//! assert_eq!((location.line, location.column), (1, 1));
//! assert!(matches!(document.blocks[1], Block::Paragraph(_)));
//! ```

mod model;
mod parse;
mod span;

pub use booker_core::{Diagnostic, Error, Result, SourceLocation};

pub use model::{
    Attributes, Block, CodeBlock, Document, Heading, Image, Inline, Link, List, ListItem, Node,
    Paragraph, Quote,
};
pub use span::{LineIndex, Span};

impl Document {
    /// Parse a Markdown file. Infallible: a strange document is still a
    /// document (`AGENTS.md` §7), and anything this build does not model is
    /// kept as `Block::Unsupported` with its span rather than dropped.
    pub fn parse(source: &str) -> Document {
        parse::parse(source)
    }

    /// The diagnostic location of a span in this document.
    ///
    /// `file` is the path as it should appear to the user — relative to the
    /// project root, so output is identical on every machine
    /// (`PLAN.md` §11.5). Callers that need many locations from one file
    /// should build a [`LineIndex`] once and reuse it.
    pub fn location(
        &self,
        source: &str,
        file: impl AsRef<std::path::Path>,
        span: Span,
    ) -> SourceLocation {
        LineIndex::new(source).source_location(source, file, span)
    }
}

// Deliberate breakage, on a throwaway branch: this proves the fmt job and the
// clippy job are wired to something. Never merged.
#[allow(dead_code)]
fn   badly_formatted_on_purpose( ) ->   u8 { 1 }

#[allow(dead_code)]
fn clippy_bait(v: &Vec<u8>) -> usize {
    v.len()
}

//! Byte ranges, and turning one into the file/line/column a person reads.

use std::path::{Path, PathBuf};

use booker_core::SourceLocation;

/// A byte range in the file a node was parsed from.
///
/// Byte offsets rather than line/column, because that is what the parser
/// produces and what an edit needs; `LineIndex` turns one into the pair a
/// person (or an editor) wants. Ranges are half-open: `start..end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    /// The source text this node came from, or `None` if the span does not
    /// land on character boundaries of this string — which would mean the
    /// span and the text do not belong together.
    pub fn slice<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.range())
    }

    /// The smallest span covering both.
    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

/// Line starts of a file, so a byte offset becomes a line and column.
///
/// Built once per file and kept with the parsed document: every diagnostic
/// that will ever point into this file needs it, and so does click-to-source
/// (`PLAN.md` §11.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    /// Byte offset of the first character of each line. Always starts with 0.
    starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(offset + 1);
            }
        }
        Self {
            starts,
            len: source.len(),
        }
    }

    /// 1-based line and column, counted in characters as editors count them.
    /// A `\r\n` line ending is not part of the line, so a column never lands
    /// on the carriage return.
    pub fn line_column(&self, source: &str, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.len);
        let line = match self.starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        let start = self.starts[line];
        // Count characters, not bytes: an accented letter is one column.
        let column = source
            .get(start..offset)
            .map(|text| text.chars().count())
            .unwrap_or(0);
        (line as u32 + 1, column as u32 + 1)
    }

    pub fn line_count(&self) -> usize {
        self.starts.len()
    }

    /// The diagnostic location for a span: the file as the user will see it
    /// (project-relative, `PLAN.md` §11.5), the line and column of its start,
    /// and the range itself so a tool can select exactly the right text.
    pub fn source_location(
        &self,
        source: &str,
        file: impl AsRef<Path>,
        span: Span,
    ) -> SourceLocation {
        let (line, column) = self.line_column(source, span.start);
        SourceLocation {
            file: PathBuf::from(file.as_ref()),
            line,
            column,
            span: Some((span.start, span.end)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_lines_from_one() {
        let source = "alpha\nbeta\ngamma";
        let index = LineIndex::new(source);
        assert_eq!(index.line_column(source, 0), (1, 1));
        assert_eq!(index.line_column(source, 5), (1, 6));
        assert_eq!(index.line_column(source, 6), (2, 1));
        assert_eq!(index.line_column(source, 11), (3, 1));
        assert_eq!(index.line_count(), 3);
    }

    #[test]
    fn columns_are_characters_not_bytes() {
        // "Mia's grandmère" — the accented letter must not shift the column
        // by two, or every diagnostic after it points at the wrong place.
        let source = "grandmère was here";
        let index = LineIndex::new(source);
        let offset = source.find("was").unwrap();
        assert_eq!(offset, 11, "the accented letter is two bytes");
        assert_eq!(index.line_column(source, offset), (1, 11));
    }

    #[test]
    fn handles_windows_line_endings() {
        let source = "alpha\r\nbeta";
        let index = LineIndex::new(source);
        assert_eq!(
            index.line_column(source, source.find("beta").unwrap()),
            (2, 1)
        );
    }

    #[test]
    fn an_offset_past_the_end_still_answers() {
        let source = "alpha\n";
        let index = LineIndex::new(source);
        assert_eq!(index.line_column(source, 9_999), (2, 1));
    }

    #[test]
    fn a_span_slices_back_to_its_own_text() {
        let source = "# The Garden\n";
        let span = Span::new(2, 12);
        assert_eq!(span.slice(source), Some("The Garden"));
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn a_source_location_carries_the_range_as_well_as_the_line() {
        let source = "one\ntwo\nthree";
        let index = LineIndex::new(source);
        let location = index.source_location(source, "content/01.md", Span::new(8, 13));
        assert_eq!(location.line, 3);
        assert_eq!(location.column, 1);
        assert_eq!(location.span, Some((8, 13)));
        assert_eq!(location.file.to_str(), Some("content/01.md"));
    }
}

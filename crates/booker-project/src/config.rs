//! `book.toml`: read leniently, written back with `toml_edit`.
//!
//! Two obligations meet in this file (`AGENTS.md` §7):
//!
//! * **A broken project still opens.** Nothing here returns an error for bad
//!   content. A value that cannot be understood becomes a diagnostic with a
//!   file, line and column, and the default is used in its place, so the
//!   rest of the book still loads.
//! * **Nothing the user wrote is lost.** The file's own text is kept and is
//!   what gets written back; comments, key order and keys this build has
//!   never heard of come back byte for byte.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use booker_core::diagnostic::Severity;
use booker_core::geometry::{Length, Margins, PageSize, Preset};
use booker_core::{BookConfig, Diagnostic, PageConfig, SourceLocation, FORMAT_VERSION};
use booker_doc::{LineIndex, Span};
use toml_edit::{DocumentMut, ImDocument};

use crate::rules;
use crate::toml_tree::{closest, to_toml_value, Node};

/// The name of the project file, everywhere.
pub const BOOK_TOML: &str = "book.toml";

const TOP_LEVEL_KEYS: &[&str] = &["format", "title", "author", "language", "chapters", "page"];
const PAGE_KEYS: &[&str] = &["size", "margins", "facing", "bleed"];
const MARGIN_KEYS: &[&str] = &["top", "bottom", "inside", "outside"];

/// Keys the plan has already named but this build does not implement yet
/// (`PLAN.md` §5.2). They are preserved like any unknown key, but telling a
/// user their `[toc]` section is a mystery would be a lie: we know what it
/// is, it simply arrives in a later wave.
const PLANNED_KEYS: &[&str] = &["theme", "toc", "chapter", "output", "check", "styles"];

/// `book.toml`, held twice on purpose.
///
/// `toml_edit` records the byte span of every key and value while parsing —
/// and throws all of them away the moment the document becomes editable
/// (`ImDocument::into_mut`; see `docs/FINDINGS.md`). Diagnostics need the
/// spans and writing needs the editable document, so this keeps both, parsed
/// from the same text:
///
/// * `spanned` — read-only, with spans. Everything that reports a location
///   reads from here.
/// * `document` — editable. Every write goes through here, and it is what
///   gets rendered back to disk.
///
/// `disk_text` is exactly what was read, so "did this change?" is a byte
/// comparison — which is how "opening a project changes nothing on disk"
/// stays true.
pub struct ConfigFile {
    path: PathBuf,
    relative: PathBuf,
    disk_text: String,
    exists: bool,
    parsed: bool,
    spanned: ImDocument<String>,
    document: DocumentMut,
    lines: LineIndex,
}

impl ConfigFile {
    /// Read `book.toml` from a project root. Never fails: a missing or
    /// unparseable file is a diagnostic, not a refusal.
    pub fn load(root: &Path, diagnostics: &mut Vec<Diagnostic>) -> ConfigFile {
        let path = root.join(BOOK_TOML);
        let relative = PathBuf::from(BOOK_TOML);
        let (text, exists) = match std::fs::read_to_string(&path) {
            Ok(text) => (text, true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                diagnostics.push(
                    Diagnostic::error(
                        rules::NO_BOOK_TOML,
                        format!(
                            "no `{BOOK_TOML}` in this folder; \
                             Booker is using the default settings"
                        ),
                    )
                    .with_severity(Severity::Warning)
                    .at(SourceLocation {
                        file: relative.clone(),
                        line: 1,
                        column: 1,
                        span: None,
                    }),
                );
                (String::new(), false)
            }
            Err(error) => {
                diagnostics.push(
                    Diagnostic::error(
                        rules::INVALID_TOML,
                        format!("could not read `{BOOK_TOML}`: {error}"),
                    )
                    .at(SourceLocation {
                        file: relative.clone(),
                        line: 1,
                        column: 1,
                        span: None,
                    }),
                );
                (String::new(), true)
            }
        };

        let text_lines = LineIndex::new(&text);
        let (spanned, parsed) = match text.parse::<ImDocument<String>>() {
            Ok(document) => (document, true),
            Err(error) => {
                let span = error
                    .span()
                    .map(Span::from)
                    .unwrap_or_else(|| Span::new(0, text.len().min(1)));
                let location = text_lines.source_location(&text, &relative, span);
                diagnostics.push(
                    Diagnostic::error(
                        rules::INVALID_TOML,
                        format!("`{BOOK_TOML}` is not valid TOML: {}", error.message()),
                    )
                    .at(location),
                );
                // An unparseable file still opens: it loads as an empty
                // document, the diagnostic above says where the syntax error
                // is, and `Project::save` refuses to write over it.
                (
                    ImDocument::parse(String::new()).expect("an empty document is valid TOML"),
                    false,
                )
            }
        };
        let document = text.parse::<DocumentMut>().unwrap_or_default();
        let lines = LineIndex::new(spanned.raw());

        ConfigFile {
            path,
            relative,
            disk_text: text,
            exists,
            parsed,
            spanned,
            document,
            lines,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path as it appears in messages: project-relative, so output is
    /// the same on every machine (`PLAN.md` §11.5).
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }

    /// The file as it was read. Empty when there is no `book.toml`.
    pub fn text(&self) -> &str {
        &self.disk_text
    }

    pub fn exists(&self) -> bool {
        self.exists
    }

    /// Whether the file parsed. A file that did not must never be written
    /// back: that would replace what the user wrote with our empty document.
    pub fn is_parsed(&self) -> bool {
        self.parsed
    }

    /// What the document would be written as right now.
    pub fn rendered(&self) -> String {
        self.document.to_string()
    }

    /// Whether the document has drifted from the text on disk.
    pub fn is_dirty(&self) -> bool {
        self.rendered() != self.disk_text
    }

    pub(crate) fn document_mut(&mut self) -> &mut DocumentMut {
        &mut self.document
    }

    /// Re-read the spans from the edited document, so that a diagnostic
    /// raised after an edit points at where the value is *now*.
    pub(crate) fn refresh_spans(&mut self) {
        let rendered = self.rendered();
        self.lines = LineIndex::new(&rendered);
        if let Ok(spanned) = ImDocument::parse(rendered) {
            self.spanned = spanned;
        }
    }

    pub(crate) fn mark_written(&mut self, text: String) {
        self.disk_text = text;
        self.exists = true;
        self.refresh_spans();
    }

    fn location(&self, span: Option<std::ops::Range<usize>>) -> SourceLocation {
        match span {
            Some(range) => {
                self.lines
                    .source_location(self.spanned.raw(), &self.relative, Span::from(range))
            }
            None => SourceLocation {
                file: self.relative.clone(),
                line: 1,
                column: 1,
                span: None,
            },
        }
    }

    /// Where the *n*th entry of `chapters` was written, so "this chapter is
    /// missing" points at the line that lists it rather than at the file
    /// that is not there.
    pub fn chapter_location(&self, index: usize) -> SourceLocation {
        let span = Node::Item(self.spanned.as_item())
            .get("chapters")
            .and_then(Node::as_array)
            .and_then(|array| array.get(index))
            .and_then(toml_edit::Value::span);
        self.location(span)
    }

    /// The typed view of the file, plus everything wrong with it.
    ///
    /// Each problem costs one value, never the whole file: a book with a
    /// nonsense page size still opens, with the default page size and a
    /// diagnostic pointing at the line.
    pub fn typed(&self) -> (BookConfig, Vec<Diagnostic>) {
        let mut diagnostics = Vec::new();
        let root = Node::Item(self.spanned.as_item());

        let format = self
            .read_integer(root, "format", &mut diagnostics)
            .map(|value| value.clamp(0, u32::MAX as i64) as u32)
            .unwrap_or(FORMAT_VERSION);

        let title = self
            .read_string(root, "title", &mut diagnostics)
            .unwrap_or_default();
        let author = self.read_string(root, "author", &mut diagnostics);
        let language = self
            .read_string(root, "language", &mut diagnostics)
            .unwrap_or_else(|| "en".to_string());
        let chapters = self.read_chapters(root, &mut diagnostics);
        let page = self.read_page(root, &mut diagnostics);
        let extra = self.read_extra(root, &mut diagnostics);

        let config = BookConfig {
            format,
            title,
            author,
            language,
            chapters,
            page,
            extra,
        };

        if !config.is_supported() {
            diagnostics.push(
                Diagnostic::error(
                    rules::FORMAT_TOO_NEW,
                    format!(
                        "this project is format {} and this build of Booker understands \
                         format {FORMAT_VERSION}; some of it may not be shown correctly, \
                         and nothing it does not understand will be changed",
                        config.format
                    ),
                )
                .at(self.location(root.get("format").and_then(Node::span))),
            );
        }

        (config, diagnostics)
    }

    fn bad_value(
        &self,
        node: Node<'_>,
        key: &str,
        expected: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        diagnostics.push(
            Diagnostic::error(
                rules::BAD_VALUE,
                format!(
                    "`{key}` should be {expected}, but it is {}; the default is being used \
                     instead",
                    node.type_name()
                ),
            )
            .at(self.location(node.span())),
        );
    }

    fn read_string(
        &self,
        parent: Node<'_>,
        key: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<String> {
        let node = parent.get(key)?;
        match node.as_str() {
            Some(value) => Some(value.to_string()),
            None => {
                self.bad_value(node, key, "text in quotes", diagnostics);
                None
            }
        }
    }

    fn read_integer(
        &self,
        parent: Node<'_>,
        key: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<i64> {
        let node = parent.get(key)?;
        match node.as_integer() {
            Some(value) => Some(value),
            None => {
                self.bad_value(node, key, "a whole number", diagnostics);
                None
            }
        }
    }

    fn read_bool(
        &self,
        parent: Node<'_>,
        key: &str,
        path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<bool> {
        let node = parent.get(key)?;
        match node.as_bool() {
            Some(value) => Some(value),
            None => {
                self.bad_value(node, path, "true or false", diagnostics);
                None
            }
        }
    }

    fn read_length(
        &self,
        parent: Node<'_>,
        key: &str,
        path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<Length> {
        let node = parent.get(key)?;
        let Some(text) = node.as_str() else {
            self.bad_value(node, path, "a measurement such as `18mm`", diagnostics);
            return None;
        };
        match Length::from_str(text) {
            Ok(length) => Some(length),
            Err(message) => {
                diagnostics.push(
                    Diagnostic::error(
                        rules::BAD_VALUE,
                        format!("`{path}` is not a measurement: {message}"),
                    )
                    .at(self.location(node.span())),
                );
                None
            }
        }
    }

    fn read_chapters(&self, root: Node<'_>, diagnostics: &mut Vec<Diagnostic>) -> Vec<PathBuf> {
        let Some(node) = root.get("chapters") else {
            return Vec::new();
        };
        let Some(array) = node.as_array() else {
            self.bad_value(
                node,
                "chapters",
                "a list of file names, such as `[\"content/01.md\"]`",
                diagnostics,
            );
            return Vec::new();
        };
        let mut chapters = Vec::new();
        for entry in array.iter() {
            match entry.as_str() {
                Some(text) => chapters.push(PathBuf::from(text)),
                None => diagnostics.push(
                    Diagnostic::error(
                        rules::BAD_VALUE,
                        "every entry in `chapters` should be a file name in quotes".to_string(),
                    )
                    .at(self.location(entry.span())),
                ),
            }
        }
        chapters
    }

    fn read_page(&self, root: Node<'_>, diagnostics: &mut Vec<Diagnostic>) -> PageConfig {
        let mut page = PageConfig::default();
        let Some(node) = root.get("page") else {
            return page;
        };
        if !node.is_table() {
            self.bad_value(node, "page", "a table, written `[page]`", diagnostics);
            return page;
        }

        if let Some(size) = node.get("size") {
            match self.read_page_size(size) {
                Ok(value) => page.size = value,
                Err(message) => diagnostics.push(
                    Diagnostic::error(rules::BAD_VALUE, message).at(self.location(size.span())),
                ),
            }
        }
        if let Some(facing) = self.read_bool(node, "facing", "page.facing", diagnostics) {
            page.facing = facing;
        }
        if node.get("bleed").is_some() {
            page.bleed = self.read_length(node, "bleed", "page.bleed", diagnostics);
        }
        if let Some(margins) = node.get("margins") {
            if margins.is_table() {
                let mut value = Margins::default();
                let read = |key: &str, diagnostics: &mut Vec<Diagnostic>| {
                    self.read_length(margins, key, &format!("page.margins.{key}"), diagnostics)
                };
                if let Some(length) = read("top", diagnostics) {
                    value.top = length;
                }
                if let Some(length) = read("bottom", diagnostics) {
                    value.bottom = length;
                }
                if let Some(length) = read("inside", diagnostics) {
                    value.inside = length;
                }
                if let Some(length) = read("outside", diagnostics) {
                    value.outside = length;
                }
                page.margins = value;
                self.check_keys(margins, MARGIN_KEYS, "page.margins", diagnostics);
            } else {
                self.bad_value(
                    margins,
                    "page.margins",
                    "a table of measurements",
                    diagnostics,
                );
            }
        }

        self.check_keys(node, PAGE_KEYS, "page", diagnostics);
        page
    }

    fn read_page_size(&self, node: Node<'_>) -> Result<PageSize, String> {
        if node.is_table() {
            let width = node
                .get("width")
                .and_then(Node::as_str)
                .ok_or_else(|| "`page.size` needs a `width`, such as `\"148mm\"`".to_string())?;
            let height = node
                .get("height")
                .and_then(Node::as_str)
                .ok_or_else(|| "`page.size` needs a `height`, such as `\"210mm\"`".to_string())?;
            return Ok(PageSize::Custom {
                width: Length::from_str(width).map_err(|error| format!("`width`: {error}"))?,
                height: Length::from_str(height).map_err(|error| format!("`height`: {error}"))?,
            });
        }
        let Some(text) = node.as_str() else {
            return Err(format!(
                "`page.size` should be a preset name such as `\"a5\"`, or a size such as \
                 `\"8.5x8.5in\"`, but it is {}",
                node.type_name()
            ));
        };
        parse_page_size(text)
    }

    /// Unknown keys are kept, and said out loud (`AGENTS.md` §7).
    fn check_keys(
        &self,
        table: Node<'_>,
        known: &[&str],
        path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for key in table.keys() {
            if known.contains(&key) {
                continue;
            }
            let full = if path.is_empty() {
                key.to_string()
            } else {
                format!("{path}.{key}")
            };
            let planned = path.is_empty() && PLANNED_KEYS.contains(&key);
            let mut message = if planned {
                format!(
                    "`{full}` is part of the project format but is not implemented in this \
                     build; it is being kept unchanged"
                )
            } else {
                format!("unknown key `{full}`; it is being kept unchanged")
            };
            if !planned {
                if let Some(suggestion) = closest(key, known) {
                    let suggested = if path.is_empty() {
                        suggestion.to_string()
                    } else {
                        format!("{path}.{suggestion}")
                    };
                    message.push_str(&format!(" — did you mean `{suggested}`?"));
                }
            }
            diagnostics.push(
                Diagnostic::error(rules::UNKNOWN_KEY, message)
                    .with_severity(if planned {
                        Severity::Info
                    } else {
                        Severity::Warning
                    })
                    .at(self.location(
                        table
                            .key_span(key)
                            .or_else(|| table.get(key).and_then(Node::span)),
                    )),
            );
        }
    }

    fn read_extra(
        &self,
        root: Node<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> BTreeMap<String, toml::Value> {
        self.check_keys(root, TOP_LEVEL_KEYS, "", diagnostics);
        let mut extra = BTreeMap::new();
        for key in root.keys() {
            if TOP_LEVEL_KEYS.contains(&key) {
                continue;
            }
            if let Some(value) = root.get(key).and_then(to_toml_value) {
                extra.insert(key.to_string(), value);
            }
        }
        extra
    }
}

/// `"a5"`, `"8.5x8.5in"`, `"210mm x 297mm"`.
///
/// The presets are the names `PageSize` already understands; the `WxH` form
/// is what `PLAN.md` §5.2 shows an author writing, and it lands in
/// `PageSize::Custom` without the contract needing to change.
fn parse_page_size(text: &str) -> Result<PageSize, String> {
    let trimmed = text.trim();
    let preset = match trimmed.to_ascii_lowercase().as_str() {
        "a4" => Some(Preset::A4),
        "a5" => Some(Preset::A5),
        "letter" => Some(Preset::Letter),
        "trade" | "6x9in" => Some(Preset::Trade),
        "digest" | "5x8in" => Some(Preset::Digest),
        "square" => Some(Preset::Square),
        _ => None,
    };
    if let Some(preset) = preset {
        return Ok(PageSize::Named(preset));
    }

    if let Some((left, right)) = trimmed.split_once(['x', 'X', '×']) {
        let right = right.trim();
        let left = left.trim();
        // `8.5x8.5in`: the unit is written once, at the end.
        let unit: String = right
            .chars()
            .skip_while(|c| c.is_ascii_digit() || *c == '.' || c.is_whitespace())
            .collect();
        let left = if left.chars().any(|c| c.is_ascii_alphabetic()) {
            left.to_string()
        } else {
            format!("{left}{unit}")
        };
        let width = Length::from_str(&left).map_err(|error| format!("`page.size`: {error}"))?;
        let height = Length::from_str(right).map_err(|error| format!("`page.size`: {error}"))?;
        return Ok(PageSize::Custom { width, height });
    }

    Err(format!(
        "`page.size`: `{trimmed}` is not a page size; write a preset — a4, a5, letter, trade, \
         digest, square — or a size such as `8.5x8.5in`"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_sizes_people_write() {
        assert_eq!(parse_page_size("a5"), Ok(PageSize::Named(Preset::A5)));
        assert_eq!(parse_page_size("A4"), Ok(PageSize::Named(Preset::A4)));
        let square = parse_page_size("8.5x8.5in").expect("a size");
        let (width, height) = square.dimensions();
        assert_eq!(width.to_string(), "8.5in");
        assert_eq!(height.to_string(), "8.5in");
        let mixed = parse_page_size("148mm x 210mm").expect("a size");
        assert_eq!(mixed.dimensions().0.to_string(), "148mm");
    }

    #[test]
    fn a_page_size_that_is_not_one_explains_itself() {
        let error = parse_page_size("enormous").unwrap_err();
        assert!(error.contains("a4"), "{error}");
        assert!(error.contains("8.5x8.5in"), "{error}");
    }
}

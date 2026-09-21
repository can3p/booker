//! The chapters: finding them, reading them, and checking what they point at.

use std::path::{Path, PathBuf};

use booker_core::diagnostic::Severity;
use booker_core::{BookConfig, Diagnostic, SourceLocation};
use booker_doc::{Attributes, Block, Document, Inline, LineIndex, Node, Span, ATTRIBUTE_KEYS};

use crate::config::{ConfigFile, BOOK_TOML};
use crate::rules;
use crate::toml_tree::closest;

/// A chapter file, parsed, with everything needed to point back into it.
pub struct Chapter {
    path: PathBuf,
    relative: PathBuf,
    source: String,
    document: Document,
    lines: LineIndex,
}

impl Chapter {
    /// Where the file is on this machine.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Where it is in the project — the form that goes in messages.
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    /// The diagnostic location of a span in this chapter.
    pub fn location(&self, span: Span) -> SourceLocation {
        self.lines
            .source_location(&self.source, &self.relative, span)
    }

    /// The chapter's title: the text of its first heading, if it has one.
    pub fn title(&self) -> Option<String> {
        self.document.headings().first().map(|heading| {
            heading
                .inlines
                .iter()
                .map(Inline::text)
                .collect::<String>()
                .trim()
                .to_string()
        })
    }

    pub fn word_count(&self) -> usize {
        self.document.word_count()
    }
}

/// Which files make up the book, in order.
///
/// Explicit `chapters` in `book.toml` win. Otherwise `content/*.md` sorted
/// by name, and failing that a single `book.md` in the root — a folder with
/// one file in it is a book (`PLAN.md` §5.1).
pub(crate) fn discover(
    root: &Path,
    config: &BookConfig,
    config_file: &ConfigFile,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Chapter> {
    let mut chapters = Vec::new();

    if !config.chapters.is_empty() {
        for (index, relative) in config.chapters.iter().enumerate() {
            let location = config_file.chapter_location(index);
            match read_chapter(root, relative) {
                Ok(chapter) => chapters.push(chapter),
                Err(problem) => diagnostics.push(problem.into_diagnostic(relative, Some(location))),
            }
        }
        return chapters;
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    let content = root.join("content");
    if let Ok(entries) = std::fs::read_dir(&content) {
        let mut found: Vec<PathBuf> = entries
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
            .collect();
        found.sort();
        candidates.extend(
            found
                .iter()
                .map(|path| PathBuf::from("content").join(path.file_name().unwrap_or_default())),
        );
    }
    if candidates.is_empty() && root.join("book.md").is_file() {
        candidates.push(PathBuf::from("book.md"));
    }

    if candidates.is_empty() {
        diagnostics.push(
            Diagnostic::error(
                rules::NO_CONTENT,
                format!(
                    "this project has no text in it yet; add a `content/*.md` file, or a \
                     `book.md`, or list your chapters in `{BOOK_TOML}`"
                ),
            )
            .with_severity(Severity::Warning)
            .at(SourceLocation {
                file: PathBuf::from(BOOK_TOML),
                line: 1,
                column: 1,
                span: None,
            }),
        );
        return chapters;
    }

    for relative in candidates {
        match read_chapter(root, &relative) {
            Ok(chapter) => chapters.push(chapter),
            Err(problem) => diagnostics.push(problem.into_diagnostic(&relative, None)),
        }
    }
    chapters
}

enum Problem {
    Missing,
    Unreadable(String),
}

impl Problem {
    fn into_diagnostic(self, relative: &Path, at: Option<SourceLocation>) -> Diagnostic {
        let shown = crate::display_path(relative);
        let (rule, message) = match self {
            Problem::Missing => (
                rules::MISSING_CHAPTER,
                format!("chapter `{shown}` is listed but the file is not there"),
            ),
            Problem::Unreadable(error) => (
                rules::UNREADABLE_CHAPTER,
                format!("chapter `{shown}` could not be read: {error}"),
            ),
        };
        let location = at.unwrap_or(SourceLocation {
            file: relative.to_path_buf(),
            line: 1,
            column: 1,
            span: None,
        });
        Diagnostic::error(rule, message).at(location)
    }
}

fn read_chapter(root: &Path, relative: &Path) -> std::result::Result<Chapter, Problem> {
    let path = root.join(relative);
    if !path.exists() {
        return Err(Problem::Missing);
    }
    let source =
        std::fs::read_to_string(&path).map_err(|error| Problem::Unreadable(error.to_string()))?;
    let document = Document::parse(&source);
    Ok(Chapter {
        lines: LineIndex::new(&source),
        path,
        relative: relative.to_path_buf(),
        source,
        document,
    })
}

/// Every image a chapter points at must be on disk. A missing one is the
/// single most common way a book breaks, and it is exactly the kind of
/// problem that has to name the file and the line (`PLAN.md` §11.2).
pub(crate) fn check_images(root: &Path, chapter: &Chapter, diagnostics: &mut Vec<Diagnostic>) {
    for image in chapter.document().images() {
        let url = image.url.trim();
        if url.is_empty() || is_external(url) {
            continue;
        }
        let relative = url.split(['?', '#']).next().unwrap_or(url);
        let from_root = root.join(relative);
        let from_chapter = chapter
            .path()
            .parent()
            .map(|directory| directory.join(relative))
            .unwrap_or_else(|| from_root.clone());
        if from_root.is_file() || from_chapter.is_file() {
            continue;
        }
        diagnostics.push(
            Diagnostic::error(
                rules::MISSING_IMAGE,
                format!("image `{relative}` does not exist"),
            )
            .at(chapter.location(image.span)),
        );
    }
}

/// What the parser kept but nothing uses: attribute keys no feature reads
/// (`BK-DOC-001`), and Markdown this build cannot lay out yet
/// (`BK-DOC-002`). Both are warnings — the text is still there, and the
/// book still builds — but until Wave 2 both disappeared from the PDF
/// without a word, which is the one thing a diagnostic is for.
pub(crate) fn check_document(chapter: &Chapter, diagnostics: &mut Vec<Diagnostic>) {
    let mut check_attributes = |attributes: &Attributes, owner: Span| {
        for (key, _) in &attributes.pairs {
            if ATTRIBUTE_KEYS.contains(&key.as_str()) {
                continue;
            }
            let suggestion = match closest(key, ATTRIBUTE_KEYS) {
                Some(near) => format!("did you mean `{near}`? "),
                None => String::new(),
            };
            diagnostics.push(
                Diagnostic::error(
                    rules::UNKNOWN_ATTRIBUTE,
                    format!(
                        "nothing reads the attribute `{key}`; it is kept as written. \
                         {suggestion}Booker reads: {}",
                        ATTRIBUTE_KEYS.join(", ")
                    ),
                )
                .with_severity(Severity::Warning)
                .at(chapter.location(attributes.span.unwrap_or(owner))),
            );
        }
    };

    let mut not_laid_out = Vec::new();
    chapter.document().walk(&mut |node| match node {
        Node::Block(Block::Heading(heading)) => check_attributes(&heading.attributes, heading.span),
        Node::Block(Block::Div(div)) => check_attributes(&div.attributes, div.span),
        Node::Inline(Inline::Image(image)) => check_attributes(&image.attributes, image.span),
        Node::Inline(Inline::Span(span)) => check_attributes(&span.attributes, span.span),
        Node::Block(Block::Html { span, value }) | Node::Inline(Inline::Html { span, value }) => {
            // An HTML comment is how people leave themselves notes in
            // Markdown. It is meant to be invisible, and it is.
            if !value.trim_start().starts_with("<!--") {
                not_laid_out.push((*span, "HTML"));
            }
        }
        Node::Block(Block::Unsupported { span, kind })
        | Node::Inline(Inline::Unsupported { span, kind }) => {
            not_laid_out.push((*span, unsupported_description(kind)));
        }
        _ => {}
    });

    for (span, what) in not_laid_out {
        diagnostics.push(
            Diagnostic::error(
                rules::NOT_LAID_OUT,
                format!(
                    "{what} is kept in the file but not laid out yet, so it does not appear \
                     in the book"
                ),
            )
            .with_severity(Severity::Warning)
            .at(chapter.location(span)),
        );
    }
}

fn unsupported_description(kind: &str) -> &'static str {
    match kind {
        "footnote-reference" => "a footnote reference",
        "footnote-definition" => "a footnote",
        "task-list-marker" => "a checkbox",
        "definition-list" => "a definition list",
        "metadata-block" => "a metadata block",
        _ => "this Markdown",
    }
}

fn is_external(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with("//")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_that_are_not_files_are_left_alone() {
        assert!(is_external("https://example.com/cat.png"));
        assert!(is_external("data:image/png;base64,AAAA"));
        assert!(!is_external("assets/images/cat.png"));
        assert!(!is_external("../assets/cat.png"));
    }
}

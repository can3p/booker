//! The one translation from a book to Typst.
//!
//! Both the window and the terminal go through here, by way of
//! [`crate::Engine::set_book`]: two translations would drift, and
//! `AGENTS.md` §6 forbids the second implementation.
//!
//! Three things this file is careful about:
//!
//! * **Good typography without asking** (`PLAN.md` §6). Justified text,
//!   hyphenation in the book's language, protection against stranded lines,
//!   ligatures and kerning, and — because the author's quotes and hyphens
//!   are passed through rather than escaped — typographic quotes and dashes:
//!   Typst turns `"`, `'`, `--` and `---` into “ ” ’ – — by itself. How the
//!   rest looks comes from the theme (`crate::themes`).
//! * **The author's text is never read as Typst.** Everything a reader could
//!   type that Typst treats as markup is escaped, text never starts a line
//!   where it could open a heading or a list, and structure is written as
//!   function calls (`#emph[…]`, `#heading(…)[…]`) rather than markup that
//!   depends on what surrounds it.
//! * **Every piece of text remembers where it came from.** The [`SpanMap`]
//!   records, for each stretch of generated source, the chapter and byte
//!   range of Markdown it was made from. That is what click-to-source reads,
//!   in both directions.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::ops::Range;

use booker_core::{BookConfig, ChapterStart, Length, Margins};
use booker_doc::{
    Alignment, Attributes, Block, Div, Document, Image, Inline, List, ListItem, Node, Span, Table,
};

use crate::themes::{self, PageNumbers, Paragraphs, Theme, TitleAlign};

/// Where each stretch of generated Typst came from.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpanMap {
    /// Sorted by `typst.start`. Entries nest — a paragraph contains its
    /// text — and a lookup prefers the innermost.
    entries: Vec<Mapping>,
}

#[derive(Debug, Clone, PartialEq)]
struct Mapping {
    typst: Range<usize>,
    chapter: usize,
    source: Span,
    /// Text maps character for character; a block only as a whole.
    text: bool,
}

impl SpanMap {
    /// The chapter and Markdown byte offset that produced the generated
    /// source at `offset`.
    pub fn to_source(&self, offset: usize) -> Option<(usize, usize)> {
        let mapping = self
            .entries
            .iter()
            // Half-open, like every range in Booker: the block that ends
            // where this text begins must not claim it.
            .filter(|m| m.typst.contains(&offset))
            .min_by_key(|m| m.typst.len())?;
        let within = if mapping.text {
            (offset - mapping.typst.start).min(mapping.source.len())
        } else {
            0
        };
        Some((mapping.chapter, mapping.source.start + within))
    }

    /// The generated source for a Markdown offset in a chapter, at the
    /// innermost piece of *text* covering it, so the result lands on
    /// something that was laid out.
    pub fn to_typst(&self, chapter: usize, offset: usize) -> Option<usize> {
        let covering = |m: &&Mapping| {
            m.text && m.chapter == chapter && m.source.start <= offset && offset <= m.source.end
        };
        if let Some(mapping) = self
            .entries
            .iter()
            .filter(covering)
            .min_by_key(|m| m.source.len())
        {
            let within = (offset - mapping.source.start).min(mapping.typst.len().saturating_sub(1));
            return Some(mapping.typst.start + within);
        }
        // Between two pieces of text — a blank line, a list marker: the
        // nearest text after it in the same chapter.
        self.entries
            .iter()
            .filter(|m| m.text && m.chapter == chapter && m.source.start >= offset)
            .min_by_key(|m| m.source.start)
            .map(|m| m.typst.start)
    }
}

/// Turn a whole book into one Typst source file.
///
/// `chapters` are the chapters in reading order, each as its Markdown source
/// and its parsed document. `image_exists` answers whether a path an image
/// points at is a file in the project: a missing image is drawn as a marked
/// placeholder rather than failing the whole book, because a broken book
/// must still show its pages (`AGENTS.md` §7) — the problem itself is
/// reported by the project loader (`BK-REF-001`).
pub fn book_to_typst(
    config: &BookConfig,
    chapters: &[(&str, &Document)],
    image_exists: &dyn Fn(&str) -> bool,
) -> (String, SpanMap) {
    let theme = themes::by_name(config.theme());
    let mut writer = Writer {
        out: String::new(),
        map: SpanMap::default(),
        chapter: 0,
        theme,
        linkable: linkable_ids(chapters),
        image_exists,
    };
    writer.preamble(config);

    let contents = config
        .toc
        .enabled
        .unwrap_or(theme.contents && chapters.len() > 1);
    let start = config.chapter.start.unwrap_or(theme.chapter_start);

    if contents {
        let depth = config.toc.depth.unwrap_or(1).clamp(1, 3);
        let title = match &config.toc.title {
            Some(title) => format!("[{}]", escape_markup(title)),
            // Typst names it in the book's language: "Contents", "Inhalt",
            // "Table des matières"…
            None => "auto".to_string(),
        };
        let _ = writeln!(writer.out, "#outline(title: {title}, depth: {depth})\n");
        writer.out.push_str(END_MARK);
    }

    for (index, (_source, document)) in chapters.iter().enumerate() {
        writer.chapter = index;
        if index > 0 || contents {
            writer.out.push_str(match start {
                ChapterStart::NewPage => "#pagebreak(weak: true)\n",
                ChapterStart::RightPage => "#pagebreak(weak: true, to: \"odd\")\n",
                ChapterStart::Continue => "#v(2em, weak: true)\n",
            });
        }
        writer.out.push_str(START_MARK);
        for block in &document.blocks {
            writer.block(block, 0);
        }
        writer.out.push_str(END_MARK);
    }

    writer.map.entries.sort_by_key(|m| m.typst.start);
    (writer.out, writer.map)
}

/// Where a chapter (or the table of contents) begins and ends, so the footer
/// can tell a page left blank to start a chapter on the right from a page
/// with text on it — a blank page carries no page number.
const START_MARK: &str = "#metadata(\"start\") <booker-mark>\n";
const END_MARK: &str = "#metadata(\"end\") <booker-mark>\n\n";

/// Ids a link can point at: those that appear exactly once. A link to an id
/// that is missing or duplicated becomes plain text rather than a compile
/// error — both are reported by the project (`BK-REF-005`, `BK-REF-006`),
/// and the book still lays out.
fn linkable_ids(chapters: &[(&str, &Document)]) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, document) in chapters {
        document.walk(&mut |node| {
            let attributes = match node {
                Node::Block(Block::Heading(heading)) => Some(&heading.attributes),
                Node::Block(Block::Div(div)) => Some(&div.attributes),
                Node::Inline(Inline::Span(span)) => Some(&span.attributes),
                Node::Inline(Inline::Image(image)) => Some(&image.attributes),
                _ => None,
            };
            if let Some(id) = attributes.and_then(|a| a.id.as_ref()) {
                *counts.entry(id.clone()).or_default() += 1;
            }
        });
    }
    counts
        .into_iter()
        .filter(|(id, count)| *count == 1 && is_label(id))
        .map(|(id, _)| id)
        .collect()
}

/// Whether Typst accepts `id` in `<id>` label syntax.
fn is_label(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}

struct Writer<'a> {
    out: String,
    map: SpanMap,
    chapter: usize,
    theme: Theme,
    linkable: HashSet<String>,
    image_exists: &'a dyn Fn(&str) -> bool,
}

impl Writer<'_> {
    fn preamble(&mut self, config: &BookConfig) {
        let theme = self.theme;
        let (width, height) = config.page.size.dimensions();
        let Margins {
            top,
            bottom,
            inside,
            outside,
        } = config.page.margins;
        // Facing pages swap their margins between left and right pages; a
        // book printed on one side keeps them where they are.
        let sides = if config.page.facing {
            format!("inside: {}, outside: {}", pt(inside), pt(outside))
        } else {
            format!("left: {}, right: {}", pt(inside), pt(outside))
        };

        let out = &mut self.out;
        let _ = writeln!(
            out,
            "#set document(title: \"{}\"{})",
            escape_string(&config.title),
            config
                .author
                .as_ref()
                .map(|a| format!(", author: \"{}\"", escape_string(a)))
                .unwrap_or_default()
        );
        let _ = writeln!(
            out,
            "#set page(width: {}, height: {}, margin: (top: {}, bottom: {}, {sides}), binding: left, footer: {})",
            pt(width),
            pt(height),
            pt(top),
            pt(bottom),
            footer(theme.page_numbers, config.page.facing),
        );
        let _ = writeln!(
            out,
            "#set text(font: (\"{}\", \"Libertinus Serif\"), size: {}pt, lang: \"{}\", costs: (widow: 100%, orphan: 100%))",
            theme.body_font,
            theme.body_size_pt,
            escape_string(&config.language),
        );
        let (spacing, indent) = match theme.paragraphs {
            Paragraphs::Indented { indent_em } => (theme.leading_em, indent_em),
            Paragraphs::Spaced { gap_em } => (theme.leading_em + gap_em, 0.0),
        };
        let _ = writeln!(
            out,
            "#set par(justify: {}, leading: {}em, spacing: {spacing}em, first-line-indent: (amount: {indent}em, all: false))",
            theme.justify, theme.leading_em,
        );
        // Pictures in a book are not "Figure 3": no numbering, and a
        // picture is never split across pages.
        out.push_str("#set figure(numbering: none, gap: 0.8em)\n");
        out.push_str("#show figure: set block(breakable: false)\n");
        out.push_str("#show figure.caption: set text(size: 0.85em)\n");
        out.push_str("#set quote(block: true)\n");
        // In indented prose paragraphs touch, so a list or a quotation needs
        // space of its own or the text after it runs straight into it.
        if let Paragraphs::Indented { .. } = theme.paragraphs {
            out.push_str("#show list: set block(above: 0.9em, below: 0.9em)\n");
            out.push_str("#show enum: set block(above: 0.9em, below: 0.9em)\n");
            out.push_str("#show quote.where(block: true): set block(above: 1em, below: 1em)\n");
            out.push_str("#show table: set block(above: 1em, below: 1em)\n");
        }
        out.push_str("#show raw: set text(size: 0.85em)\n");

        let weight = if theme.heading_bold {
            "\"bold\""
        } else {
            "\"regular\""
        };
        let _ = writeln!(
            out,
            "#show heading: set text(font: (\"{}\", \"Libertinus Serif\"), weight: {weight})",
            theme.heading_font
        );
        out.push_str("#show heading: set par(justify: false)\n");
        for (level, size) in theme.heading_size_pt.iter().enumerate().skip(1) {
            let _ = writeln!(
                out,
                "#show heading.where(level: {}): set text(size: {size}pt)",
                level + 1
            );
        }
        let _ = writeln!(
            out,
            "#show heading.where(level: {}): set text(size: {}pt)",
            4, theme.heading_size_pt[2]
        );
        // A chapter title: dropped down the page, set apart from the text.
        let align = match theme.title_align {
            TitleAlign::Left => "left",
            TitleAlign::Centre => "center",
        };
        let _ = writeln!(
            out,
            "#show heading.where(level: 1): it => block(width: 100%, inset: (top: {}em), below: {}em, align({align}, text(size: {}pt, it.body)))",
            theme.title_sink_em, theme.title_below_em, theme.heading_size_pt[0],
        );
        out.push_str("#set heading(numbering: none)\n");
        // The table of contents lists chapters, with its own quiet style.
        out.push_str("#show outline.entry.where(level: 1): set block(above: 0.9em)\n\n");
    }

    fn block(&mut self, block: &Block, depth: usize) {
        let start = self.out.len();
        match block {
            Block::Heading(heading) => {
                self.break_before(&heading.attributes);
                let _ = write!(self.out, "#heading(level: {})[", heading.level.clamp(1, 6));
                self.inlines(&heading.inlines);
                self.out.push(']');
                self.label(&heading.attributes);
                self.out.push_str("\n\n");
            }
            Block::Paragraph(paragraph) => {
                self.paragraph_start(&paragraph.inlines);
                self.inlines(&paragraph.inlines);
                self.out.push_str("\n\n");
            }
            Block::List(list) => self.list(list, depth),
            Block::Quote(quote) => {
                self.out.push_str("#quote[\n");
                for inner in &quote.blocks {
                    self.block(inner, depth + 1);
                }
                self.out.push_str("]\n\n");
            }
            Block::Code(code) => self.code(code),
            Block::ThematicBreak { .. } => {
                // A scene break, as a novel prints one — not a rule.
                self.out.push_str(
                    "#align(center, block(above: 1.4em, below: 1.4em, text(size: 1.4em)[⁂]))\n\n",
                );
            }
            Block::Div(div) => self.div(div, depth),
            Block::Table(table) => self.table(table),
            // Raw HTML means nothing in a PDF until the whitelisted subset
            // exists (Wave 3); the project reports it (`BK-DOC-002`).
            Block::Html { .. } | Block::Unsupported { .. } => {}
        }
        self.record(start, block.span(), false);
    }

    fn div(&mut self, div: &Div, depth: usize) {
        if div.has_class("page-break") {
            self.out.push_str("#pagebreak(weak: true)\n\n");
            return;
        }
        self.break_before(&div.attributes);
        let labelled = div.attributes.id.as_ref().is_some_and(|id| is_label(id));
        if labelled {
            self.out.push_str("#block[\n");
        }
        for inner in &div.blocks {
            self.block(inner, depth);
        }
        if labelled {
            self.out.push(']');
            self.label(&div.attributes);
            self.out.push_str("\n\n");
        }
    }

    fn break_before(&mut self, attributes: &Attributes) {
        if attributes.get("break-before") == Some("page") {
            self.out.push_str("#pagebreak(weak: true)\n");
        }
    }

    fn label(&mut self, attributes: &Attributes) {
        if let Some(id) = attributes.id.as_ref().filter(|id| is_label(id)) {
            let _ = write!(self.out, " <{id}>");
        }
    }

    fn code(&mut self, code: &booker_doc::CodeBlock) {
        match code.language.as_deref().map(str::trim) {
            // The escape hatches of `PLAN.md` §5.3: Typst for the PDF,
            // HTML for the web output (and so nothing here).
            Some("{=typst}") => {
                self.out.push_str(&code.code);
                self.out.push_str("\n\n");
            }
            Some("{=html}") => {}
            language => {
                // Fenced with more backticks than the code contains, so
                // nothing inside can end it early.
                let fence = "`".repeat(longest_backtick_run(&code.code).max(2) + 1);
                self.out.push_str(&fence);
                if let Some(language) = language.filter(|l| is_label(l)) {
                    self.out.push_str(language);
                }
                self.out.push('\n');
                self.out.push_str(&code.code);
                if !code.code.ends_with('\n') {
                    self.out.push('\n');
                }
                self.out.push_str(&fence);
                self.out.push_str("\n\n");
            }
        }
    }

    fn table(&mut self, table: &Table) {
        let columns = table
            .alignments
            .len()
            .max(table.head.len())
            .max(table.rows.iter().map(Vec::len).max().unwrap_or(0))
            .max(1);
        let aligns: Vec<&str> = (0..columns)
            .map(|index| match table.alignments.get(index) {
                Some(Alignment::Center) => "center",
                Some(Alignment::Right) => "right",
                _ => "left",
            })
            .collect();
        // Rules above and below, and under the header: how a book sets a
        // table, rather than a grid of boxes.
        let _ = writeln!(
            self.out,
            "#table(columns: {columns}, align: ({},), stroke: none, inset: (x: 0.5em, y: 0.35em),",
            aligns.join(", ")
        );
        self.out.push_str("  table.hline(stroke: 0.8pt),\n");
        if !table.head.is_empty() {
            self.out.push_str("  table.header(");
            for cell in &table.head {
                self.cell(cell);
            }
            self.out.push_str("),\n  table.hline(stroke: 0.5pt),\n");
        }
        for row in &table.rows {
            self.out.push_str("  ");
            for index in 0..columns {
                match row.get(index) {
                    Some(cell) => self.cell(cell),
                    None => self.out.push_str("[], "),
                }
            }
            self.out.push('\n');
        }
        self.out.push_str("  table.hline(stroke: 0.8pt),\n)\n\n");
    }

    fn cell(&mut self, cell: &booker_doc::TableCell) {
        self.out.push('[');
        self.inlines(&cell.inlines);
        self.out.push_str("], ");
    }

    fn list(&mut self, list: &List, depth: usize) {
        for item in &list.items {
            self.out.push_str(&"  ".repeat(depth));
            self.out
                .push_str(if list.is_ordered() { "+ " } else { "- " });
            self.item(item, depth);
        }
        self.out.push('\n');
    }

    fn item(&mut self, item: &ListItem, depth: usize) {
        let mut first = true;
        for block in &item.blocks {
            match block {
                Block::Paragraph(paragraph) if first => {
                    let start = self.out.len();
                    self.paragraph_start(&paragraph.inlines);
                    self.inlines(&paragraph.inlines);
                    self.record(start, paragraph.span, false);
                    self.out.push('\n');
                }
                Block::List(nested) => self.list(nested, depth + 1),
                other => {
                    // A second paragraph in a loose item: indented under it.
                    let mut nested = Writer {
                        out: String::new(),
                        map: SpanMap::default(),
                        chapter: self.chapter,
                        theme: self.theme,
                        linkable: self.linkable.clone(),
                        image_exists: self.image_exists,
                    };
                    nested.block(other, depth + 1);
                    let offset = self.out.len();
                    let indent = "  ".repeat(depth + 1);
                    for line in nested.out.trim_end().lines() {
                        self.out.push_str(&indent);
                        self.out.push_str(line);
                        self.out.push('\n');
                    }
                    // The re-indentation shifts offsets; keep block-level
                    // mappings only, which is what a loose item needs.
                    self.record(offset, other.span(), false);
                }
            }
            first = false;
        }
    }

    /// Text at the start of a line that Typst would read as syntax — `- `,
    /// `+ `, `= `, `/ `, `1. ` — is protected by an empty content block in
    /// front of it, which is invisible and ends the line-start position.
    ///
    /// The check reads the leading *text*, across nodes: `1984\. A year.`
    /// arrives as `1984` and `.` separately, and together they would still
    /// open a numbered list.
    fn paragraph_start(&mut self, inlines: &[Inline]) {
        let mut leading = String::new();
        for inline in inlines {
            match inline {
                Inline::Text { value, .. } => leading.push_str(value),
                _ => break,
            }
            if leading.len() > 24 {
                break;
            }
        }
        let digits = leading.chars().take_while(char::is_ascii_digit).count();
        let risky = leading.starts_with(['-', '+', '=', '/'])
            || (digits > 0 && leading[digits..].starts_with(['.', ')']));
        if risky {
            self.out.push_str("#[]");
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }

    fn inline(&mut self, inline: &Inline) {
        match inline {
            Inline::Text { value, span } => {
                let start = self.out.len();
                self.out.push_str(&escape_markup(value));
                self.record(start, *span, true);
            }
            Inline::Emphasis { children, .. } => self.wrapped("#emph[", children),
            Inline::Strong { children, .. } => self.wrapped("#strong[", children),
            Inline::Strikethrough { children, .. } => self.wrapped("#strike[", children),
            Inline::Span(span) => {
                self.wrapped("#[", &span.children);
                self.label(&span.attributes);
                // A label ends where the next text begins; keep them apart.
                if span.attributes.id.is_some() {
                    self.out.push_str("#[]");
                }
            }
            Inline::Code { value, span } => {
                let start = self.out.len();
                let fence = "`".repeat(longest_backtick_run(value) + 1);
                // A single backtick fence cannot start or end with one.
                let pad = if value.starts_with('`') || value.ends_with('`') {
                    " "
                } else {
                    ""
                };
                let _ = write!(self.out, "{fence}{pad}{value}{pad}{fence}");
                self.record(start, *span, false);
            }
            Inline::Link(link) => {
                let target = link.url.trim();
                if let Some(id) = target.strip_prefix('#') {
                    if self.linkable.contains(id) {
                        let _ = write!(self.out, "#link(<{id}>)[");
                        self.inlines(&link.children);
                        self.out.push(']');
                    } else {
                        // Reported as BK-REF-005 or BK-REF-006; the words
                        // stay in the book.
                        self.inlines(&link.children);
                    }
                } else {
                    let _ = write!(self.out, "#link(\"{}\")[", escape_string(target));
                    self.inlines(&link.children);
                    self.out.push(']');
                }
            }
            Inline::Image(image) => self.image(image),
            // Soft breaks are spaces — or, in a theme that keeps the
            // author's lines, line breaks — but never a newline in the
            // generated source: a paragraph is one line of it, so no line
            // of the author's text can start where Typst looks for a
            // heading or a list.
            Inline::SoftBreak { .. } if self.theme.line_breaks => self.out.push_str("#linebreak()"),
            Inline::SoftBreak { .. } => self.out.push(' '),
            Inline::HardBreak { .. } => self.out.push_str("#linebreak()"),
            Inline::Html { .. } | Inline::Unsupported { .. } => {}
        }
    }

    fn wrapped(&mut self, open: &str, children: &[Inline]) {
        self.out.push_str(open);
        self.inlines(children);
        self.out.push(']');
    }

    fn image(&mut self, image: &Image) {
        let start = self.out.len();
        let url = image.url.trim();
        let relative = url.split(['?', '#']).next().unwrap_or(url);
        let width = image
            .attributes
            .get("width")
            .and_then(typst_width)
            .unwrap_or_else(|| format!("{}%", self.theme.image_width_percent));
        let caption = image.alt_text().trim().to_string();

        self.out.push_str("#figure(");
        if (self.image_exists)(relative) {
            let _ = write!(
                self.out,
                "image(\"{}\", width: {width})",
                escape_string(relative)
            );
        } else {
            // The book still lays out; the problems panel names the file.
            let _ = write!(
                self.out,
                "block(width: {width}, height: 4em, stroke: 0.5pt + gray, inset: 0.6em, align(center + horizon, text(size: 0.8em, fill: gray)[missing picture: {}]))",
                escape_markup(relative)
            );
        }
        if !caption.is_empty() {
            let _ = write!(self.out, ", caption: [{}]", escape_markup(&caption));
        }
        self.out.push(')');
        self.label(&image.attributes);
        self.record(start, image.span, false);
    }

    fn record(&mut self, start: usize, source: Span, text: bool) {
        let end = self.out.len();
        if end > start && !source.is_empty() {
            self.map.entries.push(Mapping {
                typst: start..end,
                chapter: self.chapter,
                source,
                text,
            });
        }
    }
}

/// The footer: the page number where the theme puts it, and nothing on a
/// page left blank so that a chapter can start on the right.
fn footer(numbers: PageNumbers, facing: bool) -> String {
    let align = match numbers {
        PageNumbers::None => return "none".to_string(),
        PageNumbers::Centre => "center",
        PageNumbers::Outside if facing => "if calc.odd(p) { right } else { left }",
        PageNumbers::Outside => "right",
    };
    format!(
        "context {{ let p = here().page(); let marks = query(<booker-mark>); \
         let blank = range(calc.max(marks.len() - 1, 0)).any(i => marks.at(i).value == \"end\" and marks.at(i + 1).value == \"start\" and marks.at(i).location().page() < p and p < marks.at(i + 1).location().page()); \
         if not blank {{ align({align}, text(size: 0.85em, counter(page).display())) }} }}"
    )
}

/// An image width as the author wrote it — `60%`, `80mm`, `3in` — in Typst's
/// spelling, or `None` when it is neither.
fn typst_width(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(number) = value.strip_suffix('%') {
        let percent: f64 = number.trim().parse().ok()?;
        return (percent > 0.0).then(|| format!("{}%", percent.min(100.0)));
    }
    let length: Length = value.parse().ok()?;
    Some(pt(length))
}

/// Typst reads plain numbers as points, so every length crosses in points.
fn pt(length: Length) -> String {
    format!("{:.2}pt", length.to_pt())
}

/// Escape what Typst would read as markup in the author's text.
///
/// Deliberately *not* escaped: `"`, `'` and `-`. Typst turns straight quotes
/// into curly ones in the book's language, and `--` and `---` into en and
/// em dashes; escaping them — as Booker did until Wave 2 — is what made
/// every book look typed rather than typeset. `...` becomes an ellipsis for
/// the same reason. Line-start syntax (`=`, `-`, `+`, `1.`) is handled where
/// a line starts (`paragraph_start`), because in the middle of a line those
/// characters are just text.
fn escape_markup(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' | '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '[' | ']' | '~' | '/' => {
                escaped.push('\\');
                escaped.push(character);
            }
            other => escaped.push(other),
        }
    }
    escaped
}

/// Escape a Typst string literal.
fn escape_string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn longest_backtick_run(text: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for character in text.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;
    use booker_doc::Document as Parsed;

    fn typst_with(config: &str, markdown: &[&str]) -> String {
        let config: BookConfig = toml::from_str(config).unwrap();
        let documents: Vec<Parsed> = markdown.iter().map(|m| Parsed::parse(m)).collect();
        let chapters: Vec<(&str, &Parsed)> = markdown.iter().copied().zip(&documents).collect();
        book_to_typst(&config, &chapters, &|_| true).0
    }

    fn typst_of(markdown: &str) -> String {
        typst_with(r#"title = "Test""#, &[markdown])
    }

    #[test]
    fn headings_become_typst_headings() {
        let typst = typst_of("# The Garden\n\n## Later\n");
        assert!(typst.contains("#heading(level: 1)[The Garden]"), "{typst}");
        assert!(typst.contains("#heading(level: 2)[Later]"), "{typst}");
    }

    #[test]
    fn emphasis_and_strong_survive() {
        let typst = typst_of("She was *very* **sure**.\n");
        assert!(typst.contains("#emph[very]"), "{typst}");
        assert!(typst.contains("#strong[sure]"), "{typst}");
    }

    #[test]
    fn text_that_looks_like_typst_markup_is_escaped() {
        let typst = typst_of("Costs #5 and 50% of $x, a_b, 2*3, me@here, a // b.\n");
        for raw in ["\\#5", "\\$x", "a\\_b", "2\\*3", "me\\@here", "\\/\\/"] {
            assert!(typst.contains(raw), "expected {raw} in:\n{typst}");
        }
    }

    #[test]
    fn quotes_and_dashes_are_left_for_typst_to_make_typographic() {
        let typst = typst_of("\"Yes,\" she said -- it's true --- all of it...\n");
        assert!(
            typst.contains("\"Yes,\" she said -- it's true --- all of it..."),
            "quotes and dashes must reach Typst unescaped:\n{typst}"
        );
    }

    #[test]
    fn a_paragraph_that_looks_like_a_list_stays_a_paragraph() {
        for (markdown, generated) in [
            ("\\- not a list", "#[]- not a list"),
            ("1984\\. A year.", "#[]1984. A year."),
            ("= not a heading", "#[]= not a heading"),
            ("\\+ nor this", "#[]+ nor this"),
        ] {
            let typst = typst_of(&format!("{markdown}\n"));
            assert!(typst.contains(generated), "{markdown}: {typst}");
        }
    }

    #[test]
    fn poetry_keeps_the_lines_the_poet_typed_and_prose_joins_them() {
        let verse = "I found the moon\non the kitchen shelf\n";
        let poem = typst_with("title = \"T\"\ntheme = \"poetry\"", &[verse]);
        assert!(
            poem.contains("I found the moon#linebreak()on the kitchen shelf"),
            "{poem}"
        );
        let prose = typst_with(r#"title = "T""#, &[verse]);
        assert!(
            prose.contains("I found the moon on the kitchen shelf"),
            "{prose}"
        );
    }

    #[test]
    fn a_scene_break_is_an_asterism_not_a_rule() {
        let typst = typst_of("One.\n\n---\n\nTwo.\n");
        assert!(typst.contains("⁂"), "{typst}");
        assert!(!typst.contains("#line("), "{typst}");
    }

    #[test]
    fn the_theme_sets_the_face_and_paragraph_style() {
        let novel = typst_with(r#"title = "T""#, &["Hello.\n"]);
        assert!(novel.contains("\"EB Garamond\""), "{novel}");
        assert!(novel.contains("justify: true"), "{novel}");
        assert!(novel.contains("amount: 1.3em"), "{novel}");

        let picture = typst_with("title = \"T\"\ntheme = \"picture-book\"", &["Hello.\n"]);
        assert!(picture.contains("\"Andika\""), "{picture}");
        assert!(picture.contains("justify: false"), "{picture}");
        assert!(picture.contains("footer: none"), "{picture}");
    }

    #[test]
    fn chapters_start_where_the_book_says() {
        let two = ["# One\n", "# Two\n"];
        let novel = typst_with(r#"title = "T""#, &two);
        assert_eq!(
            novel.matches("to: \"odd\"").count(),
            2,
            "after the contents, then each chapter: {novel}"
        );

        let new_page = typst_with("title = \"T\"\n[chapter]\nstart = \"new-page\"", &two);
        assert!(!new_page.contains("to: \"odd\""), "{new_page}");
        assert!(new_page.contains("#pagebreak(weak: true)\n"), "{new_page}");

        let paper = typst_with("title = \"T\"\ntheme = \"paper\"", &two);
        assert!(!paper.contains("#pagebreak"), "a paper runs on: {paper}");
    }

    #[test]
    fn a_book_of_several_chapters_gets_a_table_of_contents() {
        let two = typst_with(r#"title = "T""#, &["# One\n", "# Two\n"]);
        assert!(two.contains("#outline(title: auto, depth: 1)"), "{two}");

        let one = typst_with(r#"title = "T""#, &["# One\n"]);
        assert!(
            !one.contains("#outline"),
            "one chapter needs no contents page"
        );

        let asked = typst_with(
            "title = \"T\"\n[toc]\nenabled = true\ndepth = 2\ntitle = \"What is inside\"",
            &["# One\n"],
        );
        assert!(
            asked.contains("#outline(title: [What is inside], depth: 2)"),
            "{asked}"
        );
    }

    #[test]
    fn a_page_break_and_break_before_become_page_breaks() {
        let typst = typst_of("One.\n\n::: page-break\n:::\n\n# Two {break-before=page}\n");
        assert_eq!(
            typst.matches("#pagebreak(weak: true)\n").count(),
            2,
            "{typst}"
        );
    }

    #[test]
    fn links_to_ids_that_exist_once_are_links_and_others_are_text() {
        let typst =
            typst_of("# Garden {#garden}\n\nSee [the garden](#garden) and [nowhere](#nope).\n");
        assert!(
            typst.contains("#heading(level: 1)[Garden] <garden>"),
            "{typst}"
        );
        assert!(typst.contains("#link(<garden>)[the garden]"), "{typst}");
        assert!(!typst.contains("<nope>"), "{typst}");
        assert!(typst.contains("nowhere"), "{typst}");
    }

    #[test]
    fn a_missing_image_is_a_placeholder_not_a_failed_book() {
        let config: BookConfig = toml::from_str(r#"title = "T""#).unwrap();
        let markdown = "![Cat](assets/cat.png)\n";
        let document = Parsed::parse(markdown);
        let (typst, _) = book_to_typst(&config, &[(markdown, &document)], &|_| false);
        assert!(!typst.contains("image(\"assets/cat.png\""), "{typst}");
        assert!(
            typst.contains("missing picture: assets\\/cat.png"),
            "{typst}"
        );
    }

    #[test]
    fn image_width_comes_from_the_attribute_or_the_theme() {
        let typst = typst_of("![](a.png){width=50%}\n\n![](b.png){width=40mm}\n\n![](c.png)\n");
        assert!(typst.contains("image(\"a.png\", width: 50%)"), "{typst}");
        assert!(
            typst.contains("image(\"b.png\", width: 113.39pt)"),
            "{typst}"
        );
        assert!(typst.contains("image(\"c.png\", width: 80%)"), "{typst}");
    }

    #[test]
    fn tables_and_strikethrough_are_laid_out() {
        let typst = typst_of("| A | B |\n|---|--:|\n| 1 | 2 |\n\n~~gone~~\n");
        assert!(
            typst.contains("#table(columns: 2, align: (left, right,)"),
            "{typst}"
        );
        assert!(typst.contains("table.header([A], [B], )"), "{typst}");
        assert!(typst.contains("#strike[gone]"), "{typst}");
    }

    #[test]
    fn a_typst_block_passes_through_and_an_html_block_does_not() {
        let typst = typst_of("```{=typst}\n#rect(width: 1cm)\n```\n\n```{=html}\n<b>x</b>\n```\n");
        assert!(typst.contains("#rect(width: 1cm)\n"), "{typst}");
        assert!(!typst.contains("<b>"), "{typst}");
    }

    #[test]
    fn facing_pages_swap_margins_and_single_pages_do_not() {
        let facing = typst_of("Hi.\n");
        assert!(facing.contains("inside: "), "{facing}");
        let single = typst_with("title = \"T\"\n[page]\nfacing = false", &["Hi.\n"]);
        assert!(
            single.contains("left: ") && !single.contains("inside: "),
            "{single}"
        );
    }

    #[test]
    fn the_span_map_leads_from_generated_text_back_to_the_markdown() {
        let markdown = "# Garden\n\nMia opened the *green* door.\n";
        let config: BookConfig = toml::from_str(r#"title = "T""#).unwrap();
        let document = Parsed::parse(markdown);
        let (typst, map) = book_to_typst(&config, &[(markdown, &document)], &|_| true);

        let green = typst.find("green").unwrap();
        let (chapter, offset) = map.to_source(green + 2).unwrap();
        assert_eq!(chapter, 0);
        assert_eq!(&markdown[offset..offset + 3], "een");

        let door = markdown.find("door").unwrap();
        let back = map.to_typst(0, door).unwrap();
        assert_eq!(&typst[back..back + 4], "door");
    }
}

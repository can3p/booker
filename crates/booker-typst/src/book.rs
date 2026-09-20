//! A minimal document model to Typst translation.
//!
//! ## Deliberately temporary
//!
//! The real translation — styles, page rules, frames, anchored images — is
//! Wave 2 track B, and replaces this file in place. This exists so that
//! Wave 0 ended with a book a person can actually look at: `booker build`
//! writes a PDF instead of describing one. It covers headings, paragraphs,
//! emphasis, lists, quotes, code, images and links, and nothing else.
//!
//! It was written in `booker-cli` and moved here in Wave 1, when the app
//! needed the same translation. There is one way for a book to become
//! pages, and both the window and the terminal go through it: two would
//! drift, and `AGENTS.md` §6 forbids the second implementation. The move
//! also puts it where `PLAN.md` §6 always had it — codegen belongs beside
//! the engine — so Wave 2 track B replaces one file rather than two.

use booker_core::{BookConfig, Length, Margins};
use booker_doc::{Block, Document, Inline, List, ListItem};

/// Turn a whole book into one Typst source file.
pub fn book_to_typst(config: &BookConfig, chapters: &[(&str, &Document)]) -> String {
    let mut out = String::new();
    preamble(config, &mut out);

    for (index, (_source, document)) in chapters.iter().enumerate() {
        // Chapters start on a new page (`PLAN.md` §5.2, `chapter.start`);
        // configurable once styles exist.
        if index > 0 {
            out.push_str("#pagebreak()\n\n");
        }
        for block in &document.blocks {
            write_block(block, 0, &mut out);
        }
    }
    out
}

fn preamble(config: &BookConfig, out: &mut String) {
    let (width, height) = config.page.size.dimensions();
    let Margins {
        top,
        bottom,
        inside,
        outside,
    } = config.page.margins;

    out.push_str(&format!(
        "#set page(width: {}, height: {}, margin: (top: {}, bottom: {}, inside: {}, outside: {}), binding: left)\n",
        pt(width),
        pt(height),
        pt(top),
        pt(bottom),
        pt(inside),
        pt(outside),
    ));
    out.push_str(&format!(
        "#set text(size: 11pt, lang: \"{}\")\n",
        escape_string(&config.language)
    ));
    // Justification and hyphenation are on from the start: they are most of
    // what separates an amateur-looking page from a decent one
    // (`PLAN.md` §6).
    out.push_str("#set par(justify: true, leading: 0.65em)\n");
    out.push_str("#set heading(numbering: none)\n\n");
}

/// Typst reads plain numbers as points, so every length crosses in points.
fn pt(length: Length) -> String {
    format!("{:.2}pt", length.to_pt())
}

fn write_block(block: &Block, depth: usize, out: &mut String) {
    match block {
        Block::Heading(heading) => {
            out.push_str(&"=".repeat(heading.level.clamp(1, 6) as usize));
            out.push(' ');
            write_inlines(&heading.inlines, out);
            out.push_str("\n\n");
        }
        Block::Paragraph(paragraph) => {
            write_inlines(&paragraph.inlines, out);
            out.push_str("\n\n");
        }
        Block::List(list) => write_list(list, depth, out),
        Block::Quote(quote) => {
            out.push_str("#quote(block: true)[\n");
            for inner in &quote.blocks {
                write_block(inner, depth + 1, out);
            }
            out.push_str("]\n\n");
        }
        Block::Code(code) => {
            // A raw block, fenced with more backticks than the code contains
            // so that nothing inside can end it early.
            let fence = "`".repeat(longest_backtick_run(&code.code).max(2) + 1);
            out.push_str(&fence);
            if let Some(language) = &code.language {
                out.push_str(language);
            }
            out.push('\n');
            out.push_str(&code.code);
            if !code.code.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&fence);
            out.push_str("\n\n");
        }
        Block::ThematicBreak { .. } => out.push_str("#line(length: 100%)\n\n"),
        // Raw HTML has no meaning in a PDF until the whitelisted subset
        // exists (Wave 3). Dropping it silently would be worse than leaving
        // the gap visible, but printing markup would be worse still.
        Block::Html { .. } => {}
        Block::Unsupported { .. } => {}
    }
}

fn write_list(list: &List, depth: usize, out: &mut String) {
    for item in &list.items {
        out.push_str(&"  ".repeat(depth));
        out.push_str(if list.is_ordered() { "+ " } else { "- " });
        write_item(item, depth, out);
    }
    out.push('\n');
}

fn write_item(item: &ListItem, depth: usize, out: &mut String) {
    let mut first = true;
    for block in &item.blocks {
        match block {
            Block::Paragraph(paragraph) if first => {
                write_inlines(&paragraph.inlines, out);
                out.push('\n');
            }
            Block::List(nested) => write_list(nested, depth + 1, out),
            other => {
                let mut nested = String::new();
                write_block(other, depth + 1, &mut nested);
                out.push_str(nested.trim_end());
                out.push('\n');
            }
        }
        first = false;
    }
}

fn write_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text { value, .. } => out.push_str(&escape_markup(value)),
            Inline::Emphasis { children, .. } => {
                out.push('_');
                write_inlines(children, out);
                out.push('_');
            }
            Inline::Strong { children, .. } => {
                out.push('*');
                write_inlines(children, out);
                out.push('*');
            }
            Inline::Code { value, .. } => {
                let fence = "`".repeat(longest_backtick_run(value) + 1);
                out.push_str(&fence);
                out.push_str(value);
                out.push_str(&fence);
            }
            Inline::Link(link) => {
                out.push_str(&format!("#link(\"{}\")[", escape_string(&link.url)));
                write_inlines(&link.children, out);
                out.push(']');
            }
            Inline::Image(image) => {
                // Paths are written relative to the project root, and the
                // generated file sits at the root, so they carry over.
                out.push_str(&format!(
                    "\n#figure(image(\"{}\", width: 80%)",
                    escape_string(&image.url)
                ));
                let caption = image
                    .alt
                    .iter()
                    .map(Inline::text)
                    .collect::<String>()
                    .trim()
                    .to_string();
                if caption.is_empty() {
                    out.push_str(")\n");
                } else {
                    out.push_str(&format!(", caption: [{}])\n", escape_markup(&caption)));
                }
            }
            Inline::SoftBreak { .. } => out.push('\n'),
            Inline::HardBreak { .. } => out.push_str(" \\\n"),
            Inline::Html { .. } => {}
        }
    }
}

/// Escape the characters Typst reads as markup.
///
/// The list is deliberately generous: a book full of apostrophes, asterisks
/// and hash signs must come out looking like the author's text, not like
/// something that half-compiled.
fn escape_markup(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' | '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '[' | ']' | '=' | '-' | '+'
            | '/' | '~' | '\'' | '"' => {
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

    fn typst_of(markdown: &str) -> String {
        let document = Parsed::parse(markdown);
        let config: BookConfig = toml::from_str(r#"title = "Test""#).unwrap();
        book_to_typst(&config, &[(markdown, &document)])
    }

    #[test]
    fn headings_become_typst_headings() {
        let typst = typst_of("# The Garden\n\n## Later\n");
        assert!(typst.contains("= The Garden"), "{typst}");
        assert!(typst.contains("== Later"), "{typst}");
    }

    #[test]
    fn emphasis_and_strong_survive() {
        let typst = typst_of("She was *very* **sure**.\n");
        assert!(typst.contains("_very_"), "{typst}");
        assert!(typst.contains("*sure*"), "{typst}");
    }

    #[test]
    fn text_that_looks_like_typst_markup_is_escaped() {
        let typst = typst_of("Costs #5 and 50% of $x, a_b, 2*3, me@here.\n");
        for raw in ["\\#5", "\\$x", "a\\_b", "2\\*3", "me\\@here"] {
            assert!(typst.contains(raw), "expected {raw} in:\n{typst}");
        }
    }

    #[test]
    fn code_fences_are_longer_than_the_code_inside_them() {
        // The inner code holds a run of two backticks, so the fence must be
        // at least three or the block would end in the middle of the code.
        let typst = typst_of("```\nlet a = ``x``;\n```\n");
        let fence = typst
            .lines()
            .find(|line| line.starts_with("``"))
            .expect("a fenced block");
        assert!(fence.len() > 2, "fence {fence:?} in:\n{typst}");
    }

    #[test]
    fn lists_keep_their_kind() {
        let typst = typst_of("- one\n- two\n\n1. first\n2. second\n");
        assert!(typst.contains("- one"), "{typst}");
        assert!(typst.contains("+ first"), "{typst}");
    }

    #[test]
    fn images_become_figures_with_their_alt_text_as_caption() {
        let typst = typst_of("![Tulips in May](assets/images/tulips.jpg)\n");
        assert!(
            typst.contains("image(\"assets/images/tulips.jpg\""),
            "{typst}"
        );
        assert!(typst.contains("caption: [Tulips in May]"), "{typst}");
    }

    #[test]
    fn the_page_is_set_up_from_the_configuration() {
        let config: BookConfig = toml::from_str(
            r#"
            title = "Mia"
            language = "nl"

            [page]
            size = "a5"
            "#,
        )
        .unwrap();
        let document = Parsed::parse("Hello.\n");
        let typst = book_to_typst(&config, &[("Hello.\n", &document)]);
        assert!(typst.contains("width: 419.53pt"), "A5 in points: {typst}");
        assert!(typst.contains("lang: \"nl\""), "{typst}");
        assert!(typst.contains("justify: true"), "{typst}");
    }

    #[test]
    fn chapters_start_on_a_new_page() {
        let first = Parsed::parse("One.\n");
        let second = Parsed::parse("Two.\n");
        let config: BookConfig = toml::from_str(r#"title = "Test""#).unwrap();
        let typst = book_to_typst(&config, &[("One.\n", &first), ("Two.\n", &second)]);
        assert_eq!(typst.matches("#pagebreak()").count(), 1, "{typst}");
    }
}

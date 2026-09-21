//! The attribute pass over inlines: `![alt](src){width=50%}` and
//! `[some text]{.class}`.
//!
//! `pulldown-cmark` reads both as ordinary text after the image, or as text
//! with brackets in it. This pass finds them in the parsed tree and turns
//! them into attributes and `Inline::Span`s.
//!
//! It only ever touches a text node whose value is **exactly** the source it
//! came from. Then a byte offset into the value is a byte offset into the
//! file, and every span this pass creates still slices the author's text.
//! A text node that differs from its source — one with an entity in it — is
//! left alone. An escaped bracket needs its own check (`escaped`), because
//! the parser's span for `\[` leaves the backslash out and so the node
//! *does* look exactly like its source: an author who escaped a bracket did
//! not mean a span.

use crate::attributes::parse_block;
use crate::model::{Block, Inline, InlineSpan};
use crate::span::Span;

/// Run the pass over every inline container in `blocks`.
pub fn attach(blocks: &mut [Block], source: &str) {
    for block in blocks {
        match block {
            Block::Paragraph(paragraph) => {
                paragraph.inlines = process(std::mem::take(&mut paragraph.inlines), source);
            }
            Block::Heading(heading) => {
                heading.inlines = process(std::mem::take(&mut heading.inlines), source);
                if !heading.attributes.is_empty() && heading.attributes.span.is_none() {
                    heading.attributes.span = trailing_braces(heading.span, source);
                }
            }
            Block::Quote(quote) => attach(&mut quote.blocks, source),
            Block::Div(div) => attach(&mut div.blocks, source),
            Block::List(list) => {
                for item in &mut list.items {
                    attach(&mut item.blocks, source);
                }
            }
            Block::Table(table) => {
                for cell in table.head.iter_mut().chain(table.rows.iter_mut().flatten()) {
                    cell.inlines = process(std::mem::take(&mut cell.inlines), source);
                }
            }
            Block::Code(_)
            | Block::ThematicBreak { .. }
            | Block::Html { .. }
            | Block::Unsupported { .. } => {}
        }
    }
}

fn process(inlines: Vec<Inline>, source: &str) -> Vec<Inline> {
    let inlines = inlines
        .into_iter()
        .map(|inline| process_children(inline, source))
        .collect();
    let inlines = merge_text(inlines);
    let inlines = image_attributes(inlines, source);
    bracketed_spans(inlines, source)
}

fn process_children(inline: Inline, source: &str) -> Inline {
    match inline {
        Inline::Emphasis { span, children } => Inline::Emphasis {
            span,
            children: process(children, source),
        },
        Inline::Strong { span, children } => Inline::Strong {
            span,
            children: process(children, source),
        },
        Inline::Strikethrough { span, children } => Inline::Strikethrough {
            span,
            children: process(children, source),
        },
        Inline::Link(mut link) => {
            link.children = process(link.children, source);
            Inline::Link(link)
        }
        other => other,
    }
}

/// `pulldown-cmark` splits text at every character that *might* have been
/// syntax — a `[` that turned out not to open a link arrives as its own
/// text node. Rejoin neighbours that touch, so a span's brackets can be
/// found in one place.
fn merge_text(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut merged: Vec<Inline> = Vec::with_capacity(inlines.len());
    for inline in inlines {
        if let (
            Some(Inline::Text {
                span: previous,
                value: text,
            }),
            Inline::Text { span, value },
        ) = (merged.last_mut(), &inline)
        {
            if previous.end == span.start {
                text.push_str(value);
                *previous = previous.join(*span);
                continue;
            }
        }
        merged.push(inline);
    }
    merged
}

fn is_raw(value: &str, span: Span, source: &str) -> bool {
    span.slice(source) == Some(value)
}

/// `![alt](src){width=50%}`: the braces arrive as text straight after the
/// image.
fn image_attributes(mut inlines: Vec<Inline>, source: &str) -> Vec<Inline> {
    let mut index = 0;
    while index + 1 < inlines.len() {
        let (image_end, text_span, text) = match (&inlines[index], &inlines[index + 1]) {
            (Inline::Image(image), Inline::Text { span, value })
                if value.starts_with('{') && image.span.end == span.start =>
            {
                (image.span.end, *span, value.clone())
            }
            _ => {
                index += 1;
                continue;
            }
        };
        if !is_raw(&text, text_span, source) {
            index += 1;
            continue;
        }
        let Some(close) = text.find('}') else {
            index += 1;
            continue;
        };
        let block_span = Span::new(image_end, image_end + close + 1);
        let Some(attributes) = parse_block(&text[..=close], block_span) else {
            index += 1;
            continue;
        };
        if let Inline::Image(image) = &mut inlines[index] {
            image.attributes = attributes;
            image.span = image.span.join(block_span);
        }
        let rest = &text[close + 1..];
        if rest.is_empty() {
            inlines.remove(index + 1);
        } else {
            inlines[index + 1] = Inline::Text {
                span: Span::new(block_span.end, text_span.end),
                value: rest.to_string(),
            };
        }
        index += 1;
    }
    inlines
}

/// `[some text]{.class}` — possibly with emphasis and other inlines inside
/// the brackets, so the opening and closing text can be different nodes.
fn bracketed_spans(mut inlines: Vec<Inline>, source: &str) -> Vec<Inline> {
    let mut index = 0;
    while index < inlines.len() {
        match find_span(&inlines, index, source) {
            Some(found) => {
                let (replacement, resume) = build_span(&mut inlines, found);
                let count = replacement.len();
                inlines.splice(found.open_node..=found.close_node, replacement);
                index += resume.min(count);
            }
            None => index += 1,
        }
    }
    inlines
}

#[derive(Clone, Copy)]
struct Found {
    open_node: usize,
    /// Byte offset of `[` in the opening text.
    open_at: usize,
    close_node: usize,
    /// Byte offset of `]` in the closing text.
    close_at: usize,
    /// Byte offset of the `}` that ends the attributes, in the closing text.
    brace_end: usize,
}

fn raw_text(inline: &Inline, source: &str) -> Option<(Span, String)> {
    match inline {
        Inline::Text { span, value } if is_raw(value, *span, source) => {
            Some((*span, value.clone()))
        }
        _ => None,
    }
}

/// Find a span opening at a `[` in the text node at `index`.
///
/// The `]` that closes it is the one that *matches* the `[` — brackets are
/// counted, as Pandoc counts them — and it must be followed at once by an
/// attribute block. `[^1] and [x]{.y}` is therefore a span around `x` only.
/// The scan gives up at a text node it cannot read byte for byte (one with
/// an escape in it), because it could not tell an escaped bracket from a
/// real one there.
fn find_span(inlines: &[Inline], index: usize, source: &str) -> Option<Found> {
    let (open_span, open_text) = raw_text(&inlines[index], source)?;
    'candidates: for (open_at, _) in open_text.match_indices('[') {
        if escaped(source, open_span.start + open_at) {
            continue;
        }
        let mut depth = 0usize;
        for (node, candidate) in inlines.iter().enumerate().skip(index) {
            let (span, text) = match candidate {
                Inline::Text { .. } => match raw_text(candidate, source) {
                    Some(found) => found,
                    None => continue 'candidates,
                },
                // Brackets inside code, links or emphasis are not ours to count.
                _ => continue,
            };
            let from = if node == index { open_at } else { 0 };
            for (offset, character) in text[from..].char_indices() {
                let at = from + offset;
                if escaped(source, span.start + at) {
                    continue;
                }
                match character {
                    '[' => depth += 1,
                    ']' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            let rest = &text[at + 1..];
                            if !rest.starts_with('{') {
                                continue 'candidates;
                            }
                            let brace_end = at + 1 + rest.find('}')?;
                            parse_block(&text[at + 1..=brace_end], Span::new(0, 0))?;
                            return Some(Found {
                                open_node: index,
                                open_at,
                                close_node: node,
                                close_at: at,
                                brace_end,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Whether the character at `offset` was written with a backslash before it.
///
/// Needed because `pulldown-cmark` gives the text of `\[` a span covering
/// only the `[`: the node looks exactly like its source, and the escape is
/// visible only one byte to the left, outside it.
fn escaped(source: &str, offset: usize) -> bool {
    let before = &source.as_bytes()[..offset.min(source.len())];
    let backslashes = before.iter().rev().take_while(|b| **b == b'\\').count();
    backslashes % 2 == 1
}

/// Replace the nodes from `[` to `}` with: the text before `[`, the span,
/// and the text after `}`. Returns the replacement, and how many of its
/// nodes to step over before looking for the next span (the text after `}`
/// may hold another).
fn build_span(inlines: &mut [Inline], found: Found) -> (Vec<Inline>, usize) {
    let text_of = |inline: &Inline| match inline {
        Inline::Text { span, value } => (*span, value.clone()),
        _ => unreachable!("find_span only returns text nodes at both ends"),
    };
    let (open_span, open_text) = text_of(&inlines[found.open_node]);
    let (close_span, close_text) = text_of(&inlines[found.close_node]);

    let mut replacement = Vec::new();
    if found.open_at > 0 {
        replacement.push(Inline::Text {
            span: Span::new(open_span.start, open_span.start + found.open_at),
            value: open_text[..found.open_at].to_string(),
        });
    }

    let mut children = Vec::new();
    let push_text = |children: &mut Vec<Inline>, start: usize, text: &str| {
        if !text.is_empty() {
            children.push(Inline::Text {
                span: Span::new(start, start + text.len()),
                value: text.to_string(),
            });
        }
    };
    if found.open_node == found.close_node {
        push_text(
            &mut children,
            open_span.start + found.open_at + 1,
            &open_text[found.open_at + 1..found.close_at],
        );
    } else {
        push_text(
            &mut children,
            open_span.start + found.open_at + 1,
            &open_text[found.open_at + 1..],
        );
        children.extend(
            inlines[found.open_node + 1..found.close_node]
                .iter()
                .cloned(),
        );
        push_text(
            &mut children,
            close_span.start,
            &close_text[..found.close_at],
        );
    }

    let attributes_span = Span::new(
        close_span.start + found.close_at + 1,
        close_span.start + found.brace_end + 1,
    );
    let attributes = parse_block(
        &close_text[found.close_at + 1..=found.brace_end],
        attributes_span,
    )
    .unwrap_or_default();
    replacement.push(Inline::Span(InlineSpan {
        children,
        attributes,
        span: Span::new(open_span.start + found.open_at, attributes_span.end),
    }));
    let resume = replacement.len();

    let after = &close_text[found.brace_end + 1..];
    if !after.is_empty() {
        replacement.push(Inline::Text {
            span: Span::new(attributes_span.end, close_span.end),
            value: after.to_string(),
        });
    }
    (replacement, resume)
}

/// Where `# Heading {#id .class}` wrote its braces. The parser reads heading
/// attributes itself but does not say where they were.
fn trailing_braces(span: Span, source: &str) -> Option<Span> {
    let text = span.slice(source)?;
    let trimmed = text.trim_end();
    if !trimmed.ends_with('}') {
        return None;
    }
    let open = trimmed.rfind('{')?;
    Some(Span::new(span.start + open, span.start + trimmed.len()))
}

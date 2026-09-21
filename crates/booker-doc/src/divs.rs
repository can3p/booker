//! Fenced divs: `::: {.poem}` … `:::`.
//!
//! `pulldown-cmark` has no idea what a `:::` line is — to it, one is a
//! paragraph that happens to start with colons, and it would happily merge
//! the fence into the paragraph that follows. So divs are handled around the
//! parser rather than inside it:
//!
//! 1. find the fence lines, skipping any inside code blocks;
//! 2. overwrite each fence line with spaces, byte for byte, so that to the
//!    parser it is a blank line — which ends a paragraph, exactly as a fence
//!    should — **and every byte offset in the file stays where it was**;
//! 3. parse that, and regroup the top-level blocks into divs by their spans.
//!
//! Step 2 is what keeps click-to-source honest: no span is invented or
//! shifted, every one still slices the author's own text out of the file.
//!
//! Fences are recognised at the start of a line only. A `:::` inside a quote
//! or a list item is text, as it is in most Markdown tools.

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::attributes::parse_fence_info;
use crate::model::{Attributes, Block, Div};
use crate::span::Span;

/// One fenced div, as found in the source, with the divs nested in it.
#[derive(Debug)]
pub struct Fence {
    /// The opening line, without its line ending.
    open: Span,
    /// The closing line, when there is one.
    close: Option<Span>,
    attributes: Attributes,
    children: Vec<Fence>,
}

/// The source with every fence line blanked, and the fences it contained.
pub fn find(source: &str, options: pulldown_cmark::Options) -> (String, Vec<Fence>) {
    if !source.contains(":::") {
        return (source.to_string(), Vec::new());
    }
    let code = code_ranges(source, options);
    let in_code = |offset: usize| code.iter().any(|range| range.contains(&offset));

    let mut masked = source.as_bytes().to_vec();
    let mut stack: Vec<Fence> = Vec::new();
    let mut top: Vec<Fence> = Vec::new();

    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        let colons = content.chars().take_while(|c| *c == ':').count();
        if colons < 3 || in_code(start) {
            continue;
        }
        let line_span = Span::new(start, start + content.len());
        // Pandoc lets an opening fence end in colons too: `::: poem :::`.
        let info = content[colons..].trim().trim_end_matches(':').trim();

        if info.is_empty() {
            let Some(mut open) = stack.pop() else {
                // A closing fence with nothing open is just text.
                continue;
            };
            open.close = Some(line_span);
            blank(&mut masked, line_span);
            match stack.last_mut() {
                Some(parent) => parent.children.push(open),
                None => top.push(open),
            }
        } else {
            let info_start = start + content.find(info).unwrap_or(colons);
            let info_span = Span::new(info_start, info_start + info.len());
            let Some(attributes) = parse_fence_info(info, info_span) else {
                // `:::` followed by something that is not attributes: text.
                continue;
            };
            blank(&mut masked, line_span);
            stack.push(Fence {
                open: line_span,
                close: None,
                attributes,
                children: Vec::new(),
            });
        }
    }

    // Divs never closed run to the end of the file (or of the div around
    // them). Nothing inside them is lost.
    while let Some(open) = stack.pop() {
        match stack.last_mut() {
            Some(parent) => parent.children.push(open),
            None => top.push(open),
        }
    }

    // Only ASCII colons and line content were replaced by ASCII spaces, so
    // this is still valid UTF-8; fall back to the original if that ever
    // stopped being true rather than panic on a user's file.
    let masked = String::from_utf8(masked).unwrap_or_else(|_| source.to_string());
    (masked, top)
}

/// Group `blocks` — the top level of a document parsed from the masked
/// source — into the divs `fences` describe.
pub fn assemble(blocks: Vec<Block>, fences: Vec<Fence>, end: usize) -> Vec<Block> {
    if fences.is_empty() {
        return blocks;
    }
    let mut result = Vec::with_capacity(blocks.len());
    let mut blocks = blocks.into_iter().peekable();

    for fence in fences {
        while let Some(block) = blocks.next_if(|b| b.span().start < fence.open.start) {
            result.push(block);
        }
        let inner_end = fence.close.map(|close| close.start).unwrap_or(end);
        let mut inner = Vec::new();
        while let Some(block) = blocks.next_if(|b| b.span().start < inner_end) {
            inner.push(block);
        }
        let span_end = fence.close.map(|close| close.end).unwrap_or(inner_end);
        result.push(Block::Div(Div {
            attributes: fence.attributes,
            blocks: assemble(inner, fence.children, inner_end),
            span: Span::new(fence.open.start, span_end.max(fence.open.end)),
        }));
    }
    result.extend(blocks);
    result
}

fn blank(bytes: &mut [u8], span: Span) {
    for byte in &mut bytes[span.range()] {
        *byte = b' ';
    }
}

/// Where the code blocks are, so a `:::` inside one stays code.
fn code_ranges(source: &str, options: pulldown_cmark::Options) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) | Event::Start(Tag::HtmlBlock) => ranges.push(range),
            Event::End(TagEnd::CodeBlock) | Event::End(TagEnd::HtmlBlock) => {}
            _ => {}
        }
    }
    ranges
}

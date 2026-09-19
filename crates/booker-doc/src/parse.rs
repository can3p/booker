//! Markdown to the document model, keeping every byte range.
//!
//! The parser is `pulldown-cmark`, chosen on source positions alone; the
//! evidence is in `docs/FINDINGS.md` ("Markdown parser: pulldown-cmark, for
//! source positions"). What matters here: `into_offset_iter()` gives a
//! non-optional byte range with every event, including inline ones, and each
//! range slices back out of the source to the text that produced it. We keep
//! those ranges verbatim; nothing in this file invents a position.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::model::{
    Attributes, Block, CodeBlock, Document, Heading, Image, Inline, Link, List, ListItem,
    Paragraph, Quote,
};
use crate::span::Span;

/// Parse a Markdown file into the document model.
///
/// Infallible by design: a book with unusual Markdown in it still opens
/// (`AGENTS.md` §7). Anything this build does not model is kept as
/// `Block::Unsupported` with its span rather than dropped.
pub fn parse(source: &str) -> Document {
    let mut options = Options::empty();
    // `# Heading {#id .class}` — part of Booker's attribute syntax
    // (`PLAN.md` §5.3) that the parser already understands. The rest of the
    // syntax (images, spans, fenced divs) is Wave 1, and lands as a pass
    // over these same spans.
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let mut stack: Vec<Frame> = vec![Frame::new(FrameKind::Root, Span::new(0, source.len()))];

    for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
        let span = Span::from(range);
        match event {
            Event::Start(tag) => stack.push(Frame::new(FrameKind::from_tag(tag), span)),
            Event::End(tag) => {
                // `pulldown-cmark` emits balanced events, so there is always
                // a frame to close; if that ever stops being true we want a
                // slightly odd document, not a panic in a user's project.
                if stack.len() > 1 {
                    let frame = stack.pop().expect("checked by the length test above");
                    let built = frame.finish(span);
                    attach(&mut stack, built);
                } else {
                    debug_assert!(false, "unbalanced end event: {tag:?}");
                }
            }
            Event::Text(text) => {
                if let Some(frame) = stack.last_mut() {
                    if frame.kind.collects_raw_text() {
                        frame.text.push_str(&text);
                    } else {
                        frame.inlines.push(Inline::Text {
                            span,
                            value: text.into_string(),
                        });
                    }
                }
            }
            Event::Code(value) => push_inline(
                &mut stack,
                Inline::Code {
                    span,
                    value: value.into_string(),
                },
            ),
            Event::InlineHtml(value) => push_inline(
                &mut stack,
                Inline::Html {
                    span,
                    value: value.into_string(),
                },
            ),
            Event::Html(value) => {
                if let Some(frame) = stack.last_mut() {
                    if frame.kind.collects_raw_text() {
                        frame.text.push_str(&value);
                    } else {
                        frame.blocks.push(Block::Html {
                            span,
                            value: value.into_string(),
                        });
                    }
                }
            }
            Event::SoftBreak => push_inline(&mut stack, Inline::SoftBreak { span }),
            Event::HardBreak => push_inline(&mut stack, Inline::HardBreak { span }),
            Event::Rule => push_block(&mut stack, Block::ThematicBreak { span }),
            other => push_block(
                &mut stack,
                Block::Unsupported {
                    span,
                    kind: unsupported_name(&other).to_string(),
                },
            ),
        }
    }

    // Close anything still open (only reachable if the event stream were
    // unbalanced) so no content is lost.
    while stack.len() > 1 {
        if let Some(frame) = stack.pop() {
            let span = frame.span;
            let built = frame.finish(span);
            attach(&mut stack, built);
        }
    }

    let root = stack
        .pop()
        .unwrap_or_else(|| Frame::new(FrameKind::Root, Span::new(0, source.len())));
    Document {
        blocks: root.blocks,
        span: Span::new(0, source.len()),
    }
}

fn unsupported_name(event: &Event<'_>) -> &'static str {
    match event {
        Event::FootnoteReference(_) => "footnote-reference",
        Event::TaskListMarker(_) => "task-list-marker",
        Event::InlineMath(_) => "inline-math",
        Event::DisplayMath(_) => "display-math",
        _ => "unknown",
    }
}

enum Built {
    Block(Block),
    Item(ListItem),
    Inline(Inline),
}

fn attach(stack: &mut [Frame], built: Built) {
    let Some(parent) = stack.last_mut() else {
        return;
    };
    match built {
        Built::Block(block) => parent.blocks.push(block),
        Built::Item(item) => parent.items.push(item),
        Built::Inline(inline) => parent.inlines.push(inline),
    }
}

fn push_inline(stack: &mut [Frame], inline: Inline) {
    if let Some(frame) = stack.last_mut() {
        frame.inlines.push(inline);
    }
}

fn push_block(stack: &mut [Frame], block: Block) {
    if let Some(frame) = stack.last_mut() {
        frame.blocks.push(block);
    }
}

enum FrameKind {
    Root,
    Paragraph,
    Heading { level: u8, attributes: Attributes },
    Quote,
    List { start: Option<u64> },
    Item,
    Code { language: Option<String> },
    HtmlBlock,
    Emphasis,
    Strong,
    Link { url: String, title: Option<String> },
    Image { url: String, title: Option<String> },
    Unsupported { kind: String },
}

impl FrameKind {
    /// Code blocks and HTML blocks keep their contents verbatim rather than
    /// as inline nodes: what is inside them is not Markdown.
    fn collects_raw_text(&self) -> bool {
        matches!(self, FrameKind::Code { .. } | FrameKind::HtmlBlock)
    }

    fn from_tag(tag: Tag<'_>) -> Self {
        match tag {
            Tag::Paragraph => FrameKind::Paragraph,
            Tag::HtmlBlock => FrameKind::HtmlBlock,
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
                ..
            } => FrameKind::Heading {
                level: heading_level(level),
                attributes: Attributes {
                    id: id.map(CowStr::into_string),
                    classes: classes.into_iter().map(CowStr::into_string).collect(),
                    pairs: attrs
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                key.into_string(),
                                value.map(CowStr::into_string).unwrap_or_default(),
                            )
                        })
                        .collect(),
                    // The parser does not report where the attribute block
                    // itself sat; Wave 1's own attribute pass will.
                    span: None,
                },
            },
            Tag::BlockQuote(_) => FrameKind::Quote,
            Tag::List(start) => FrameKind::List { start },
            Tag::Item => FrameKind::Item,
            Tag::CodeBlock(kind) => FrameKind::Code {
                language: match kind {
                    CodeBlockKind::Fenced(info) => {
                        let info = info.trim().to_string();
                        (!info.is_empty()).then_some(info)
                    }
                    CodeBlockKind::Indented => None,
                },
            },
            Tag::Emphasis => FrameKind::Emphasis,
            Tag::Strong => FrameKind::Strong,
            Tag::Link {
                dest_url, title, ..
            } => FrameKind::Link {
                url: dest_url.into_string(),
                title: non_empty(title),
            },
            Tag::Image {
                dest_url, title, ..
            } => FrameKind::Image {
                url: dest_url.into_string(),
                title: non_empty(title),
            },
            other => FrameKind::Unsupported {
                kind: tag_name(&other).to_string(),
            },
        }
    }
}

fn tag_name(tag: &Tag<'_>) -> &'static str {
    match tag.to_end() {
        TagEnd::Table => "table",
        TagEnd::TableHead => "table-head",
        TagEnd::TableRow => "table-row",
        TagEnd::TableCell => "table-cell",
        TagEnd::FootnoteDefinition => "footnote-definition",
        TagEnd::Strikethrough => "strikethrough",
        TagEnd::DefinitionList => "definition-list",
        TagEnd::MetadataBlock(_) => "metadata-block",
        _ => "unknown",
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn non_empty(text: CowStr<'_>) -> Option<String> {
    let text = text.into_string();
    (!text.is_empty()).then_some(text)
}

struct Frame {
    kind: FrameKind,
    span: Span,
    blocks: Vec<Block>,
    items: Vec<ListItem>,
    inlines: Vec<Inline>,
    text: String,
}

impl Frame {
    fn new(kind: FrameKind, span: Span) -> Self {
        Self {
            kind,
            span,
            blocks: Vec::new(),
            items: Vec::new(),
            inlines: Vec::new(),
            text: String::new(),
        }
    }

    /// `end_span` is the range the parser reported on the closing event; it
    /// covers the whole construct, and is the one we keep.
    fn finish(self, end_span: Span) -> Built {
        let span = self.span.join(end_span);
        match self.kind {
            FrameKind::Root => Built::Block(Block::Unsupported {
                span,
                kind: "root".to_string(),
            }),
            FrameKind::Paragraph => Built::Block(Block::Paragraph(Paragraph {
                inlines: self.inlines,
                attributes: Attributes::default(),
                span,
            })),
            FrameKind::Heading { level, attributes } => Built::Block(Block::Heading(Heading {
                level,
                inlines: self.inlines,
                attributes,
                span,
            })),
            FrameKind::Quote => Built::Block(Block::Quote(Quote {
                blocks: self.blocks,
                attributes: Attributes::default(),
                span,
            })),
            FrameKind::List { start } => Built::Block(Block::List(List {
                start,
                items: self.items,
                span,
            })),
            FrameKind::Item => {
                let mut blocks = self.blocks;
                // A *tight* list item ("- one") has no paragraph of its own:
                // pulldown-cmark sends its text straight to the item, so the
                // inlines land here rather than in a Paragraph frame.
                // Without this they would be dropped, and the item's text
                // would silently vanish from the document.
                if !self.inlines.is_empty() {
                    let first = self.inlines.first().map(Inline::span).unwrap_or(span);
                    let last = self.inlines.last().map(Inline::span).unwrap_or(span);
                    blocks.insert(
                        0,
                        Block::Paragraph(Paragraph {
                            inlines: self.inlines,
                            attributes: Attributes::default(),
                            span: first.join(last),
                        }),
                    );
                }
                Built::Item(ListItem { blocks, span })
            }
            FrameKind::Code { language } => Built::Block(Block::Code(CodeBlock {
                language,
                code: self.text,
                span,
            })),
            FrameKind::HtmlBlock => Built::Block(Block::Html {
                span,
                value: self.text,
            }),
            FrameKind::Emphasis => Built::Inline(Inline::Emphasis {
                span,
                children: self.inlines,
            }),
            FrameKind::Strong => Built::Inline(Inline::Strong {
                span,
                children: self.inlines,
            }),
            FrameKind::Link { url, title } => Built::Inline(Inline::Link(Link {
                url,
                title,
                children: self.inlines,
                attributes: Attributes::default(),
                span,
            })),
            FrameKind::Image { url, title } => Built::Inline(Inline::Image(Image {
                url,
                title,
                alt: self.inlines,
                attributes: Attributes::default(),
                span,
            })),
            FrameKind::Unsupported { kind } => Built::Block(Block::Unsupported { span, kind }),
        }
    }
}

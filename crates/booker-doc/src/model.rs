//! The document model.
//!
//! Small on purpose: Wave 0 needs headings, paragraphs, emphasis, lists,
//! quotes, images and code blocks. The invariant that must not be given up
//! as it grows is that **every node carries the span it was parsed from**.

use crate::span::Span;

/// A parsed Markdown file.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// The whole file.
    pub span: Span,
}

/// Booker's `{#id .class key=value}` attribute block.
///
/// Wave 0 fills this in only where the parser hands it to us for free
/// (headings). Wave 1 adds the attribute pass for images, spans and fenced
/// divs — which is why every node that will be able to carry attributes has
/// the field now, rather than the model having to change shape later.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attributes {
    pub id: Option<String>,
    pub classes: Vec<String>,
    /// `key=value` pairs, in the order written.
    pub pairs: Vec<(String, String)>,
    /// Where the attribute block itself was written, when there was one.
    pub span: Option<Span>,
}

impl Attributes {
    pub fn is_empty(&self) -> bool {
        self.id.is_none() && self.classes.is_empty() && self.pairs.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading(Heading),
    Paragraph(Paragraph),
    List(List),
    Quote(Quote),
    Code(CodeBlock),
    /// `---`
    ThematicBreak {
        span: Span,
    },
    /// A raw HTML block. Wave 0 keeps it verbatim with its span; the
    /// whitelisted subset (`PLAN.md` §5.3) is Wave 1's job.
    Html {
        span: Span,
        value: String,
    },
    /// A construct this build does not model yet — a table, a footnote
    /// definition. Kept, with its span, so nothing silently disappears from
    /// a document and a diagnostic can still point at it.
    Unsupported {
        span: Span,
        kind: String,
    },
}

impl Block {
    pub fn span(&self) -> Span {
        match self {
            Block::Heading(h) => h.span,
            Block::Paragraph(p) => p.span,
            Block::List(l) => l.span,
            Block::Quote(q) => q.span,
            Block::Code(c) => c.span,
            Block::ThematicBreak { span }
            | Block::Html { span, .. }
            | Block::Unsupported { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    pub inlines: Vec<Inline>,
    pub attributes: Attributes,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub inlines: Vec<Inline>,
    pub attributes: Attributes,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    /// `Some(n)` for an ordered list starting at `n`.
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
    pub span: Span,
}

impl List {
    pub fn is_ordered(&self) -> bool {
        self.start.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub blocks: Vec<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub blocks: Vec<Block>,
    pub attributes: Attributes,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    /// The info string of a fenced block: `rust`, `{=typst}`, or nothing.
    pub language: Option<String>,
    pub code: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text { span: Span, value: String },
    Emphasis { span: Span, children: Vec<Inline> },
    Strong { span: Span, children: Vec<Inline> },
    Code { span: Span, value: String },
    Link(Link),
    Image(Image),
    SoftBreak { span: Span },
    HardBreak { span: Span },
    Html { span: Span, value: String },
}

impl Inline {
    pub fn span(&self) -> Span {
        match self {
            Inline::Text { span, .. }
            | Inline::Emphasis { span, .. }
            | Inline::Strong { span, .. }
            | Inline::Code { span, .. }
            | Inline::SoftBreak { span }
            | Inline::HardBreak { span }
            | Inline::Html { span, .. } => *span,
            Inline::Link(link) => link.span,
            Inline::Image(image) => image.span,
        }
    }

    pub fn children(&self) -> &[Inline] {
        match self {
            Inline::Emphasis { children, .. } | Inline::Strong { children, .. } => children,
            Inline::Link(link) => &link.children,
            Inline::Image(image) => &image.alt,
            _ => &[],
        }
    }

    /// The plain text of this inline and everything inside it.
    pub fn text(&self) -> String {
        match self {
            Inline::Text { value, .. } | Inline::Code { value, .. } => value.clone(),
            Inline::SoftBreak { .. } | Inline::HardBreak { .. } => " ".to_string(),
            Inline::Html { .. } => String::new(),
            other => other.children().iter().map(Inline::text).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
    pub children: Vec<Inline>,
    pub attributes: Attributes,
    pub span: Span,
}

/// An image reference.
///
/// `url` is exactly what the author wrote, unresolved: resolving it against
/// the project root, and saying so when the file is not there, belongs to
/// whoever has the filesystem (`booker-project`), not to the parser.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub url: String,
    pub title: Option<String>,
    /// The alt text, as inlines, so its spans survive too.
    pub alt: Vec<Inline>,
    pub attributes: Attributes,
    pub span: Span,
}

impl Image {
    pub fn alt_text(&self) -> String {
        self.alt.iter().map(Inline::text).collect()
    }
}

/// A borrowed pointer at any node in the tree, for walking it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Node<'a> {
    Block(&'a Block),
    Item(&'a ListItem),
    Inline(&'a Inline),
}

impl Node<'_> {
    pub fn span(&self) -> Span {
        match self {
            Node::Block(block) => block.span(),
            Node::Item(item) => item.span,
            Node::Inline(inline) => inline.span(),
        }
    }

    /// A short name for messages and tests.
    pub fn kind(&self) -> &'static str {
        match self {
            Node::Item(_) => "list-item",
            Node::Block(block) => match block {
                Block::Heading(_) => "heading",
                Block::Paragraph(_) => "paragraph",
                Block::List(_) => "list",
                Block::Quote(_) => "quote",
                Block::Code(_) => "code",
                Block::ThematicBreak { .. } => "thematic-break",
                Block::Html { .. } => "html",
                Block::Unsupported { .. } => "unsupported",
            },
            Node::Inline(inline) => match inline {
                Inline::Text { .. } => "text",
                Inline::Emphasis { .. } => "emphasis",
                Inline::Strong { .. } => "strong",
                Inline::Code { .. } => "inline-code",
                Inline::Link(_) => "link",
                Inline::Image(_) => "image",
                Inline::SoftBreak { .. } => "soft-break",
                Inline::HardBreak { .. } => "hard-break",
                Inline::Html { .. } => "inline-html",
            },
        }
    }
}

impl Document {
    /// Visit every node, parents before children, in document order.
    pub fn walk<'a>(&'a self, visit: &mut impl FnMut(Node<'a>)) {
        for block in &self.blocks {
            walk_block(block, visit);
        }
    }

    /// Every image in the document, in order.
    pub fn images(&self) -> Vec<&Image> {
        let mut found = Vec::new();
        self.walk(&mut |node| {
            if let Node::Inline(Inline::Image(image)) = node {
                found.push(image);
            }
        });
        found
    }

    /// Every heading, in order — the outline a sidebar or a table of
    /// contents is built from.
    pub fn headings(&self) -> Vec<&Heading> {
        let mut found = Vec::new();
        self.walk(&mut |node| {
            if let Node::Block(Block::Heading(heading)) = node {
                found.push(heading);
            }
        });
        found
    }

    /// The document's plain text, used for word counts and search.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            block_text(block, &mut out);
        }
        out
    }

    pub fn word_count(&self) -> usize {
        self.text().split_whitespace().count()
    }
}

fn walk_block<'a>(block: &'a Block, visit: &mut impl FnMut(Node<'a>)) {
    visit(Node::Block(block));
    match block {
        Block::Heading(heading) => walk_inlines(&heading.inlines, visit),
        Block::Paragraph(paragraph) => walk_inlines(&paragraph.inlines, visit),
        Block::Quote(quote) => {
            for child in &quote.blocks {
                walk_block(child, visit);
            }
        }
        Block::List(list) => {
            for item in &list.items {
                visit(Node::Item(item));
                for child in &item.blocks {
                    walk_block(child, visit);
                }
            }
        }
        Block::Code(_)
        | Block::ThematicBreak { .. }
        | Block::Html { .. }
        | Block::Unsupported { .. } => {}
    }
}

fn walk_inlines<'a>(inlines: &'a [Inline], visit: &mut impl FnMut(Node<'a>)) {
    for inline in inlines {
        visit(Node::Inline(inline));
        walk_inlines(inline.children(), visit);
    }
}

fn block_text(block: &Block, out: &mut String) {
    match block {
        Block::Heading(heading) => push_inlines(&heading.inlines, out),
        Block::Paragraph(paragraph) => push_inlines(&paragraph.inlines, out),
        Block::Quote(quote) => {
            for child in &quote.blocks {
                block_text(child, out);
            }
        }
        Block::List(list) => {
            for item in &list.items {
                for child in &item.blocks {
                    block_text(child, out);
                }
            }
        }
        Block::Code(code) => {
            out.push_str(&code.code);
            out.push('\n');
        }
        Block::ThematicBreak { .. } | Block::Html { .. } | Block::Unsupported { .. } => {}
    }
}

fn push_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        out.push_str(&inline.text());
    }
    out.push('\n');
}

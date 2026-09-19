//! The one hard requirement of this crate: reliable source positions.
//!
//! These tests are the guard on it. If a future change to the model or the
//! parser loses a span, or moves one, they fail.

use booker_doc::{Block, Document, Inline, Node};

const BOOK: &str = "\
# The Garden {#garden .fancy}

Mia opened the *little* green door and found **everything** green,
even the `cat`.

> She had never seen
> a garden like it.

- a rake
- a watering can with *holes*
  - and a snail

1. first
2. second

![Tulips in May](assets/images/tulips.jpg \"May\")

```typst
#rect(width: 10mm)
```

---
";

fn document() -> Document {
    Document::parse(BOOK)
}

#[test]
fn every_node_carries_a_span_that_slices_back_out_of_the_source() {
    let document = document();
    let mut seen = 0;
    document.walk(&mut |node: Node<'_>| {
        seen += 1;
        let span = node.span();
        assert!(
            span.end <= BOOK.len(),
            "{} span {span:?} runs past the end of the file",
            node.kind()
        );
        assert!(
            !span.is_empty(),
            "{} has an empty span: {span:?}",
            node.kind()
        );
        assert!(
            span.slice(BOOK).is_some(),
            "{} span {span:?} does not land on character boundaries",
            node.kind()
        );
    });
    assert!(seen > 25, "the walk visited only {seen} nodes");
}

#[test]
fn a_childs_span_sits_inside_its_parents() {
    let document = document();
    for block in &document.blocks {
        let outer = block.span();
        let mut inner_spans = Vec::new();
        collect(block, &mut inner_spans);
        for span in inner_spans {
            assert!(
                span.start >= outer.start && span.end <= outer.end,
                "{span:?} is not inside {outer:?}"
            );
        }
    }

    fn collect(block: &Block, out: &mut Vec<booker_doc::Span>) {
        let document = Document {
            blocks: vec![block.clone()],
            span: block.span(),
        };
        document.walk(&mut |node| out.push(node.span()));
    }
}

#[test]
fn a_heading_keeps_its_level_its_text_and_its_attributes() {
    let document = document();
    let headings = document.headings();
    assert_eq!(headings.len(), 1);
    let heading = headings[0];
    assert_eq!(heading.level, 1);
    assert_eq!(
        heading.span.slice(BOOK),
        Some("# The Garden {#garden .fancy}\n")
    );
    // The attribute block is Wave 1 work, but the parser already gives us
    // the heading's; the model must not drop it.
    assert_eq!(heading.attributes.id.as_deref(), Some("garden"));
    assert_eq!(heading.attributes.classes, vec!["fancy".to_string()]);
    let text: String = heading.inlines.iter().map(Inline::text).collect();
    assert_eq!(text, "The Garden");
}

#[test]
fn emphasis_spans_cover_the_markers_and_the_inner_text_does_not() {
    let document = document();
    let Block::Paragraph(paragraph) = &document.blocks[1] else {
        panic!("expected the first paragraph, got {:?}", document.blocks[1]);
    };
    let emphasis = paragraph
        .inlines
        .iter()
        .find(|inline| matches!(inline, Inline::Emphasis { .. }))
        .expect("the paragraph has emphasis in it");
    assert_eq!(emphasis.span().slice(BOOK), Some("*little*"));
    let inner = emphasis.children()[0].span();
    assert_eq!(inner.slice(BOOK), Some("little"));

    let strong = paragraph
        .inlines
        .iter()
        .find(|inline| matches!(inline, Inline::Strong { .. }))
        .expect("the paragraph has strong text in it");
    assert_eq!(strong.span().slice(BOOK), Some("**everything**"));

    let code = paragraph
        .inlines
        .iter()
        .find(|inline| matches!(inline, Inline::Code { .. }))
        .expect("the paragraph has inline code in it");
    assert_eq!(code.span().slice(BOOK), Some("`cat`"));
    assert_eq!(code.text(), "cat");
}

#[test]
fn a_quote_keeps_its_markers_in_its_span_and_its_paragraphs_inside() {
    let document = document();
    let quote = document
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Quote(quote) => Some(quote),
            _ => None,
        })
        .expect("the document has a quote");
    let text = quote.span.slice(BOOK).expect("a valid span");
    assert!(text.starts_with("> She had never seen"), "{text:?}");
    assert_eq!(quote.blocks.len(), 1);
}

#[test]
fn lists_keep_their_items_their_nesting_and_their_numbering() {
    let document = document();
    let lists: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::List(list) => Some(list),
            _ => None,
        })
        .collect();
    assert_eq!(lists.len(), 2, "one bulleted list and one ordered list");

    let bulleted = lists[0];
    assert!(!bulleted.is_ordered());
    assert_eq!(bulleted.items.len(), 2);
    assert_eq!(bulleted.items[0].span.slice(BOOK), Some("- a rake\n"));
    let nested = bulleted.items[1]
        .blocks
        .iter()
        .any(|block| matches!(block, Block::List(_)));
    assert!(nested, "the second item contains a nested list");

    let ordered = lists[1];
    assert_eq!(ordered.start, Some(1));
    assert_eq!(ordered.items.len(), 2);
}

#[test]
fn an_image_keeps_the_url_the_author_wrote_and_its_alt_text() {
    let document = document();
    let images = document.images();
    assert_eq!(images.len(), 1);
    let image = images[0];
    assert_eq!(image.url, "assets/images/tulips.jpg");
    assert_eq!(image.alt_text(), "Tulips in May");
    assert_eq!(image.title.as_deref(), Some("May"));
    assert_eq!(
        image.span.slice(BOOK),
        Some("![Tulips in May](assets/images/tulips.jpg \"May\")")
    );
}

#[test]
fn a_code_block_keeps_its_language_and_its_contents_verbatim() {
    let document = document();
    let code = document
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Code(code) => Some(code),
            _ => None,
        })
        .expect("the document has a fenced block");
    assert_eq!(code.language.as_deref(), Some("typst"));
    assert_eq!(code.code, "#rect(width: 10mm)\n");
    assert!(code
        .span
        .slice(BOOK)
        .expect("a valid span")
        .starts_with("```typst"));
}

#[test]
fn a_thematic_break_is_kept() {
    let document = document();
    assert!(document
        .blocks
        .iter()
        .any(|block| matches!(block, Block::ThematicBreak { .. })));
}

#[test]
fn spans_survive_multi_byte_characters() {
    // The failure this guards against: counting bytes as characters, which
    // puts every diagnostic after an accented letter in the wrong column.
    let source = "Grand-mère écrit « au jardin » — *toujours*.\n";
    let document = Document::parse(source);
    let Block::Paragraph(paragraph) = &document.blocks[0] else {
        panic!("expected a paragraph");
    };
    let emphasis = paragraph
        .inlines
        .iter()
        .find(|inline| matches!(inline, Inline::Emphasis { .. }))
        .expect("emphasis");
    assert_eq!(emphasis.span().slice(source), Some("*toujours*"));

    let location = document.location(source, "content/01.md", emphasis.span());
    assert_eq!(location.line, 1);
    // 33 characters precede the `*`, though many more bytes do.
    assert_eq!(location.column, 34);
    assert!(
        emphasis.span().start > 33,
        "bytes outnumber characters here"
    );
}

#[test]
fn a_location_points_at_the_line_the_author_would_look_at() {
    let document = document();
    let images = document.images();
    let location = document.location(BOOK, "content/01-the-garden.md", images[0].span);
    assert_eq!(location.file.to_str(), Some("content/01-the-garden.md"));
    assert_eq!(location.line, 16);
    assert_eq!(location.column, 1);
    let (start, end) = location.span.expect("the byte range is kept as well");
    assert_eq!(
        &BOOK[start..end],
        "![Tulips in May](assets/images/tulips.jpg \"May\")"
    );
}

#[test]
fn an_empty_document_is_a_document() {
    let document = Document::parse("");
    assert!(document.blocks.is_empty());
    assert_eq!(document.word_count(), 0);
}

#[test]
fn unusual_input_does_not_panic_and_does_not_lose_content() {
    // Tables and footnotes are not modelled in Wave 0. They must still be
    // kept, with a span, rather than vanishing from the document.
    let source = "| a | b |\n|---|---|\n| 1 | 2 |\n\nText[^1]\n\n[^1]: note\n";
    let document = Document::parse(source);
    assert!(!document.blocks.is_empty());
    document.walk(&mut |node| {
        assert!(node.span().slice(source).is_some(), "{:?}", node.span());
    });

    let messy = "*unclosed emphasis\n\n> quote with a [broken](link\n\n    indented code\n";
    let document = Document::parse(messy);
    assert!(!document.blocks.is_empty());
    document.walk(&mut |node| {
        assert!(node.span().slice(messy).is_some());
    });
}

#[test]
fn word_count_covers_the_text_and_not_the_markup() {
    let document = Document::parse("# Title\n\nOne *two* three.\n");
    assert_eq!(document.word_count(), 4);
}

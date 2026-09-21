//! The Markdown Booker understands from Wave 2 (`docs/waves/wave-2.md`,
//! contract 2): attributes on images and `[spans]`, `:::` divs, page breaks,
//! strikethrough and tables — each with a span that slices the author's own
//! text back out of the file.

use booker_doc::{Alignment, Block, Document, Inline, Node};

const BOOK: &str = "\
# The Garden {#garden break-before=page}

She said it was [very old]{.whisper #old} and [quite *green*]{.x} too.

![Tulips in May](assets/images/tulips.jpg){#tulips width=60%}

::: {.poem break-before=page}
Roses are red,
violets are ~~blue~~ green.

::: stanza
Inner.
:::
:::

::: page-break
:::

| Flower | Colour |
|:-------|-------:|
| rose   | red    |

A note.[^1] Braces {like these} and \\[escaped]{.no} stay text.

[^1]: The note itself.

```
::: not a div
```
";

fn document() -> Document {
    Document::parse(BOOK)
}

fn slice(span: booker_doc::Span) -> &'static str {
    span.slice(BOOK).expect("span on character boundaries")
}

#[test]
fn every_new_node_slices_back_out_of_the_source() {
    let document = document();
    document.walk(&mut |node: Node<'_>| {
        let span = node.span();
        assert!(
            span.slice(BOOK).is_some(),
            "{} span {span:?} does not slice the source",
            node.kind()
        );
    });
}

#[test]
fn heading_attributes_carry_the_span_of_their_braces() {
    let document = document();
    let heading = document.headings()[0];
    assert_eq!(heading.attributes.id.as_deref(), Some("garden"));
    assert_eq!(heading.attributes.get("break-before"), Some("page"));
    let braces = heading.attributes.span.expect("the braces are located");
    assert_eq!(slice(braces), "{#garden break-before=page}");
}

#[test]
fn a_bracketed_span_becomes_a_span_with_its_attributes() {
    let document = document();
    let Block::Paragraph(paragraph) = &document.blocks[1] else {
        panic!("second block is the paragraph: {:?}", document.blocks[1]);
    };
    let spans: Vec<_> = paragraph
        .inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::Span(span) => Some(span),
            _ => None,
        })
        .collect();
    assert_eq!(spans.len(), 2, "{:#?}", paragraph.inlines);
    let span = spans[0];
    assert_eq!(slice(span.span), "[very old]{.whisper #old}");
    assert_eq!(span.attributes.classes, ["whisper"]);
    assert_eq!(span.attributes.id.as_deref(), Some("old"));
    assert_eq!(slice(span.attributes.span.unwrap()), "{.whisper #old}");
    let text: String = span.children.iter().map(Inline::text).collect();
    assert_eq!(text, "very old");
}

#[test]
fn text_around_a_span_is_kept_whole() {
    let document = document();
    let Block::Paragraph(paragraph) = &document.blocks[1] else {
        panic!()
    };
    let text: String = paragraph.inlines.iter().map(Inline::text).collect();
    assert_eq!(
        text, "She said it was very old and quite green too.",
        "not a letter lost or added: {:#?}",
        paragraph.inlines
    );
}

#[test]
fn image_attributes_are_read_and_the_span_covers_them() {
    let document = document();
    let image = document.images()[0];
    assert_eq!(image.attributes.id.as_deref(), Some("tulips"));
    assert_eq!(image.attributes.get("width"), Some("60%"));
    assert_eq!(
        slice(image.span),
        "![Tulips in May](assets/images/tulips.jpg){#tulips width=60%}"
    );
}

#[test]
fn divs_group_their_blocks_and_nest() {
    let document = document();
    let divs: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Div(div) => Some(div),
            _ => None,
        })
        .collect();
    assert_eq!(divs.len(), 2, "the poem and the page break");

    let poem = divs[0];
    assert!(poem.has_class("poem"));
    assert_eq!(poem.attributes.get("break-before"), Some("page"));
    assert!(slice(poem.span).starts_with("::: {.poem break-before=page}"));
    assert!(slice(poem.span).ends_with(":::"));
    assert_eq!(poem.blocks.len(), 2, "a paragraph and the inner div");
    let Block::Div(stanza) = &poem.blocks[1] else {
        panic!("{:#?}", poem.blocks);
    };
    assert!(stanza.has_class("stanza"));
    assert_eq!(stanza.blocks.len(), 1);

    let page_break = divs[1];
    assert!(page_break.has_class("page-break"));
    assert!(page_break.blocks.is_empty());
}

#[test]
fn a_fence_line_is_not_part_of_the_paragraph_after_it() {
    let document = document();
    let Block::Div(poem) = document
        .blocks
        .iter()
        .find(|b| matches!(b, Block::Div(_)))
        .unwrap()
    else {
        unreachable!()
    };
    let Block::Paragraph(first) = &poem.blocks[0] else {
        panic!()
    };
    assert!(slice(first.span).starts_with("Roses are red"));
}

#[test]
fn strikethrough_is_an_inline() {
    let mut found = None;
    document().walk(&mut |node| {
        if let Node::Inline(inline @ Inline::Strikethrough { .. }) = node {
            found = Some(inline.span());
        }
    });
    assert_eq!(slice(found.expect("a strikethrough")), "~~blue~~");
}

#[test]
fn a_table_keeps_its_alignment_head_and_rows() {
    let document = document();
    let table = document
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Table(table) => Some(table),
            _ => None,
        })
        .expect("a table");
    assert_eq!(table.alignments, [Alignment::Left, Alignment::Right]);
    let head: Vec<String> = table
        .head
        .iter()
        .map(|cell| cell.inlines.iter().map(Inline::text).collect())
        .collect();
    assert_eq!(head, ["Flower", "Colour"]);
    assert_eq!(table.rows.len(), 1);
    assert!(slice(table.span).starts_with("| Flower"));
}

#[test]
fn a_footnote_reference_is_kept_rather_than_lost() {
    let mut found = Vec::new();
    document().walk(&mut |node| {
        if let Node::Inline(Inline::Unsupported { kind, span }) = node {
            found.push((kind.clone(), slice(*span)));
        }
    });
    assert_eq!(
        found,
        [("footnote-reference".to_string(), "[^1]")],
        "reported, so BK-DOC-002 can point at it"
    );
}

#[test]
fn prose_braces_escaped_brackets_and_code_stay_as_written() {
    let document = document();
    assert!(
        document
            .text()
            .contains("Braces {like these} and [escaped]{.no} stay text."),
        "{}",
        document.text()
    );
    let code = document
        .blocks
        .iter()
        .find_map(|block| match block {
            Block::Code(code) => Some(code),
            _ => None,
        })
        .expect("the code block");
    assert_eq!(code.code, "::: not a div\n");
}

#[test]
fn an_unclosed_div_keeps_everything_to_the_end() {
    let source = "Before.\n\n::: {.poem}\nOne.\n\nTwo.\n";
    let document = Document::parse(source);
    let Block::Div(div) = &document.blocks[1] else {
        panic!("{:#?}", document.blocks);
    };
    assert_eq!(div.blocks.len(), 2);
    assert_eq!(div.span.end, source.len());
    assert_eq!(document.word_count(), 3);
}

#[test]
fn a_bare_closing_fence_with_nothing_open_is_text() {
    let document = Document::parse("Hello.\n\n:::\n");
    assert_eq!(document.blocks.len(), 2);
    assert!(document.text().contains(":::"));
}

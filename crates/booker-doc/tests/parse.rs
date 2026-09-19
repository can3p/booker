use booker_doc::{Block, Document, Inline};

#[test]
fn a_tight_list_item_keeps_its_text() {
    // Regression: pulldown-cmark gives a tight item's text directly to the
    // item, with no paragraph around it. Dropping those inlines emptied
    // every bullet in every book.
    let document = Document::parse("- one\n- two\n");
    let Some(Block::List(list)) = document.blocks.first() else {
        panic!("expected a list, got {:?}", document.blocks);
    };
    assert_eq!(list.items.len(), 2);
    for (item, expected) in list.items.iter().zip(["one", "two"]) {
        let text: String = item
            .blocks
            .iter()
            .map(|block| match block {
                Block::Paragraph(paragraph) => paragraph
                    .inlines
                    .iter()
                    .map(Inline::text)
                    .collect::<String>(),
                _ => String::new(),
            })
            .collect();
        assert_eq!(text, expected, "item lost its text");
    }
}

#[test]
fn a_loose_list_item_is_not_doubled() {
    let document = Document::parse("- one\n\n- two\n");
    let Some(Block::List(list)) = document.blocks.first() else {
        panic!("expected a list");
    };
    for item in &list.items {
        assert_eq!(
            item.blocks.len(),
            1,
            "expected exactly one paragraph per item"
        );
    }
}

#[test]
fn deliberately_failing_check_that_ci_notices() {
    assert_eq!(2 + 2, 5, "this test exists to prove the test job is wired up");
}

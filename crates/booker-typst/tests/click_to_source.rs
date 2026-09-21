//! From a page back to the Markdown, and from the Markdown to a page — the
//! engine half of click-to-source (`docs/waves/wave-2.md`, contract 3).

use booker_core::{
    BookConfig, CompileRequest, CompileTarget, Length, PagePoint, ProjectRef, Revision,
};
use booker_doc::Document;
use booker_typst::Engine;

const ONE: &str = "# One\n\nThe first chapter has a single sentence about a jar.\n";
const TWO: &str = "# Two\n\nThe second chapter is about the garden.\n";

fn laid_out() -> (tempfile::TempDir, Engine) {
    let folder = tempfile::tempdir().unwrap();
    let project = ProjectRef::new(folder.path());
    let config: BookConfig =
        toml::from_str("title = \"T\"\n[chapter]\nstart = \"new-page\"\n[toc]\nenabled = false\n")
            .unwrap();
    let one = Document::parse(ONE);
    let two = Document::parse(TWO);
    let mut engine = Engine::open(project.clone()).unwrap();
    engine
        .set_book(&config, &[(ONE, &one), (TWO, &two)])
        .unwrap();
    let compiled = engine
        .compile(&CompileRequest {
            project,
            target: CompileTarget::Layout,
            revision: Revision(0),
        })
        .unwrap();
    assert_eq!(
        compiled.result.pages.len(),
        2,
        "{:?}",
        compiled.result.diagnostics
    );
    (folder, engine)
}

#[test]
fn a_sentence_lands_on_its_chapters_page() {
    let (_folder, engine) = laid_out();
    let garden = TWO.find("garden").unwrap();
    let points = engine.pages_at(1, garden);
    assert_eq!(points.len(), 1, "{points:?}");
    assert_eq!(
        points[0].page, 1,
        "the second chapter starts on the second page"
    );

    let jar = ONE.find("jar").unwrap();
    assert_eq!(engine.pages_at(0, jar)[0].page, 0);
}

/// `pages_at` points at the start of the run of text a word is in — that is
/// the precision `typst-ide` gives — so the round trip lands on the start of
/// the sentence.
#[test]
fn a_click_where_text_was_drawn_leads_back_to_that_text() {
    let (_folder, engine) = laid_out();
    let sentence = TWO.find("The second").unwrap();
    let point = engine.pages_at(1, TWO.find("garden").unwrap()).remove(0);
    // A hair to the right and above the baseline, inside the glyphs.
    let click = PagePoint {
        page: point.page,
        x: Length::mm(point.x.to_mm() + 0.5),
        y: Length::mm(point.y.to_mm() - 1.0),
    };
    let (chapter, offset) = engine.source_at(&click).expect("a word is under the click");
    assert_eq!(chapter, 1);
    assert!(
        (sentence..sentence + 3).contains(&offset),
        "offset {offset} is not at the start of the sentence ({sentence}): {:?}",
        &TWO[offset.saturating_sub(5)..(offset + 5).min(TWO.len())]
    );
}

#[test]
fn a_page_says_which_chapter_it_shows() {
    let (_folder, engine) = laid_out();
    let first = engine.page_sources(0).unwrap();
    assert!(!first.is_empty());
    assert!(first.iter().all(|(chapter, _)| *chapter == 0), "{first:?}");
    let second = engine.page_sources(1).unwrap();
    assert!(
        second.iter().all(|(chapter, _)| *chapter == 1),
        "{second:?}"
    );
    assert!(engine.page_sources(2).is_none(), "there is no third page");
}

#[test]
fn a_click_in_the_margin_leads_nowhere() {
    let (_folder, engine) = laid_out();
    let margin = PagePoint {
        page: 0,
        x: Length::mm(2.0),
        y: Length::mm(2.0),
    };
    assert_eq!(engine.source_at(&margin), None);
}

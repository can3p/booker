//! Compiling a project: pages, a PDF, and what happens when the book is broken.

mod support;

use booker_core::{CompileTarget, Severity};
use booker_typst::{Engine, PROJECT_FONT_DIR};
use support::{compile_request, fixture, temp_project};

#[test]
fn the_fixture_lays_out_into_the_pages_it_should() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("the fixture to compile");

    assert!(
        compiled.result.diagnostics.is_empty(),
        "the fixture should be clean: {:?}",
        compiled.result.diagnostics
    );
    assert_eq!(
        compiled.result.pages.len(),
        4,
        "title page, then three chapters, each after an explicit page break"
    );

    for (index, page) in compiled.result.pages.iter().enumerate() {
        assert_eq!(page.index as usize, index);
        assert!(
            (page.width.to_mm() - 148.0).abs() < 0.01,
            "page {index} is {} wide, expected A5",
            page.width
        );
        assert!(
            (page.height.to_mm() - 210.0).abs() < 0.01,
            "page {index} is {} tall, expected A5",
            page.height
        );
    }

    let labels: Vec<_> = compiled
        .result
        .pages
        .iter()
        .map(|page| page.label.clone())
        .collect();
    assert_eq!(
        labels,
        vec![
            Some("1".to_string()),
            Some("2".to_string()),
            Some("3".to_string()),
            Some("4".to_string())
        ],
        "the fixture numbers its pages with `numbering: \"1\"`"
    );

    assert_eq!(compiled.pdf, None, "a layout request writes no file");
}

#[test]
fn the_fixture_exports_a_pdf() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Pdf))
        .expect("the fixture to compile");

    let pdf = compiled.pdf.expect("a PDF");
    assert!(pdf.starts_with(b"%PDF-"), "not a PDF file");
    assert!(
        pdf.ends_with(b"%%EOF") || pdf.ends_with(b"%%EOF\n"),
        "the PDF was truncated"
    );
    assert!(
        pdf.len() > 4_000,
        "a four-page PDF with embedded fonts should not be {} bytes",
        pdf.len()
    );
    assert_eq!(compiled.result.pages.len(), 4, "pages come back either way");
}

#[test]
fn a_page_size_in_inches_survives_the_trip() {
    let (_dir, project) = temp_project(&[(
        "main.typ",
        "#set page(width: 6in, height: 9in)\nA paperback page.\n",
    )]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");

    let page = &compiled.result.pages[0];
    assert!((page.width.to_mm() - 152.4).abs() < 0.01, "{}", page.width);
    assert!(
        (page.height.to_mm() - 228.6).abs() < 0.01,
        "{}",
        page.height
    );
}

#[test]
fn a_broken_book_produces_a_located_diagnostic_rather_than_a_panic() {
    let (_dir, project) = temp_project(&[(
        "main.typ",
        "= A chapter\n\nThis line is fine.\n\n#let x = undefined_thing\n",
    )]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("a broken book is diagnostics, not an Err");

    assert!(compiled.has_errors(), "{:?}", compiled.result.diagnostics);
    assert!(
        compiled.result.pages.is_empty(),
        "nothing laid out, so there are no pages to show"
    );

    let error = compiled
        .result
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Error)
        .expect("an error");
    assert_eq!(error.rule, "BK-TYPST-001");
    let source = error.source.as_ref().expect("a source location");
    assert_eq!(source.file.to_str(), Some("main.typ"));
    assert_eq!(source.line, 5, "the line the undefined name is written on");
    assert_eq!(source.column, 10);
}

#[test]
fn an_error_in_an_included_file_points_at_that_file() {
    let (_dir, project) = temp_project(&[
        ("main.typ", "= Book\n\n#include \"content/one.typ\"\n"),
        ("content/one.typ", "First line.\n\n#(1 + \"two\")\n"),
    ]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");

    let error = compiled
        .result
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Error)
        .expect("an error");
    let source = error.source.as_ref().expect("a source location");
    assert_eq!(
        source.file.to_str(),
        Some("content/one.typ"),
        "the diagnostic must name the file the author wrote, not the entry point"
    );
    assert_eq!(source.line, 3);
}

#[test]
fn a_missing_entry_file_is_a_diagnostic_and_the_project_still_opens() {
    let (_dir, project) = temp_project(&[("book.toml", "title = \"Mia\"\n")]);

    let mut engine = Engine::open(project.clone()).expect("an empty folder still opens");
    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("a missing file is diagnostics, not an Err");

    let error = compiled
        .result
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Error)
        .expect("an error");
    assert!(
        error.message.contains("main.typ") || error.message.contains("not found"),
        "the message must say which file is missing: {}",
        error.message
    );
}

#[test]
fn a_book_cannot_read_a_file_outside_its_own_folder() {
    let outside = tempfile::tempdir().expect("a directory outside the project");
    std::fs::write(outside.path().join("secret.typ"), "= Not yours\n").expect("to write it");

    let (_dir, project) = temp_project(&[(
        "main.typ",
        "#include \"../secret.typ\"\n#include \"/etc/passwd\"\n",
    )]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");

    assert!(
        compiled.has_errors(),
        "reading outside the project must fail: {:?}",
        compiled.result.diagnostics
    );
}

#[test]
fn an_unsaved_buffer_wins_over_the_file_on_disk_until_it_is_dropped() {
    let (_dir, project) = temp_project(&[("main.typ", "On disk.\n")]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let on_disk = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    assert_eq!(on_disk.result.pages.len(), 1);

    engine
        .set_source("main.typ", "Unsaved.\n\n#pagebreak()\n\nStill unsaved.\n")
        .expect("to set the buffer");
    let unsaved = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    assert_eq!(
        unsaved.result.pages.len(),
        2,
        "the editor's text should be what gets laid out"
    );

    assert!(engine.clear_source("main.typ").expect("a valid path"));
    let again = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    assert_eq!(
        again.result.pages.len(),
        1,
        "dropping the buffer falls back to the file on disk"
    );
}

#[test]
fn a_file_rewritten_from_outside_is_picked_up_on_the_next_compile() {
    let (dir, project) = temp_project(&[("main.typ", "One page.\n")]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let before = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    assert_eq!(before.result.pages.len(), 1);

    // An agent, or the user's own editor, rewrites the file underneath us.
    std::fs::write(
        dir.path().join("main.typ"),
        "One page.\n\n#pagebreak()\n\nTwo pages.\n",
    )
    .expect("to rewrite the file");

    let after = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    assert_eq!(
        after.result.pages.len(),
        2,
        "the engine must never serve a cached copy of a file that changed"
    );
}

#[test]
fn two_projects_open_at_once_do_not_bleed_into_each_other() {
    // Typst identifies a file by its path *inside* a project, so `main.typ`
    // in two different books has the same file id, and the memoized layout
    // cache is global to the process. Nothing may leak between them.
    let (_one_dir, one) = temp_project(&[("main.typ", "Book one.\n")]);
    let (_two_dir, two) = temp_project(&[(
        "main.typ",
        "Book two.\n\n#pagebreak()\n\nAnd its second page.\n",
    )]);

    let mut engine_one = Engine::open(one.clone()).expect("to open book one");
    let mut engine_two = Engine::open(two.clone()).expect("to open book two");

    for _ in 0..3 {
        let first = engine_one
            .compile(&compile_request(&one, CompileTarget::Layout))
            .expect("to compile book one");
        let second = engine_two
            .compile(&compile_request(&two, CompileTarget::Layout))
            .expect("to compile book two");
        assert_eq!(first.result.pages.len(), 1, "book one is one page");
        assert_eq!(second.result.pages.len(), 2, "book two is two pages");
    }
}

#[test]
fn a_request_for_another_project_is_refused() {
    let (_dir, project) = temp_project(&[("main.typ", "Hello.\n")]);
    let (_other_dir, other) = temp_project(&[("main.typ", "Hello.\n")]);
    let mut engine = Engine::open(project).expect("to open the project");

    let err = engine
        .compile(&compile_request(&other, CompileTarget::Layout))
        .expect_err("an engine answers for one project only");
    assert!(err.to_string().contains("engine per project"), "{err}");
}

#[test]
fn a_project_font_is_loaded_and_an_unreadable_one_is_reported() {
    let bundled =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts/LibertinusSerif-Regular.otf");
    let (dir, project) = temp_project(&[
        ("main.typ", "Hello.\n"),
        (
            &format!("{PROJECT_FONT_DIR}/broken.ttf"),
            "this is not a font",
        ),
    ]);
    std::fs::copy(
        &bundled,
        dir.path()
            .join(PROJECT_FONT_DIR)
            .join("Project-Regular.otf"),
    )
    .expect("to place a font in the project");

    let mut engine = Engine::open(project.clone()).expect("to open the project");
    assert_eq!(
        engine.fonts().project_face_count(),
        1,
        "the readable project font should be loaded"
    );

    let compiled = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("a bad font file must not stop the book");
    let warning = compiled
        .result
        .diagnostics
        .iter()
        .find(|d| d.rule == "BK-FONT-001")
        .expect("a warning about the unreadable font");
    assert_eq!(warning.severity, Severity::Warning);
    assert!(
        warning.message.contains("broken.ttf"),
        "the warning must name the file: {}",
        warning.message
    );
    assert_eq!(
        warning.source.as_ref().map(|s| s.file.to_str()),
        Some(Some("assets/fonts/broken.ttf"))
    );
    assert_eq!(compiled.result.pages.len(), 1, "the book still laid out");
}

#[test]
fn the_compile_reports_how_long_it_took_and_which_revision_it_answered() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let mut request = compile_request(&project, CompileTarget::Layout);
    request.revision = booker_core::Revision(42);

    let compiled = engine.compile(&request).expect("to compile");
    assert_eq!(
        compiled.result.revision,
        booker_core::Revision(42),
        "a caller must be able to tell which question this answers"
    );
    assert!(compiled.result.duration_ms < 60_000, "sanity");
}

//! A broken project must still open, and every way it is broken must be
//! named and located (`AGENTS.md` §7, `PLAN.md` §11.1 and §11.2).
//!
//! `fixtures/broken` is the project this is built on, and it is deliberately
//! broken in three different ways at once. It is meant to stay broken.

use std::path::{Path, PathBuf};

use booker_project::{rules, Diagnostic, Project, Severity};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

fn broken() -> Project {
    Project::load(fixture("broken")).expect("a broken project still opens")
}

fn find<'a>(project: &'a Project, rule: &str) -> Vec<&'a Diagnostic> {
    project
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.rule == rule)
        .collect()
}

#[test]
fn the_broken_fixture_opens_and_loads_what_it_can() {
    let project = broken();
    assert_eq!(project.config().title, "The Broken Book");
    assert_eq!(
        project.chapters().len(),
        1,
        "the chapter that exists is loaded even though its neighbour is missing"
    );
    assert_eq!(project.chapters()[0].title().as_deref(), Some("The Garden"));
    assert!(project.has_errors(), "and it is honest about being broken");
}

#[test]
fn a_chapter_listed_but_absent_points_at_the_line_that_lists_it() {
    let project = broken();
    let found = find(&project, rules::MISSING_CHAPTER);
    assert_eq!(found.len(), 1, "{found:#?}");
    let diagnostic = found[0];
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(
        diagnostic
            .message
            .contains("content/02-the-missing-chapter.md"),
        "{}",
        diagnostic.message
    );
    let source = diagnostic.source.as_ref().expect("a source location");
    assert_eq!(source.file, Path::new("book.toml"));
    assert_eq!(
        (source.line, source.column),
        (11, 5),
        "it points at the entry in book.toml, not at the file that is not there"
    );
}

#[test]
fn a_missing_image_points_at_the_line_in_the_chapter() {
    let project = broken();
    let found = find(&project, rules::MISSING_IMAGE);
    assert_eq!(found.len(), 1, "{found:#?}");
    let diagnostic = found[0];
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "image `assets/images/tulips.jpg` does not exist"
    );
    let source = diagnostic.source.as_ref().expect("a source location");
    assert_eq!(source.file, Path::new("content/01-the-garden.md"));
    assert_eq!((source.line, source.column), (6, 1));
    let (start, end) = source.span.expect("the byte range too");
    let text = std::fs::read_to_string(fixture("broken").join("content/01-the-garden.md"))
        .expect("read the chapter");
    assert_eq!(
        &text[start..end],
        "![Tulips in May](assets/images/tulips.jpg)"
    );
}

#[test]
fn a_value_that_makes_no_sense_costs_that_value_and_nothing_else() {
    let project = broken();
    let found = find(&project, rules::BAD_VALUE);
    assert_eq!(found.len(), 2, "{found:#?}");

    let facing = found
        .iter()
        .find(|diagnostic| diagnostic.message.contains("facing"))
        .expect("the `facing = \"yes\"` problem");
    assert!(
        facing.message.contains("should be true or false"),
        "{}",
        facing.message
    );
    let source = facing.source.as_ref().expect("a source location");
    assert_eq!((source.line, source.column), (16, 10));

    let bleed = found
        .iter()
        .find(|diagnostic| diagnostic.message.contains("bleed"))
        .expect("the `bleed = \"3 furlongs\"` problem");
    assert!(
        bleed.message.contains("mm, cm, in, pt and px"),
        "the message says what it would have accepted: {}",
        bleed.message
    );
    assert_eq!(bleed.source.as_ref().map(|source| source.line), Some(17));

    // The defaults stand in for the values that could not be read, and the
    // values around them are unaffected.
    assert!(project.config().page.facing, "the default is used instead");
    assert_eq!(project.config().page.bleed, None);
    assert_eq!(
        project.config().page.margins.bottom.to_string(),
        "20mm",
        "the margins next to the broken values are read normally"
    );
}

#[test]
fn a_typo_in_a_key_suggests_the_key_that_was_meant() {
    let project = broken();
    let found = find(&project, rules::UNKNOWN_KEY);
    let langauge = found
        .iter()
        .find(|diagnostic| diagnostic.message.contains("langauge"))
        .expect("the `langauge` typo");
    assert_eq!(langauge.severity, Severity::Warning);
    assert!(
        langauge.message.contains("did you mean `language`?"),
        "{}",
        langauge.message
    );
    assert_eq!(langauge.source.as_ref().map(|source| source.line), Some(7));
    assert_eq!(
        project.config().language,
        "en",
        "the typo means the default language is used, not the typed one"
    );
}

#[test]
fn every_diagnostic_can_be_acted_on() {
    let project = broken();
    assert!(
        !project.diagnostics().is_empty(),
        "the fixture is broken on purpose"
    );
    for diagnostic in project.diagnostics() {
        let source = diagnostic
            .source
            .as_ref()
            .unwrap_or_else(|| panic!("{} has no source location", diagnostic.rule));
        assert!(
            source.file.is_relative(),
            "{} names an absolute path: {}",
            diagnostic.rule,
            source.file.display()
        );
        assert!(source.line >= 1 && source.column >= 1, "{diagnostic:?}");
        assert!(
            diagnostic.rule.starts_with("BK-"),
            "{} is not a rule ID",
            diagnostic.rule
        );
        assert!(
            !diagnostic.message.is_empty() && !diagnostic.message.contains("  "),
            "{diagnostic:?}"
        );
    }
}

#[test]
fn diagnostics_come_out_in_the_same_order_every_time() {
    let one = broken();
    let two = broken();
    let render = |project: &Project| {
        project
            .diagnostics()
            .iter()
            .map(booker_project::format_diagnostic)
            .collect::<Vec<_>>()
    };
    assert_eq!(render(&one), render(&two));
    let lines = render(&one);
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("book.toml:7:1: warning[BK-FORMAT-005]")),
        "{lines:#?}"
    );
    assert!(
        lines.windows(2).all(|pair| {
            let file = |line: &String| line.split(':').next().unwrap_or_default().to_string();
            file(&pair[0]) <= file(&pair[1])
        }),
        "sorted by file: {lines:#?}"
    );
}

#[test]
fn the_minimal_fixture_is_a_book_with_nothing_wrong_with_it() {
    let project = Project::load(fixture("minimal")).expect("it loads");
    assert_eq!(project.config().title, "A Very Short Book");
    assert_eq!(
        project.chapters().len(),
        1,
        "a single `book.md` in the root is a book"
    );
    assert_eq!(project.chapters()[0].relative_path(), Path::new("book.md"));
    assert!(project.chapters()[0].word_count() > 10);
    assert_eq!(
        project.diagnostics(),
        &[],
        "a project with nothing wrong says nothing"
    );
}

#[test]
fn a_folder_with_nothing_in_it_opens_and_says_what_is_missing() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let project = Project::load(directory.path()).expect("an empty folder still opens");

    let rules_found: Vec<&str> = project
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.rule.as_str())
        .collect();
    assert!(
        rules_found.contains(&rules::NO_BOOK_TOML),
        "{rules_found:?}"
    );
    assert!(rules_found.contains(&rules::NO_CONTENT), "{rules_found:?}");
    assert!(
        !project.has_errors(),
        "an empty folder is not yet a book, but nothing about it is an error"
    );
    assert_eq!(project.config().title, "");
    assert_eq!(project.config().format, booker_core::FORMAT_VERSION);
}

#[test]
fn a_project_from_a_newer_booker_opens_and_is_not_rewritten() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let original = "format = 99\ntitle = \"From The Future\"\nnew-in-v99 = \"kept\"\n";
    std::fs::write(directory.path().join("book.toml"), original).expect("write");
    std::fs::write(directory.path().join("book.md"), "# Hello\n").expect("write");

    let mut project = Project::load(directory.path()).expect("it opens");
    let found = project
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.rule == rules::FORMAT_TOO_NEW)
        .expect("it says so");
    assert_eq!(found.severity, Severity::Error);
    assert!(found.message.contains("format 99"), "{}", found.message);
    assert_eq!(found.source.as_ref().map(|source| source.line), Some(1));
    assert_eq!(project.chapters().len(), 1, "the text still loads");

    let error = project
        .migrate()
        .expect_err("we do not know how to migrate from the future");
    assert!(
        error.to_string().contains("newer than this build"),
        "{error}"
    );

    project.save().expect("saving is still a no-op");
    assert_eq!(
        std::fs::read_to_string(directory.path().join("book.toml")).expect("read"),
        original,
        "nothing a newer Booker wrote was touched"
    );
}

#[test]
fn chapters_fall_back_to_content_in_file_name_order() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(directory.path().join("book.toml"), "title = \"Mia\"\n").expect("write");
    let content = directory.path().join("content");
    std::fs::create_dir_all(&content).expect("create content/");
    for name in ["03-last.md", "01-first.md", "02-middle.md", "notes.txt"] {
        std::fs::write(content.join(name), "# One\n").expect("write");
    }

    let project = Project::load(directory.path()).expect("it loads");
    let names: Vec<String> = project
        .chapters()
        .iter()
        .map(|chapter| chapter.relative_path().display().to_string())
        .collect();
    assert_eq!(
        names,
        vec![
            format!("content{}01-first.md", std::path::MAIN_SEPARATOR),
            format!("content{}02-middle.md", std::path::MAIN_SEPARATOR),
            format!("content{}03-last.md", std::path::MAIN_SEPARATOR),
        ],
        "sorted by name, and only Markdown"
    );
}

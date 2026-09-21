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

/// A book in a temporary folder with one chapter, for rules that are about
/// what a chapter says rather than about the fixture's broken files.
fn book_with_chapter(text: &str) -> (tempfile::TempDir, Project) {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("book.toml"), "title = \"T\"\n").unwrap();
    std::fs::create_dir(folder.path().join("content")).unwrap();
    std::fs::write(folder.path().join("content/01.md"), text).unwrap();
    let project = Project::load(folder.path()).unwrap();
    (folder, project)
}

#[test]
fn an_attribute_nothing_reads_is_a_warning_that_suggests_the_right_key() {
    let (_folder, project) =
        book_with_chapter("# One {break-befor=page}\n\n![Cat](cat.png){widht=50%}\n");
    let found = find(&project, rules::UNKNOWN_ATTRIBUTE);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|d| d.severity == Severity::Warning));
    assert!(
        found[0].message.contains("did you mean `break-before`"),
        "{}",
        found[0].message
    );
    let source = found[0].source.as_ref().unwrap();
    assert_eq!(
        (source.line, source.column),
        (1, 7),
        "points at the braces, not the heading"
    );
    assert!(found[1].message.contains("did you mean `width`"));
}

#[test]
fn markdown_that_is_kept_but_not_laid_out_is_reported_where_it_is() {
    let (_folder, project) = book_with_chapter(
        "Text.[^1]\n\n<div>raw</div>\n\n<!-- a note to self -->\n\n[^1]: The note.\n",
    );
    let found = find(&project, rules::NOT_LAID_OUT);
    let lines: Vec<(u32, &str)> = found
        .iter()
        .map(|d| (d.source.as_ref().unwrap().line, d.message.as_str()))
        .collect();
    assert_eq!(
        found.len(),
        3,
        "reference, HTML, definition — not the comment: {lines:#?}"
    );
    assert!(lines[0].1.starts_with("a footnote reference"), "{lines:#?}");
    assert_eq!(lines[1].0, 3);
    assert!(lines[1].1.starts_with("HTML"));
    assert!(lines[2].1.starts_with("a footnote"));
}

fn book_with_config(toml: &str) -> (tempfile::TempDir, Project) {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("book.toml"), toml).unwrap();
    std::fs::write(folder.path().join("book.md"), "# One\n").unwrap();
    let project = Project::load(folder.path()).unwrap();
    (folder, project)
}

#[test]
fn a_theme_that_does_not_exist_names_the_ones_that_do() {
    let (_folder, project) = book_with_config("title = \"T\"\ntheme = \"novle\"\n");
    let found = find(&project, rules::UNKNOWN_THEME);
    assert_eq!(found.len(), 1);
    assert!(
        found[0].message.contains("did you mean `novel`?"),
        "{}",
        found[0].message
    );
    assert!(found[0]
        .message
        .contains("novel, picture-book, poetry, paper"));
    assert_eq!(found[0].source.as_ref().unwrap().line, 2);
    assert_eq!(
        project.config().theme(),
        "novel",
        "and the book still lays out"
    );
}

#[test]
fn the_wave_2_tables_are_read_and_their_mistakes_located() {
    let (_folder, project) = book_with_config(
        "title = \"T\"\ntheme = \"poetry\"\n\n[toc]\ndepth = 7\ntitel = \"x\"\n\n[chapter]\nstart = \"rigth-page\"\n",
    );
    assert_eq!(project.config().theme(), "poetry");
    assert_eq!(project.config().toc.depth, None, "a bad depth falls back");
    let all: Vec<String> = project
        .diagnostics()
        .iter()
        .map(booker_project::format_diagnostic)
        .collect();
    let all = all.join("\n");
    assert!(
        all.contains("book.toml:5:9:") && all.contains("`toc.depth` should be 1, 2 or 3"),
        "{all}"
    );
    assert!(
        all.contains("unknown key `toc.titel`") && all.contains("did you mean `toc.title`?"),
        "{all}"
    );
    assert!(
        all.contains("book.toml:9:9:") && all.contains("did you mean `right-page`?"),
        "{all}"
    );
    assert!(
        !all.contains("not implemented"),
        "theme, toc and chapter are implemented now: {all}"
    );
}

fn book_with_chapters(chapters: &[(&str, &str)]) -> (tempfile::TempDir, Project) {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("book.toml"), "title = \"T\"\n").unwrap();
    std::fs::create_dir(folder.path().join("content")).unwrap();
    for (name, text) in chapters {
        std::fs::write(folder.path().join("content").join(name), text).unwrap();
    }
    let project = Project::load(folder.path()).unwrap();
    (folder, project)
}

#[test]
fn a_link_to_an_id_that_exists_nowhere_is_located_and_the_nearest_suggested() {
    let (_folder, project) = book_with_chapters(&[
        ("01.md", "# The Garden {#garden}\n"),
        (
            "02.md",
            "Back in [the garden](#gardn), and [elsewhere](#nowhere-at-all).\n",
        ),
    ]);
    let found = find(&project, rules::UNRESOLVED_LINK);
    assert_eq!(found.len(), 2, "{found:#?}");
    let source = found[0].source.as_ref().unwrap();
    assert_eq!(source.file, Path::new("content/02.md"));
    assert_eq!((source.line, source.column), (1, 9));
    assert!(
        found[0].message.contains("Did you mean `#garden`?"),
        "{}",
        found[0].message
    );
    assert!(
        !found[1].message.contains("Did you mean"),
        "{}",
        found[1].message
    );
}

#[test]
fn an_id_used_twice_points_at_the_second_and_names_the_first() {
    let (_folder, project) = book_with_chapters(&[
        ("01.md", "# The Garden {#garden}\n"),
        ("02.md", "Text.\n\n## Another Garden {#garden}\n"),
    ]);
    let found = find(&project, rules::DUPLICATE_ID);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].source.as_ref().unwrap().file,
        Path::new("content/02.md")
    );
    assert!(
        found[0].message.contains("content/01.md:1"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_chapter_that_prints_nothing_is_a_warning() {
    let (_folder, project) =
        book_with_chapters(&[("01.md", "# One\n"), ("02.md", "<!-- to do -->\n")]);
    let found = find(&project, rules::EMPTY_CHAPTER);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(
        found[0].source.as_ref().unwrap().file,
        Path::new("content/02.md")
    );
}

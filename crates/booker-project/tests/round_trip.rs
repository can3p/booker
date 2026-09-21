//! The promise this crate makes to every person who ever hand-edits a
//! `book.toml`: Booker gives it back the way they wrote it.
//!
//! `AGENTS.md` §7 — unknown keys are preserved and written back unchanged,
//! writes keep comments, key order and formatting, and opening a project
//! changes nothing on disk.

use booker_project::{Project, Written};

/// A file with everything that usually gets destroyed by a round trip: a
/// leading comment, comments on their own line and at the end of a line,
/// keys in an order nobody would choose, unusual spacing, a table written
/// inline, an array over several lines, and four keys this build has never
/// heard of — one of which sits inside `[page]`.
const AWKWARD: &str = r##"# The Secret Garden of Mia
# Written by hand. Please do not tidy this file.

title   =   "The Secret Garden of Mia"     # spacing on purpose
language = "en"

# The format version goes here, not at the top, because I like it here.
format = 1

future-feature = { enabled = true, tries = 3 }

chapters = [
  "content/01-the-garden.md",   # the first one
  "content/02-flowers.md",
]

author = "Mia P."

[page]
facing   = true
size = "square"
margins = { top = "18mm", bottom = "22mm", inside = "20mm", outside = "15mm" }
lucky-number = 7                # no such key in this build

[toc]
enabled = true
depth = 2

[output.print]
format = "pdf"
crop-marks = true
"##;

fn project_with(contents: &str) -> (tempfile::TempDir, Project) {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let root = directory.path();
    std::fs::write(root.join("book.toml"), contents).expect("write book.toml");
    std::fs::create_dir_all(root.join("content")).expect("create content/");
    for name in ["01-the-garden.md", "02-flowers.md"] {
        std::fs::write(root.join("content").join(name), "# A chapter\n\nText.\n")
            .expect("write a chapter");
    }
    let project = Project::load(root).expect("a folder loads");
    (directory, project)
}

#[test]
fn a_file_written_by_hand_comes_back_byte_for_byte() {
    let (directory, mut project) = project_with(AWKWARD);
    let path = directory.path().join("book.toml");

    let written = project.save().expect("saving works");
    assert_eq!(
        written,
        Written::Unchanged,
        "nothing changed, so nothing should have been written"
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read it back"),
        AWKWARD,
        "the file must come back byte for byte"
    );
}

#[test]
fn opening_a_project_changes_nothing_on_disk() {
    let (directory, _project) = project_with(AWKWARD);
    let path = directory.path().join("book.toml");
    let before = std::fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .expect("a modification time");

    let _project = Project::load(directory.path()).expect("load again");
    let after = std::fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .expect("a modification time");
    assert_eq!(before, after, "loading must not touch the file");
}

#[test]
fn changing_one_value_changes_one_line() {
    let (directory, mut project) = project_with(AWKWARD);
    project.edit_config(|editor| editor.set_title("The Secret Garden"));
    assert_eq!(project.save().expect("saving works"), Written::Wrote);

    let after = std::fs::read_to_string(directory.path().join("book.toml")).expect("read it back");
    let changed: Vec<(&str, &str)> = AWKWARD
        .lines()
        .zip(after.lines())
        .filter(|(before, after)| before != after)
        .collect();
    assert_eq!(
        changed,
        vec![(
            "title   =   \"The Secret Garden of Mia\"     # spacing on purpose",
            "title   =   \"The Secret Garden\"     # spacing on purpose"
        )],
        "exactly one line changed, and the spacing and comment stayed"
    );
    assert_eq!(
        AWKWARD.lines().count(),
        after.lines().count(),
        "no lines were added or removed"
    );
}

#[test]
fn keys_this_build_does_not_understand_survive_a_write() {
    let (directory, mut project) = project_with(AWKWARD);

    // They are in the typed view…
    let extra = &project.config().extra;
    assert!(extra.contains_key("future-feature"), "{extra:?}");
    assert!(extra.contains_key("output"), "{extra:?}");
    // `[toc]` became a known table in Wave 2: read, not merely kept.
    assert_eq!(project.config().toc.depth, Some(2));
    assert_eq!(extra["future-feature"]["tries"].as_integer(), Some(3));

    // …and they are still in the file after we write to it.
    project.edit_config(|editor| editor.set_language("fr"));
    project.save().expect("saving works");
    let after = std::fs::read_to_string(directory.path().join("book.toml")).expect("read it back");
    assert!(
        after.contains("future-feature = { enabled = true, tries = 3 }"),
        "{after}"
    );
    assert!(
        after.contains("lucky-number = 7                # no such key in this build"),
        "{after}"
    );
    assert!(
        after.contains("[toc]\nenabled = true\ndepth = 2\n"),
        "{after}"
    );
    assert!(after.contains("[output.print]"), "{after}");
    assert!(after.contains("language = \"fr\""), "{after}");
}

#[test]
fn an_unknown_key_is_reported_and_a_typo_gets_a_suggestion() {
    let (_directory, project) =
        project_with("title = \"Mia\"\nlangauge = \"en\"\n\n[page]\nfacnig = true\n");
    let messages: Vec<String> = project
        .diagnostics()
        .iter()
        .map(booker_project::format_diagnostic)
        .collect();
    let all = messages.join("\n");
    assert!(
        all.contains("unknown key `langauge`") && all.contains("did you mean `language`?"),
        "{all}"
    );
    assert!(
        all.contains("unknown key `page.facnig`") && all.contains("did you mean `page.facing`?"),
        "{all}"
    );
    assert!(
        all.contains("book.toml:2:1:") && all.contains("book.toml:5:1:"),
        "each unknown key is located at the key itself: {all}"
    );
}

#[test]
fn keys_the_plan_has_named_but_this_build_lacks_are_not_called_unknown() {
    let (_directory, project) =
        project_with("title = \"Mia\"\n\n[output.print]\nformat = \"pdf\"\n");
    let about_output: Vec<&str> = project
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("output"))
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(about_output.len(), 1, "{about_output:?}");
    assert!(
        about_output[0].contains("not implemented in this build"),
        "{about_output:?}"
    );
    assert!(
        project
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.severity != booker_project::Severity::Error),
        "a planned-but-unimplemented section is not an error"
    );
}

#[test]
fn a_book_toml_that_does_not_parse_is_never_written_over() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let broken = "title = \"Mia\"\nthis line is not TOML at all\n";
    std::fs::write(directory.path().join("book.toml"), broken).expect("write it");

    let mut project = Project::load(directory.path()).expect("a broken project still opens");
    assert!(
        project
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.rule == booker_project::rules::INVALID_TOML),
        "{:?}",
        project.diagnostics()
    );

    let error = project.save().expect_err("saving must refuse");
    assert!(
        error.to_string().contains("will not write over it"),
        "{error}"
    );
    assert_eq!(
        std::fs::read_to_string(directory.path().join("book.toml")).expect("read it back"),
        broken,
        "the user's file is untouched"
    );
}

#[test]
fn saving_bumps_the_revision_only_when_something_was_written() {
    let (_directory, mut project) = project_with(AWKWARD);
    let start = project.revision();
    assert_eq!(project.save().expect("save"), Written::Unchanged);
    assert_eq!(
        project.revision(),
        start,
        "an unchanged save is not a change"
    );

    project.edit_config(|editor| editor.set_author(Some("M. P.")));
    assert_eq!(project.save().expect("save"), Written::Wrote);
    assert_eq!(project.revision(), start.next());
}

#[test]
fn a_migration_is_a_no_op_on_a_current_project() {
    let (directory, mut project) = project_with(AWKWARD);
    assert!(
        project
            .migrate()
            .expect("migrating a current project")
            .is_empty(),
        "format 1 is current, so there is nothing to run"
    );
    assert_eq!(project.save().expect("save"), Written::Unchanged);
    assert_eq!(
        std::fs::read_to_string(directory.path().join("book.toml")).expect("read"),
        AWKWARD
    );
}

//! Reading and writing a chapter, which is what an editor pane does all
//! day, and what an agent's text edit arrives as from the other side.
//!
//! `AGENTS.md` §7 — writes are atomic and targeted, the project is re-read
//! rather than patched in memory, and the revision counter only moves when
//! something actually changed.

use booker_project::{Project, Template};

/// A book with one chapter in it, as `booker new` writes one.
fn a_book() -> (tempfile::TempDir, Project) {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("mia");
    let (_, project) = Project::create(&root, Template::Novel, Some("The Secret Garden of Mia"))
        .expect("a new book");
    (dir, project)
}

/// The path of the first chapter, in the form that crosses to the UI.
fn first_chapter(project: &Project) -> String {
    project.chapter_summaries()[0].path.clone()
}

#[test]
fn a_chapter_is_found_by_the_path_the_ui_was_given() {
    let (_dir, project) = a_book();
    let path = first_chapter(&project);

    let chapter = project
        .chapter(&path)
        .expect("the chapter the summary named");
    assert!(chapter.source().contains("The First Chapter"));
}

#[test]
fn a_path_that_is_not_a_chapter_of_this_book_is_not_found() {
    let (_dir, project) = a_book();

    assert!(project.chapter("content/nowhere.md").is_none());
    // Containment falls out of the same check: whatever this points at, it
    // is not a chapter of this book.
    assert!(project.chapter("../../../etc/passwd").is_none());
}

#[test]
fn writing_a_chapter_changes_the_file_the_counts_and_the_revision() {
    let (_dir, mut project) = a_book();
    let path = first_chapter(&project);
    let before = project.revision();
    let words_before = project.chapter_summaries()[0].words;

    let written = project
        .write_chapter(&path, "# A New Beginning\n\nOne sentence.\n")
        .expect("the write succeeds");

    assert!(written.changed());
    assert!(
        project.revision() > before,
        "a real change moves the revision"
    );

    let summary = &project.chapter_summaries()[0];
    assert_eq!(summary.title.as_deref(), Some("A New Beginning"));
    assert_ne!(
        summary.words, words_before,
        "the re-read is what makes the counts agree with the disk"
    );

    let on_disk = std::fs::read_to_string(project.chapter(&path).unwrap().path()).unwrap();
    assert!(on_disk.starts_with("# A New Beginning"));
}

#[test]
fn writing_the_same_text_again_is_not_a_write() {
    let (_dir, mut project) = a_book();
    let path = first_chapter(&project);
    let text = project.chapter(&path).unwrap().source().to_string();
    let before = project.revision();

    let written = project.write_chapter(&path, &text).expect("it is allowed");

    assert!(!written.changed());
    assert_eq!(
        project.revision(),
        before,
        "an unchanged save must not wake every watcher in the system"
    );
}

#[test]
fn writing_a_chapter_that_does_not_exist_says_so_rather_than_creating_it() {
    let (_dir, mut project) = a_book();

    let error = project
        .write_chapter("content/invented.md", "hello")
        .expect_err("a chapter the book does not have");
    assert!(
        error.to_string().contains("no such chapter"),
        "the message should say what is wrong: {error}"
    );
}

#[test]
fn reloading_picks_up_a_change_made_from_outside() {
    let (_dir, mut project) = a_book();
    let path = first_chapter(&project);
    let file = project.chapter(&path).unwrap().path().to_path_buf();
    let before = project.revision();

    // An agent, another editor, a branch checkout — the folder simply
    // changed under us.
    std::fs::write(&file, "# Rewritten From Outside\n\nBy somebody else.\n").unwrap();
    project.reload().expect("it reloads");

    assert!(project.revision() > before);
    assert_eq!(
        project.chapter_summaries()[0].title.as_deref(),
        Some("Rewritten From Outside")
    );
}

//! `booker new` then `booker build` on a fresh folder — the thing a person
//! does first, and the one path that must never break.

use std::path::{Path, PathBuf};

use booker_cli::{run, Command, HAS_ERRORS, OK};

fn call(command: Command) -> (i32, String) {
    let mut out = Vec::new();
    let code = run(command, &mut out).expect("the command runs");
    (code, String::from_utf8(out).expect("output is text"))
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn new_then_build_works_on_a_fresh_folder() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let root = directory.path().join("the-secret-garden");

    let (code, output) = call(Command::New {
        path: root.clone(),
        template: "novel".to_string(),
        title: None,
    });
    assert_eq!(code, OK, "{output}");
    assert!(output.contains("Created a `novel` book"), "{output}");

    // The files `new` lists are project-relative, so they must read the same
    // on every platform. Windows printed `content\01-the-first-chapter.md`
    // here while `build`, three lines of output later, printed the same file
    // with a forward slash — a disagreement no reader can make sense of, and
    // one an agent comparing output across platforms cannot either
    // (`docs/FINDINGS.md`, and `display_path`).
    assert!(
        output.contains("  content/01-the-first-chapter.md"),
        "`new` lists what it wrote with forward slashes: {output}"
    );

    // Everything a book needs is there, and the title came from the folder.
    for file in [
        "book.toml",
        "content/01-the-first-chapter.md",
        ".gitignore",
        ".gitattributes",
        "AGENTS.md",
    ] {
        assert!(root.join(file).is_file(), "{file} was not created");
    }
    let book_toml = std::fs::read_to_string(root.join("book.toml")).expect("read book.toml");
    assert!(
        book_toml.contains("title = \"The Secret Garden\""),
        "{book_toml}"
    );
    assert!(book_toml.contains("format = 1"), "{book_toml}");
    assert!(
        book_toml.contains("# This is your book."),
        "the starter file explains itself: {book_toml}"
    );

    let (code, output) = call(Command::Build {
        project: root.clone(),
    });
    assert_eq!(code, OK, "a fresh project builds cleanly: {output}");
    assert!(output.contains("The Secret Garden"), "{output}");
    assert!(output.contains("Problems: none"), "{output}");
    assert!(
        output.contains("content/01-the-first-chapter.md"),
        "{output}"
    );
    assert!(
        output.contains("build/the-secret-garden.pdf"),
        "it says what it would build: {output}"
    );
}

#[test]
fn the_generated_book_explains_the_format_to_whoever_opens_it_next() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let root = directory.path().join("mia");
    call(Command::New {
        path: root.clone(),
        template: "novel".to_string(),
        title: Some("Mia".to_string()),
    });

    let agents = std::fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    // What an agent opening this folder has to be told (`PLAN.md` §11.6).
    for expected in [
        "book.toml",
        "content/*.md",
        "assets/images/",
        "format = 1",
        "Unknown keys are kept",
        "booker build",
        "BK-REF-001",
    ] {
        assert!(
            agents.contains(expected),
            "AGENTS.md never mentions {expected}"
        );
    }
}

#[test]
fn new_refuses_to_write_over_a_folder_that_already_has_something_in_it() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let root = directory.path().join("busy");
    std::fs::create_dir_all(&root).expect("create it");
    std::fs::write(root.join("notes.txt"), "my only copy").expect("write");

    let mut out = Vec::new();
    let error = run(
        Command::New {
            path: root.clone(),
            template: "novel".to_string(),
            title: None,
        },
        &mut out,
    )
    .expect_err("it must refuse");
    assert!(error.to_string().contains("not empty"), "{error}");
    assert_eq!(
        std::fs::read_to_string(root.join("notes.txt")).expect("read"),
        "my only copy"
    );
    assert!(!root.join("book.toml").exists());
}

#[test]
fn an_unknown_template_says_what_there_is() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let mut out = Vec::new();
    let error = run(
        Command::New {
            path: directory.path().join("x"),
            template: "picturebook".to_string(),
            title: None,
        },
        &mut out,
    )
    .expect_err("it must refuse");
    let message = error.to_string();
    assert!(message.contains("novel, picture-book, poetry, paper"), "{message}");
    assert!(message.contains("did you mean `picture-book`?"), "{message}");
    assert!(
        !directory.path().join("x").exists(),
        "nothing is created for a template that does not exist"
    );
}

#[test]
fn building_a_broken_project_reports_and_exits_non_zero() {
    let (code, output) = call(Command::Build {
        project: fixture("broken"),
    });
    assert_eq!(code, HAS_ERRORS, "{output}");

    // It still says everything it could load…
    assert!(output.contains("The Broken Book"), "{output}");
    assert!(output.contains("content/01-the-garden.md"), "{output}");
    // …and every problem, located.
    for expected in [
        "book.toml:11:5: error[BK-REF-002]",
        "book.toml:16:10: error[BK-FORMAT-004]",
        "content/01-the-garden.md:6:1: error[BK-REF-001]",
        "book.toml:7:1: warning[BK-FORMAT-005]",
    ] {
        assert!(output.contains(expected), "no `{expected}` in:\n{output}");
    }
}

#[test]
fn building_a_folder_that_is_not_there_says_so_without_a_stack_trace() {
    let mut out = Vec::new();
    let error = run(
        Command::Build {
            project: PathBuf::from("/no/such/book"),
        },
        &mut out,
    )
    .expect_err("it must fail");
    let message = error.to_string();
    assert!(message.contains("/no/such/book"), "{message}");
    assert!(message.contains("no folder"), "{message}");
    assert!(!message.contains("panicked"), "{message}");
}

#[test]
fn building_the_minimal_fixture_is_clean() {
    let (code, output) = call(Command::Build {
        project: fixture("minimal"),
    });
    assert_eq!(code, OK, "{output}");
    assert!(output.contains("A Very Short Book"), "{output}");
    assert!(output.contains("book.md"), "{output}");
    assert!(output.contains("Problems: none"), "{output}");
}

#[test]
fn a_build_touches_nothing_but_the_generated_folder() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let root = directory.path().join("mia");
    call(Command::New {
        path: root.clone(),
        template: "novel".to_string(),
        title: None,
    });
    let before = listing(&root);

    call(Command::Build {
        project: root.clone(),
    });
    // A build writes its PDF into `build/`, which is generated and
    // git-ignored. Everything the author wrote must be exactly as it was:
    // opening or building a project never edits it (`AGENTS.md` §7).
    let after: Vec<String> = listing(&root)
        .into_iter()
        .filter(|entry| !entry.starts_with("build"))
        .collect();
    assert_eq!(after, before, "building must not change the author's files");
    assert!(
        root.join("build").exists(),
        "the build should have produced its output folder"
    );
}

fn listing(root: &Path) -> Vec<String> {
    fn walk(directory: &Path, root: &Path, found: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, found);
            } else {
                let contents = std::fs::read(&path).unwrap_or_default();
                found.push(format!(
                    "{} {}",
                    path.strip_prefix(root).unwrap_or(&path).display(),
                    contents.len()
                ));
            }
        }
    }
    let mut found = Vec::new();
    walk(root, root, &mut found);
    found.sort();
    found
}

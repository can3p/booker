//! Starter projects.
//!
//! What `booker new` writes. Two rules shape it: the result must build
//! immediately with no configuration (`AGENTS.md`, opening paragraph), and
//! it must explain itself to whoever opens the folder next — including an
//! agent, which is what the generated `AGENTS.md` is for (`PLAN.md` §11.6).

use std::path::{Path, PathBuf};

use booker_core::{Error, Result, FORMAT_VERSION};

use crate::write::write_atomic;

/// The starter projects this build can generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Flowing text, A5, facing pages: a novel, a memoir, a short story.
    Novel,
}

impl Template {
    pub const NAMES: &'static [&'static str] = &["novel"];

    pub fn from_name(name: &str) -> Result<Template> {
        match name.trim().to_ascii_lowercase().as_str() {
            "novel" => Ok(Template::Novel),
            other => Err(Error::Project {
                path: PathBuf::from("."),
                message: format!(
                    "unknown template `{other}`; this build has: {}",
                    Template::NAMES.join(", ")
                ),
            }),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Template::Novel => "novel",
        }
    }
}

/// What was created.
#[derive(Debug, Clone)]
pub struct Created {
    pub root: PathBuf,
    /// Project-relative, in the order they were written.
    pub files: Vec<PathBuf>,
}

/// Create a starter project at `root`.
///
/// Refuses to write into a folder that already holds a book: overwriting
/// someone's work is never the helpful answer.
pub fn create(root: &Path, template: Template, title: Option<&str>) -> Result<Created> {
    if root.join(crate::config::BOOK_TOML).exists() {
        return Err(Error::Project {
            path: root.to_path_buf(),
            message: "there is already a book here; `booker new` will not write over it"
                .to_string(),
        });
    }
    if let Ok(mut entries) = std::fs::read_dir(root) {
        if entries.next().is_some() {
            return Err(Error::Project {
                path: root.to_path_buf(),
                message: "this folder is not empty; give `booker new` a new folder".to_string(),
            });
        }
    }

    let title = title
        .map(str::to_string)
        .unwrap_or_else(|| title_from_folder(root));

    let files: Vec<(PathBuf, String)> = match template {
        Template::Novel => vec![
            (PathBuf::from(crate::config::BOOK_TOML), book_toml(&title)),
            (
                PathBuf::from("content").join("01-the-first-chapter.md"),
                first_chapter(&title),
            ),
            (
                PathBuf::from("assets").join("images").join(".gitkeep"),
                String::new(),
            ),
            (PathBuf::from(".gitignore"), GITIGNORE.to_string()),
            (PathBuf::from(".gitattributes"), GITATTRIBUTES.to_string()),
            (PathBuf::from("AGENTS.md"), agents_md(&title)),
        ],
    };

    let mut written = Vec::new();
    for (relative, contents) in files {
        write_atomic(&root.join(&relative), contents.as_bytes())?;
        written.push(relative);
    }

    Ok(Created {
        root: root.to_path_buf(),
        files: written,
    })
}

/// `the-secret-garden` → `The Secret Garden`.
fn title_from_folder(root: &Path) -> String {
    let name = root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    let words: Vec<String> = name
        .split(['-', '_', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if words.is_empty() {
        "Untitled".to_string()
    } else {
        words.join(" ")
    }
}

fn book_toml(title: &str) -> String {
    format!(
        r#"# This is your book. Everything here can be changed by hand, by the
# Booker app, or by an agent working in this folder — it is plain text on
# purpose, and Booker keeps your comments and your key order when it writes.

format = {FORMAT_VERSION}          # the project format; Booker migrates this for you
title = "{title}"
# author = "Your name here"
language = "en"

# Chapters are read in this order. Delete the list and Booker reads
# `content/*.md` sorted by file name instead.
chapters = [
    "content/01-the-first-chapter.md",
]

[page]
size = "a5"        # a4, a5, letter, trade, digest, square, or "5.5x8.5in"
facing = true      # margins swap on left and right pages, as in a printed book

[page.margins]
top = "18mm"
bottom = "20mm"
inside = "20mm"
outside = "15mm"
"#
    )
}

fn first_chapter(title: &str) -> String {
    format!(
        r#"# The First Chapter

Welcome to *{title}*. This file is ordinary Markdown: write here, save, and
the page on the right changes.

A second paragraph, so the page has something to lay out. Headings, **bold**,
*italic*, lists, quotes and images all work:

> A quote sits like this.

- one thing
- another thing

Put pictures in `assets/images/` and use them like this, once you have one:

<!-- ![A tulip in May](assets/images/tulips.jpg) -->
"#
    )
}

const GITIGNORE: &str = r#"# Booker's derived files. Everything here can be rebuilt from the text,
# and deleting it at any moment is safe.
/build/
.booker/
*.pdf
.DS_Store
"#;

const GITATTRIBUTES: &str = r#"# Keep text files diffable and identical on every machine.
*.md text eol=lf
*.toml text eol=lf
"#;

fn agents_md(title: &str) -> String {
    format!(
        r#"# {title} — how this folder works

This is a Booker book: a plain folder, in plain text, made to be edited by a
person, by the Booker app, and by you. Nothing here is generated from
anything else, so editing a file *is* editing the book.

## What each file is

| Path | What it is |
|---|---|
| `book.toml` | Title, author, language, chapter order, page setup, format version |
| `content/*.md` | The text, one file per chapter, ordinary Markdown |
| `assets/images/` | Pictures the chapters point at |
| `build/`, `.booker/` | Generated. Never edit, never commit; safe to delete |

## Editing safely

- **Edit the Markdown directly.** That is what it is for, and your diff is
  the record of what you changed.
- **Keep `format = {FORMAT_VERSION}` in `book.toml`.** It says which version of
  the project format this book uses. Booker migrates it when that changes;
  do not bump it by hand.
- **Unknown keys are kept, not deleted.** If you see a key here that this
  version of Booker does not document, leave it: a newer Booker wrote it.
- **Chapter order comes from `chapters` in `book.toml`.** If a chapter is
  listed there, the file must exist. If the list is missing entirely,
  `content/*.md` is read in file-name order.
- **Images are paths relative to this folder**, for example
  `assets/images/tulips.jpg`. A path that points at nothing is reported as
  `BK-REF-001`.

## Before you say you are finished

Run the build and read what it says:

```bash
booker build .
```

It prints every problem it found with the file, line and column it came
from, each with a stable rule ID such as `BK-REF-001`. Errors mean the book
will not come out right. Leave the project with none.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_name_becomes_a_reasonable_title() {
        assert_eq!(
            title_from_folder(Path::new("/tmp/the-secret-garden")),
            "The Secret Garden"
        );
        assert_eq!(title_from_folder(Path::new("/tmp/my_book")), "My Book");
        assert_eq!(title_from_folder(Path::new("/")), "Untitled");
    }

    #[test]
    fn unknown_templates_say_what_there_is() {
        let error = Template::from_name("picture-book").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("unknown template"), "{message}");
        assert!(message.contains("novel"), "{message}");
    }
}

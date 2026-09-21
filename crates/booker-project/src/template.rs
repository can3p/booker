//! Starter projects.
//!
//! What `booker new` writes. Two rules shape it: the result must build
//! immediately with no configuration (`AGENTS.md`, opening paragraph), and
//! it must explain itself to whoever opens the folder next — including an
//! agent, which is what the generated `AGENTS.md` is for (`PLAN.md` §11.6).

use std::path::{Path, PathBuf};

use booker_core::{Error, Result, FORMAT_VERSION};

use crate::write::write_atomic;

/// The starter projects this build can generate. Each is a folder under
/// `crates/booker-project/templates/`, compiled into the binary, and each
/// picks the built-in theme of the same name (`booker_typst::themes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Flowing text, A5, facing pages, chapters on the right: a novel, a
    /// memoir, a short story.
    Novel,
    /// A square book of pictures with a few lines each, in big friendly
    /// type, one picture per page.
    PictureBook,
    /// Poems, each line kept where the poet broke it, each poem on its own
    /// page.
    Poetry,
    /// A4 on one side, sections running on: an essay, a report, a school
    /// paper.
    Paper,
}

/// A template name that does not exist. Not a file problem, so it names no
/// file: the mistake is in what was typed after `--template`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTemplate {
    pub name: String,
}

impl std::fmt::Display for UnknownTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "there is no template called `{}`; the templates are: {}",
            self.name,
            Template::NAMES.join(", ")
        )?;
        if let Some(near) = crate::toml_tree::closest(&self.name, Template::NAMES) {
            write!(f, " — did you mean `{near}`?")?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownTemplate {}

impl Template {
    pub const NAMES: &'static [&'static str] = &["novel", "picture-book", "poetry", "paper"];

    pub fn from_name(name: &str) -> std::result::Result<Template, UnknownTemplate> {
        match name.trim().to_ascii_lowercase().as_str() {
            "novel" => Ok(Template::Novel),
            "picture-book" => Ok(Template::PictureBook),
            "poetry" => Ok(Template::Poetry),
            "paper" => Ok(Template::Paper),
            _ => Err(UnknownTemplate {
                name: name.trim().to_string(),
            }),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Template::Novel => "novel",
            Template::PictureBook => "picture-book",
            Template::Poetry => "poetry",
            Template::Paper => "paper",
        }
    }

    /// The template's own files, as `(project-relative path, text)`, with
    /// `{{title}}` and `{{format}}` still to be filled in.
    fn files(self) -> &'static [(&'static str, &'static str)] {
        macro_rules! files {
            ($dir:literal: $($path:literal),+ $(,)?) => {
                &[$(($path, include_str!(concat!("../templates/", $dir, "/", $path)))),+]
            };
        }
        match self {
            Template::Novel => files!("novel": "book.toml", "content/01-the-first-chapter.md"),
            Template::PictureBook => files!(
                "picture-book": "book.toml",
                "content/01-the-story.md",
                "assets/images/moon.svg",
            ),
            Template::Poetry => files!(
                "poetry": "book.toml",
                "content/01-how-to-write-a-poem-here.md",
                "content/02-the-moon-in-a-jar.md",
            ),
            Template::Paper => files!("paper": "book.toml", "content/01-the-paper.md"),
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
    // In `book.toml` the title sits inside a TOML string, so a quote or a
    // backslash in it must be escaped there — and only there.
    let toml_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let fill = |path: &str, text: &str| {
        let title = if path == crate::config::BOOK_TOML {
            &toml_title
        } else {
            &title
        };
        text.replace("{{title}}", title)
            .replace("{{format}}", &FORMAT_VERSION.to_string())
    };

    let mut files: Vec<(PathBuf, String)> = template
        .files()
        .iter()
        .map(|(path, text)| (PathBuf::from(path), fill(path, text)))
        .collect();
    if !files
        .iter()
        .any(|(path, _)| path.starts_with("assets/images"))
    {
        // An empty folder does not survive git; this keeps the place for
        // pictures visible from the first commit.
        files.push((
            PathBuf::from("assets").join("images").join(".gitkeep"),
            String::new(),
        ));
    }
    files.extend([
        (PathBuf::from(".gitignore"), GITIGNORE.to_string()),
        (PathBuf::from(".gitattributes"), GITATTRIBUTES.to_string()),
        (PathBuf::from("AGENTS.md"), agents_md(&title)),
    ]);

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
*.svg text eol=lf
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
| `book.toml` | Title, author, language, theme, chapter order, table of contents, page setup, format version |
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
    fn unknown_templates_say_what_there_is_and_suggest_the_nearest() {
        let message = Template::from_name("picturebook").unwrap_err().to_string();
        assert_eq!(
            message,
            "there is no template called `picturebook`; the templates are: novel, \
             picture-book, poetry, paper — did you mean `picture-book`?"
        );
        assert!(
            !message.starts_with('.'),
            "it names no file, because no file is involved"
        );
    }

    #[test]
    fn every_template_name_round_trips() {
        for name in Template::NAMES {
            assert_eq!(Template::from_name(name).unwrap().name(), *name);
        }
    }
}

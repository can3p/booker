//! The chapter tree's edits: add, move, rename, remove.
//!
//! Each is a targeted change to files a person also edits by hand, so each
//! keeps to the rules in `AGENTS.md` §7: `book.toml` is changed through
//! `toml_edit`, keeping its comments and key order, and only the
//! `chapters` list moves; chapter files are written atomically; and
//! **nothing is ever deleted**. Removing a chapter takes it out of the
//! book, and the file stays in the folder for the person to delete if
//! they mean it — a writer's text is the one thing Booker must not lose.
//!
//! Every edit makes the chapter order explicit in `book.toml`. A book that
//! relied on file names for its order gets a `chapters` list written the
//! first time its order is changed from the window, because "put this
//! chapter after that one" cannot be said with file names alone.

use std::path::{Path, PathBuf};

use booker_core::{display_path, Error, Result};
use booker_doc::{Block, Span};

use crate::write::write_atomic;
use crate::Project;

impl Project {
    /// The chapters' project-relative paths, in reading order.
    pub fn chapter_order(&self) -> Vec<String> {
        self.chapters
            .iter()
            .map(|chapter| display_path(chapter.relative_path()))
            .collect()
    }

    /// Add a chapter titled `title`, after the chapter `after` (or at the
    /// end), with `text` as its contents — `# {title}` when there is none.
    ///
    /// The file goes in `content/`, named from the title
    /// (`content/the-garden.md`), with a number added if that name is
    /// taken: an existing file is never written over. Returns the new
    /// chapter's path.
    pub fn add_chapter(
        &mut self,
        after: Option<&str>,
        title: &str,
        text: Option<&str>,
    ) -> Result<String> {
        let mut order = self.chapter_order();
        let position = match after {
            Some(after) => {
                order
                    .iter()
                    .position(|path| path == after)
                    .ok_or_else(|| not_a_chapter(after))?
                    + 1
            }
            None => order.len(),
        };

        let relative = self.free_name(title);
        let contents = match text {
            Some(text) => text.to_string(),
            None => format!("# {}\n\n", title.trim()),
        };
        write_atomic(&self.root().join(&relative), contents.as_bytes())?;

        let shown = display_path(&relative);
        order.insert(position, shown.clone());
        self.write_order(&order)?;
        Ok(shown)
    }

    /// Move a chapter so that it is at `index` in the reading order.
    pub fn move_chapter(&mut self, path: &str, index: usize) -> Result<()> {
        let mut order = self.chapter_order();
        let from = order
            .iter()
            .position(|p| p == path)
            .ok_or_else(|| not_a_chapter(path))?;
        let moved = order.remove(from);
        order.insert(index.min(order.len()), moved);
        self.write_order(&order)
    }

    /// Take a chapter out of the book. Its file is left where it is.
    pub fn remove_chapter(&mut self, path: &str) -> Result<()> {
        let mut order = self.chapter_order();
        let from = order
            .iter()
            .position(|p| p == path)
            .ok_or_else(|| not_a_chapter(path))?;
        order.remove(from);
        self.write_order(&order)
    }

    /// Give a chapter a new title: its first heading is rewritten, keeping
    /// any `{#id .class}` after it, or one is added at the top when the
    /// chapter has none. The file keeps its name — a file name is not
    /// something a reader ever sees, and renaming it would break every
    /// outside reference to it.
    pub fn rename_chapter(&mut self, path: &str, title: &str) -> Result<()> {
        let chapter = self.chapter(path).ok_or_else(|| not_a_chapter(path))?;
        let source = chapter.source();
        let title = title.trim();
        let heading = chapter
            .document()
            .blocks
            .iter()
            .find_map(|block| match block {
                Block::Heading(heading) => Some(heading),
                _ => None,
            });

        let text = match heading {
            Some(heading) => {
                let line_end = source[heading.span.start..]
                    .find('\n')
                    .map(|offset| heading.span.start + offset)
                    .unwrap_or(source.len());
                let old = &source[heading.span.start..line_end];
                let marks: String = old.chars().take_while(|c| *c == '#').collect();
                // An underlined (setext) heading has no `#`s; it becomes an
                // ordinary `#` heading, over the lines it used to take.
                let (marks, replace) = if marks.is_empty() {
                    ("#".repeat(usize::from(heading.level)), heading.span)
                } else {
                    (marks, Span::new(heading.span.start, line_end))
                };
                let attributes = heading
                    .attributes
                    .span
                    .and_then(|span| span.slice(source))
                    .map(|braces| format!(" {braces}"))
                    .unwrap_or_default();
                let mut line = format!("{marks} {title}{attributes}");
                if replace.end > line_end {
                    line.push('\n');
                }
                format!(
                    "{}{line}{}",
                    &source[..replace.start],
                    &source[replace.end..]
                )
            }
            None => format!("# {title}\n\n{source}"),
        };
        self.write_chapter(path, &text)?;
        Ok(())
    }

    /// `content/<slug>.md`, or `content/<slug>-2.md` and so on when taken.
    fn free_name(&self, title: &str) -> PathBuf {
        let slug = slug(title);
        let content = Path::new("content");
        let mut candidate = content.join(format!("{slug}.md"));
        let mut number = 2;
        while self.root().join(&candidate).exists() {
            candidate = content.join(format!("{slug}-{number}.md"));
            number += 1;
        }
        candidate
    }

    /// Write `order` as `chapters` in `book.toml` and read the book again.
    fn write_order(&mut self, order: &[String]) -> Result<()> {
        self.edit_config(|editor| editor.set_chapters(order));
        self.save()?;
        self.reload()
    }
}

fn not_a_chapter(path: &str) -> Error {
    Error::Project {
        path: PathBuf::from(path),
        message: "this book has no such chapter".to_string(),
    }
}

/// `The Walk Home!` → `the-walk-home`.
fn slug(title: &str) -> String {
    let mut slug = String::new();
    for word in title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.extend(word.chars().flat_map(char::to_lowercase));
    }
    if slug.is_empty() {
        "chapter".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> (tempfile::TempDir, Project) {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path();
        std::fs::write(
            root.join("book.toml"),
            "# my book\ntitle = \"T\"   # the title\n\n[page]\nsize = \"a5\"\n",
        )
        .unwrap();
        std::fs::create_dir(root.join("content")).unwrap();
        std::fs::write(root.join("content/01-one.md"), "# One {#one}\n\nText.\n").unwrap();
        std::fs::write(root.join("content/02-two.md"), "Two\n===\n\nMore.\n").unwrap();
        let project = Project::load(root).unwrap();
        (folder, project)
    }

    #[test]
    fn adding_a_chapter_writes_the_file_and_the_order_keeping_comments() {
        let (folder, mut project) = book();
        let added = project
            .add_chapter(Some("content/01-one.md"), "The Walk Home!", None)
            .unwrap();
        assert_eq!(added, "content/the-walk-home.md");
        assert_eq!(
            project.chapter_order(),
            [
                "content/01-one.md",
                "content/the-walk-home.md",
                "content/02-two.md"
            ]
        );
        let toml = std::fs::read_to_string(folder.path().join("book.toml")).unwrap();
        assert!(
            toml.starts_with("# my book\ntitle = \"T\"   # the title\n"),
            "{toml}"
        );
        assert_eq!(
            std::fs::read_to_string(folder.path().join(&added)).unwrap(),
            "# The Walk Home!\n\n"
        );
    }

    #[test]
    fn a_taken_name_gets_a_number_rather_than_being_overwritten() {
        let (_folder, mut project) = book();
        let first = project.add_chapter(None, "Extra", Some("mine")).unwrap();
        let second = project.add_chapter(None, "Extra", Some("theirs")).unwrap();
        assert_eq!(
            (first.as_str(), second.as_str()),
            ("content/extra.md", "content/extra-2.md")
        );
    }

    #[test]
    fn moving_and_removing_change_the_order_and_never_the_files() {
        let (folder, mut project) = book();
        project.move_chapter("content/02-two.md", 0).unwrap();
        assert_eq!(
            project.chapter_order(),
            ["content/02-two.md", "content/01-one.md"]
        );
        project.remove_chapter("content/01-one.md").unwrap();
        assert_eq!(project.chapter_order(), ["content/02-two.md"]);
        assert!(
            folder.path().join("content/01-one.md").is_file(),
            "a removed chapter's file stays in the folder"
        );
    }

    #[test]
    fn renaming_rewrites_the_heading_and_keeps_its_attributes() {
        let (folder, mut project) = book();
        project
            .rename_chapter("content/01-one.md", "The First")
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(folder.path().join("content/01-one.md")).unwrap(),
            "# The First {#one}\n\nText.\n"
        );
        project
            .rename_chapter("content/02-two.md", "The Second")
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(folder.path().join("content/02-two.md")).unwrap(),
            "# The Second\n\nMore.\n",
            "an underlined heading becomes a `#` heading"
        );
    }

    #[test]
    fn a_path_that_is_not_a_chapter_is_refused() {
        let (_folder, mut project) = book();
        assert!(project.move_chapter("content/nope.md", 0).is_err());
        assert!(project.add_chapter(Some("../x.md"), "X", None).is_err());
    }
}

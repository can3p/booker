//! Targeted edits to `book.toml`.
//!
//! Every write goes through here, and every write is a change to one value.
//! The rest of the file — comments, blank lines, key order, the decoration
//! around the value being changed — is left exactly as the author wrote it
//! (`AGENTS.md` §7). This is also the API a migration gets, so a migration
//! cannot accidentally rewrite a whole file either.

use std::path::Path;

use toml_edit::{DocumentMut, Item, Key, Table, TableLike, Value};

/// A borrowed handle on the document, handing out targeted edits.
pub struct ConfigEditor<'a> {
    document: &'a mut DocumentMut,
}

impl<'a> ConfigEditor<'a> {
    pub(crate) fn new(document: &'a mut DocumentMut) -> Self {
        Self { document }
    }

    // ---- the keys this build knows -------------------------------------

    pub fn set_title(&mut self, title: &str) {
        self.set_string(&["title"], title);
    }

    pub fn set_author(&mut self, author: Option<&str>) {
        match author {
            Some(author) => self.set_string(&["author"], author),
            None => {
                self.remove(&["author"]);
            }
        }
    }

    pub fn set_language(&mut self, language: &str) {
        self.set_string(&["language"], language);
    }

    /// The format version. A migration sets this; nothing else should.
    pub fn set_format(&mut self, version: u32) {
        self.set_integer(&["format"], i64::from(version));
    }

    pub fn set_chapters(&mut self, chapters: &[impl AsRef<Path>]) {
        let mut array = toml_edit::Array::new();
        for chapter in chapters {
            // Project files always use `/`, so a book written on Windows and
            // one written on macOS are the same file.
            array.push(slashed(chapter.as_ref()));
        }
        array.set_trailing_comma(chapters.len() > 1);
        self.set_value(&["chapters"], Value::Array(array));
    }

    // ---- generic, for later waves and for migrations --------------------

    pub fn set_string(&mut self, path: &[&str], value: &str) {
        self.set_value(path, Value::from(value));
    }

    pub fn set_integer(&mut self, path: &[&str], value: i64) {
        self.set_value(path, Value::from(value));
    }

    pub fn set_bool(&mut self, path: &[&str], value: bool) {
        self.set_value(path, Value::from(value));
    }

    pub fn get_string(&self, path: &[&str]) -> Option<String> {
        let (table, key) = path.split_at(path.len().checked_sub(1)?);
        let table = table_at(self.document, table)?;
        table
            .get(key.first()?)
            .and_then(Item::as_str)
            .map(str::to_string)
    }

    /// Replace one value, keeping the formatting and any comment that sat
    /// with it. Intermediate tables are created if they are missing.
    pub fn set_value(&mut self, path: &[&str], value: Value) {
        let Some((key, parents)) = path.split_last() else {
            return;
        };
        let Some(table) = table_at_mut(self.document, parents, true) else {
            return;
        };
        match table.get_mut(key) {
            Some(Item::Value(existing)) => {
                // Keep the decoration: the spaces and the trailing comment
                // belong to the author, not to us.
                let decor = existing.decor().clone();
                *existing = value;
                *existing.decor_mut() = decor;
            }
            Some(item) => *item = Item::Value(value),
            None => {
                table.insert(key, Item::Value(value));
            }
        }
    }

    /// Remove a key. Returns whether there was one.
    pub fn remove(&mut self, path: &[&str]) -> bool {
        let Some((key, parents)) = path.split_last() else {
            return false;
        };
        match table_at_mut(self.document, parents, false) {
            Some(table) => table.remove(key).is_some(),
            None => false,
        }
    }

    /// Rename a key, keeping its value **and** the comment written above
    /// it. The tool a migration reaches for most often.
    ///
    /// A comment belongs to the key it was written for, so it travels with
    /// it; without this, `remove` + `insert` silently drops it.
    pub fn rename(&mut self, path: &[&str], new_key: &str) -> bool {
        let Some((key, parents)) = path.split_last() else {
            return false;
        };
        let Some(table) = table_at_mut(self.document, parents, false) else {
            return false;
        };
        let decor = table.key(key).map(|key| key.leaf_decor().clone());
        let Some(item) = table.remove(key) else {
            return false;
        };
        let mut renamed = Key::new(new_key);
        if let Some(decor) = decor {
            *renamed.leaf_decor_mut() = decor;
        }
        table.entry_format(&renamed).or_insert(item);
        true
    }
}

fn slashed(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn table_at<'a>(document: &'a DocumentMut, path: &[&str]) -> Option<&'a dyn TableLike> {
    let mut current: &dyn TableLike = document.as_table();
    for key in path {
        current = current.get(key)?.as_table_like()?;
    }
    Some(current)
}

fn table_at_mut<'a>(
    document: &'a mut DocumentMut,
    path: &[&str],
    create: bool,
) -> Option<&'a mut dyn TableLike> {
    let mut current: &mut dyn TableLike = document.as_table_mut();
    for key in path {
        if current.get(key).is_none() {
            if !create {
                return None;
            }
            current.insert(key, Item::Table(Table::new()));
        }
        current = current.get_mut(key)?.as_table_like_mut()?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(text: &str) -> DocumentMut {
        text.parse().expect("valid TOML")
    }

    #[test]
    fn changing_a_value_leaves_the_comment_that_sits_with_it() {
        let mut doc = document("title = \"Old\"   # the working title\nformat = 1\n");
        ConfigEditor::new(&mut doc).set_title("New");
        assert_eq!(
            doc.to_string(),
            "title = \"New\"   # the working title\nformat = 1\n"
        );
    }

    #[test]
    fn adding_a_value_does_not_reorder_the_file() {
        let mut doc = document("# a book\nlanguage = \"fr\"\ntitle = \"Mia\"\n");
        ConfigEditor::new(&mut doc).set_author(Some("Mia P."));
        assert_eq!(
            doc.to_string(),
            "# a book\nlanguage = \"fr\"\ntitle = \"Mia\"\nauthor = \"Mia P.\"\n"
        );
    }

    #[test]
    fn reaches_into_a_table_and_into_an_inline_table() {
        let mut doc =
            document("[page]\nfacing = true\nmargins = { top = \"18mm\", bottom = \"20mm\" }\n");
        let mut editor = ConfigEditor::new(&mut doc);
        editor.set_bool(&["page", "facing"], false);
        editor.set_string(&["page", "margins", "top"], "25mm");
        assert_eq!(
            doc.to_string(),
            "[page]\nfacing = false\nmargins = { top = \"25mm\", bottom = \"20mm\" }\n"
        );
    }

    #[test]
    fn creates_the_table_when_it_is_not_there_yet() {
        let mut doc = document("title = \"Mia\"\n");
        ConfigEditor::new(&mut doc).set_string(&["page", "size"], "a5");
        assert_eq!(
            doc.to_string(),
            "title = \"Mia\"\n\n[page]\nsize = \"a5\"\n"
        );
    }

    #[test]
    fn chapters_are_written_with_forward_slashes() {
        let mut doc = document("title = \"Mia\"\n");
        ConfigEditor::new(&mut doc).set_chapters(&[
            std::path::PathBuf::from("content").join("01.md"),
            std::path::PathBuf::from("content").join("02.md"),
        ]);
        assert!(
            doc.to_string()
                .contains("\"content/01.md\", \"content/02.md\","),
            "{}",
            doc.to_string()
        );
    }

    #[test]
    fn renaming_keeps_the_comment_that_was_written_above_the_key() {
        let mut doc = document("# the working title\nold-name = \"keep me\"\n");
        assert!(ConfigEditor::new(&mut doc).rename(&["old-name"], "title"));
        assert_eq!(
            doc.to_string(),
            "# the working title\ntitle = \"keep me\"\n"
        );
    }

    #[test]
    fn renaming_keeps_the_value_and_removing_says_whether_it_was_there() {
        let mut doc = document("old-name = \"keep me\"\n");
        let mut editor = ConfigEditor::new(&mut doc);
        assert!(editor.rename(&["old-name"], "new-name"));
        assert_eq!(editor.get_string(&["new-name"]).as_deref(), Some("keep me"));
        assert!(editor.remove(&["new-name"]));
        assert!(!editor.remove(&["new-name"]));
        assert_eq!(doc.to_string(), "");
    }
}

//! A uniform, borrowed view over `toml_edit`'s two kinds of table.
//!
//! `toml_edit` distinguishes `[page]` (an `Item::Table`) from
//! `page = { ... }` (a `Value::InlineTable`), and both are perfectly normal
//! things for a person to write. Everything that reads `book.toml` wants to
//! treat them the same and, crucially, to keep the **byte span** the parser
//! recorded, which is what turns a bad value into a diagnostic that points
//! at a line.

use std::ops::Range;

use toml_edit::{Array, Item, Key, Value};

/// A node in the parsed document: either an item (possibly a table) or a
/// value inside an inline table or array.
#[derive(Clone, Copy)]
pub enum Node<'a> {
    Item(&'a Item),
    Value(&'a Value),
}

impl<'a> Node<'a> {
    /// The child under `key`, whichever kind of table this is.
    pub fn get(self, key: &str) -> Option<Node<'a>> {
        match self {
            Node::Item(Item::Table(table)) => table.get(key).map(Node::Item),
            Node::Item(Item::Value(value)) | Node::Value(value) => match value {
                Value::InlineTable(table) => table.get(key).map(Node::Value),
                _ => None,
            },
            _ => None,
        }
    }

    /// The keys of this table, in the order they were written.
    pub fn keys(self) -> Vec<&'a str> {
        match self {
            Node::Item(Item::Table(table)) => table.iter().map(|(key, _)| key).collect(),
            Node::Item(Item::Value(Value::InlineTable(table)))
            | Node::Value(Value::InlineTable(table)) => table.iter().map(|(key, _)| key).collect(),
            _ => Vec::new(),
        }
    }

    /// The byte range this node occupies in the file, when the parser
    /// recorded one. It does not after the document has been edited, which
    /// is why loading reads spans before anything writes.
    pub fn span(self) -> Option<Range<usize>> {
        match self {
            Node::Item(item) => item.span(),
            Node::Value(value) => value.span(),
        }
    }

    /// Where the *key* was written, rather than its value. An unknown key
    /// should underline the key itself.
    pub fn key_span(self, key: &str) -> Option<Range<usize>> {
        match self {
            Node::Item(Item::Table(table)) => table.key(key).and_then(Key::span),
            Node::Item(Item::Value(Value::InlineTable(table)))
            | Node::Value(Value::InlineTable(table)) => table.key(key).and_then(Key::span),
            _ => None,
        }
    }

    pub fn as_value(self) -> Option<&'a Value> {
        match self {
            Node::Item(Item::Value(value)) => Some(value),
            Node::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(self) -> Option<&'a str> {
        self.as_value().and_then(Value::as_str)
    }

    pub fn as_integer(self) -> Option<i64> {
        self.as_value().and_then(Value::as_integer)
    }

    pub fn as_bool(self) -> Option<bool> {
        self.as_value().and_then(Value::as_bool)
    }

    pub fn as_array(self) -> Option<&'a Array> {
        self.as_value().and_then(Value::as_array)
    }

    pub fn is_table(self) -> bool {
        matches!(
            self,
            Node::Item(Item::Table(_))
                | Node::Item(Item::Value(Value::InlineTable(_)))
                | Node::Value(Value::InlineTable(_))
        )
    }

    /// What kind of thing this is, phrased for an error message.
    pub fn type_name(self) -> &'static str {
        match self {
            Node::Item(Item::Table(_)) => "a table",
            Node::Item(Item::ArrayOfTables(_)) => "an array of tables",
            Node::Item(Item::None) => "nothing",
            Node::Item(Item::Value(value)) | Node::Value(value) => value_type_name(value),
        }
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "a string",
        Value::Integer(_) => "an integer",
        Value::Float(_) => "a number",
        Value::Boolean(_) => "true or false",
        Value::Datetime(_) => "a date",
        Value::Array(_) => "an array",
        Value::InlineTable(_) => "a table",
    }
}

/// Convert a node into an owned `toml::Value`, which is how unknown keys are
/// carried in `BookConfig::extra`.
///
/// The text of the file remains the authority for writing it back out; this
/// is only the typed view, so a lossy corner (an array of tables inlined
/// into an array) costs nothing on disk.
pub fn to_toml_value(node: Node<'_>) -> Option<toml::Value> {
    match node {
        Node::Item(Item::Table(table)) => {
            let mut map = toml::map::Map::new();
            for (key, item) in table.iter() {
                if let Some(value) = to_toml_value(Node::Item(item)) {
                    map.insert(key.to_string(), value);
                }
            }
            Some(toml::Value::Table(map))
        }
        Node::Item(Item::ArrayOfTables(tables)) => Some(toml::Value::Array(
            tables
                .iter()
                .map(|table| {
                    let mut map = toml::map::Map::new();
                    for (key, item) in table.iter() {
                        if let Some(value) = to_toml_value(Node::Item(item)) {
                            map.insert(key.to_string(), value);
                        }
                    }
                    toml::Value::Table(map)
                })
                .collect(),
        )),
        Node::Item(Item::None) => None,
        Node::Item(Item::Value(value)) | Node::Value(value) => value_to_toml(value),
    }
}

fn value_to_toml(value: &Value) -> Option<toml::Value> {
    Some(match value {
        Value::String(s) => toml::Value::String(s.value().clone()),
        Value::Integer(i) => toml::Value::Integer(*i.value()),
        Value::Float(f) => toml::Value::Float(*f.value()),
        Value::Boolean(b) => toml::Value::Boolean(*b.value()),
        Value::Datetime(d) => toml::Value::Datetime(*d.value()),
        Value::Array(array) => toml::Value::Array(array.iter().filter_map(value_to_toml).collect()),
        Value::InlineTable(table) => {
            let mut map = toml::map::Map::new();
            for (key, value) in table.iter() {
                if let Some(value) = value_to_toml(value) {
                    map.insert(key.to_string(), value);
                }
            }
            toml::Value::Table(map)
        }
    })
}

/// Distance between two key names, for "did you mean …" (`PLAN.md` §11.4).
/// Plain Levenshtein; the key sets are tiny.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j] + cost)
                .min(previous[j + 1] + 1)
                .min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// The closest of `candidates` to `key`, when one is close enough to be
/// worth suggesting.
pub fn closest<'a>(key: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let lowered = key.to_ascii_lowercase();
    candidates
        .iter()
        .map(|candidate| {
            (
                edit_distance(&lowered, &candidate.to_ascii_lowercase()),
                *candidate,
            )
        })
        .filter(|(distance, candidate)| *distance <= max_distance(candidate))
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
        .or_else(|| unique_completion(&lowered, candidates))
}

/// A word cut short or half-remembered — `poem` for `poetry`, `marg` for
/// `margins` — is too far from its candidate by edit distance, and yet
/// obviously meant. When nothing is close, the one candidate sharing the
/// longest beginning with what was typed is suggested, if that beginning is
/// at least three letters and no other candidate shares as much.
fn unique_completion<'a>(typed: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let shared = |candidate: &str| {
        typed
            .chars()
            .zip(candidate.to_ascii_lowercase().chars())
            .take_while(|(a, b)| a == b)
            .count()
    };
    let best = candidates.iter().map(|c| shared(c)).max()?;
    if best < 3 {
        return None;
    }
    let mut sharing = candidates.iter().filter(|c| shared(c) == best);
    let first = sharing.next()?;
    sharing.next().is_none().then_some(*first)
}

fn max_distance(candidate: &str) -> usize {
    match candidate.chars().count() {
        0..=3 => 1,
        4..=7 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_the_key_the_user_meant() {
        let keys = ["format", "title", "author", "language", "chapters", "page"];
        assert_eq!(closest("langauge", &keys), Some("language"));
        assert_eq!(closest("Title", &keys), Some("title"));
        assert_eq!(closest("chapter", &keys), Some("chapters"));
        assert_eq!(closest("authr", &keys), Some("author"));
    }

    #[test]
    fn says_nothing_when_nothing_is_close() {
        let keys = ["format", "title", "author"];
        assert_eq!(closest("subtitle-in-welsh", &keys), None);
        assert_eq!(closest("xyzzy", &keys), None);
    }

    #[test]
    fn reads_both_kinds_of_table_the_same_way() {
        // `ImDocument`, not `DocumentMut`: only the immutable parse keeps
        // spans (see `docs/FINDINGS.md`).
        let document: toml_edit::ImDocument<String> = r#"
[page]
facing = false
margins = { top = "18mm" }
"#
        .parse()
        .unwrap();
        let root = Node::Item(document.as_item());
        let page = root.get("page").expect("[page] is a table");
        assert_eq!(page.get("facing").and_then(Node::as_bool), Some(false));
        let margins = page.get("margins").expect("an inline table");
        assert!(margins.is_table());
        assert_eq!(margins.get("top").and_then(Node::as_str), Some("18mm"));
        assert!(margins.get("top").and_then(Node::span).is_some());
    }

    #[test]
    fn unknown_values_survive_as_toml_values() {
        let document: toml_edit::ImDocument<String> = r#"
future-feature = { enabled = true, tries = 3, names = ["a", "b"] }
"#
        .parse()
        .unwrap();
        let root = Node::Item(document.as_item());
        let value = to_toml_value(root.get("future-feature").unwrap()).unwrap();
        assert_eq!(value["enabled"].as_bool(), Some(true));
        assert_eq!(value["tries"].as_integer(), Some(3));
        assert_eq!(value["names"][1].as_str(), Some("b"));
    }
}

#[cfg(test)]
mod completion_tests {
    use super::closest;

    #[test]
    fn a_word_cut_short_suggests_the_one_word_it_starts_like() {
        let templates = ["novel", "picture-book", "poetry", "paper"];
        assert_eq!(closest("poem", &templates), Some("poetry"));
        assert_eq!(closest("pict", &templates), Some("picture-book"));
        assert_eq!(closest("pa", &templates), None, "too short to guess");
        assert_eq!(closest("xyz", &templates), None);
    }
}

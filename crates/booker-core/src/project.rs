use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::geometry::{Length, Margins, PageSize};

/// The project format this build writes. A change to the format means
/// bumping this and shipping a migration in the same change (`AGENTS.md` §7).
pub const FORMAT_VERSION: u32 = 1;

/// A project is a folder. This is its canonical root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ProjectRef {
    #[ts(type = "string")]
    pub root: PathBuf,
}

impl ProjectRef {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Paths that leave a project are always relative to its root, so that
    /// output is identical on every machine — which is what lets us diff two
    /// runs and compare CLI output with MCP output (`PLAN.md` §11.5).
    pub fn relative<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

/// A monotonic counter bumped every time the project on disk changes.
///
/// It lets a caller — the preview, the CLI, an agent — tell whether what it
/// read is still current, without locking anything (`PLAN.md` §11.1).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, TS,
)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct Revision(pub u64);

impl Revision {
    pub fn next(self) -> Self {
        Revision(self.0 + 1)
    }
}

/// `book.toml`.
///
/// `extra` captures everything this build does not recognise, so that a
/// project written by a newer Booker survives a round trip through an older
/// one (`AGENTS.md` §7). The parsing layer keeps the original text; this is
/// the typed view of it.
///
/// This is why there is no `deny_unknown_fields` here: rejecting a key we do
/// not know would break exactly the projects we promised to keep. A typo is
/// caught instead by a `format.*` diagnostic that suggests the right key
/// (`PLAN.md` §11.4), which helps a person far more than a parse failure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct BookConfig {
    #[serde(default = "default_format")]
    pub format: u32,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    /// Explicit chapter order. When empty, `content/*.md` sorted by name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "string[]")]
    pub chapters: Vec<PathBuf>,
    #[serde(default)]
    pub page: PageConfig,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(type = "Record<string, unknown>")]
    pub extra: BTreeMap<String, toml::Value>,
}

fn default_format() -> u32 {
    FORMAT_VERSION
}

fn default_language() -> String {
    "en".to_string()
}

impl BookConfig {
    /// Whether this build can open the project at all. Too new is an error
    /// the user can act on; too old is a migration, never a refusal.
    pub fn is_supported(&self) -> bool {
        self.format <= FORMAT_VERSION
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct PageConfig {
    #[serde(default)]
    pub size: PageSize,
    #[serde(default)]
    pub margins: Margins,
    /// Facing pages: margins are inside and outside rather than left and right.
    #[serde(default = "default_true")]
    pub facing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bleed: Option<Length>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(type = "Record<string, unknown>")]
    pub extra: BTreeMap<String, toml::Value>,
}

fn default_true() -> bool {
    true
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            size: PageSize::default(),
            margins: Margins::default(),
            facing: true,
            bleed: None,
            extra: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minimal_book_needs_only_a_title() {
        let config: BookConfig = toml::from_str(r#"title = "The Secret Garden of Mia""#).unwrap();
        assert_eq!(config.format, FORMAT_VERSION);
        assert_eq!(config.language, "en");
        assert!(config.page.facing);
        assert!(config.chapters.is_empty());
    }

    #[test]
    fn keeps_keys_it_does_not_understand() {
        let config: BookConfig = toml::from_str(
            r#"
            title = "Mia"
            future-feature = { enabled = true }
            "#,
        )
        .unwrap();
        assert!(
            config.extra.contains_key("future-feature"),
            "an older build must not silently drop what a newer one wrote"
        );
    }

    #[test]
    fn refuses_a_format_from_the_future() {
        let config: BookConfig = toml::from_str(
            r#"
            format = 99
            title = "Mia"
            "#,
        )
        .unwrap();
        assert!(!config.is_supported());
    }

    #[test]
    fn reads_the_page_block() {
        let config: BookConfig = toml::from_str(
            r#"
            title = "Mia"

            [page]
            size = "square"
            facing = false
            bleed = "3mm"

            [page.margins]
            top = "18mm"
            bottom = "22mm"
            inside = "20mm"
            outside = "15mm"
            "#,
        )
        .unwrap();
        assert!(!config.page.facing);
        assert_eq!(config.page.bleed.unwrap().to_string(), "3mm");
        let (w, _) = config.page.size.dimensions();
        assert!((w.to_mm() - 215.9).abs() < 1e-9);
    }

    #[test]
    fn paths_that_leave_the_project_are_relative_to_it() {
        let project = ProjectRef::new("/books/mia");
        assert_eq!(
            project.relative(Path::new("/books/mia/content/01.md")),
            Path::new("content/01.md")
        );
    }
}

//! The command surface the application exposes to its UI.
//!
//! Two rules (`AGENTS.md` §3, §6):
//!
//! * This list is **append-only** within a wave, and is edited only in an
//!   integration commit, so that parallel tracks never conflict over it.
//! * Every command here must have an equivalent in the CLI. The agent-facing
//!   surface is a thin layer over the same core, never a second
//!   implementation (`PLAN.md` §11.3).
//!
//! The second rule needs one clarification, added in Wave 1 when the app
//! first grew commands of its own. It binds **questions about a book**:
//! anything the window can say about a project — its structure, its pages,
//! its problems — the CLI must be able to print, or we have two answers to
//! one question and they will drift. It does not bind the plumbing a window
//! needs and a terminal does not: which folders were opened recently, and
//! whether a newer build exists. Those are not questions about a book, they
//! have no meaningful CLI form, and pretending otherwise would add commands
//! nobody would ever run.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::diagnostic::Diagnostic;
use crate::project::{BookConfig, ProjectRef, Revision};

/// Names of the commands the app exposes. Keep them sorted; add, never
/// renumber or reuse.
///
/// Wave 2's contracts step added six, in its contracts commit: the chapter
/// tree's four writes (`add_chapter`, `move_chapter`, `remove_chapter`,
/// `rename_chapter`) — writes rather than questions about a book, so they
/// need no CLI twin (see the module comment) — and click-to-source in both
/// directions (`pages_at`, `source_at`), which do: `booker where` and
/// `booker page`.
pub const COMMANDS: &[&str] = &[
    "add_chapter",
    "close_project",
    "compile",
    "export_pdf",
    "move_chapter",
    "open_project",
    "page_image_url",
    "pages_at",
    "project_info",
    "read_chapter",
    "recent_projects",
    // Appended in Wave 1 track C, in an integration commit: the watcher
    // says *that* the folder changed, and this says what the book now is
    // (`AGENTS.md` §3 — the list grows by appending, never by renumbering).
    "reload_project",
    "remove_chapter",
    "rename_chapter",
    "render_page",
    "save_chapter",
    "source_at",
];

/// Names of the events the core pushes at the UI, rather than answering when
/// asked. Same rules as [`COMMANDS`]: sorted, append-only within a wave.
///
/// There is one, and it is the whole of the outside-edit story the app needs
/// in Wave 1: the project on disk is no longer what you last read, here is
/// its new revision. A bulk change — an agent rewriting thirty files, a
/// branch checkout — is coalesced into a single event rather than thirty
/// (`AGENTS.md` §7).
pub const EVENTS: &[&str] = &["project-changed"];

/// Page images are served over a custom protocol rather than passed through
/// IPC, because a preview scrolls fast and base64 through a JSON channel is
/// the slowest thing in the app (`PLAN.md` §4).
///
/// Shape: `booker://page/<revision>/<page>@<scale>x.<png|svg>`
///
/// The revision is in the path on purpose: an image is immutable for a given
/// revision, so the webview may cache it forever, and a stale one can never
/// be mistaken for a current one.
pub fn page_image_url(revision: u64, page: u32, scale: f32, format: &str) -> String {
    format!("booker://page/{revision}/{page}@{scale}x.{format}")
}

/// One chapter, as the sidebar and the status bar need it.
///
/// These are exactly the numbers `booker build` prints for each chapter, so
/// that the window and the terminal cannot disagree about a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ChapterSummary {
    /// Relative to the project root, with forward slashes on every platform
    /// (`crate::display_path`).
    pub path: String,
    /// The first heading in the file, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub words: usize,
    pub headings: usize,
    pub images: usize,
}

/// Everything the window knows about an open project before it has compiled
/// anything.
///
/// A project that fails to load in part is still described here, with the
/// failures in `diagnostics`: refusing to open is never right, because that
/// is exactly when the errors need to be visible (`AGENTS.md` §7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ProjectInfo {
    pub project: ProjectRef,
    pub config: BookConfig,
    pub chapters: Vec<ChapterSummary>,
    /// Always present, possibly empty.
    pub diagnostics: Vec<Diagnostic>,
    pub revision: Revision,
}

/// The source of one chapter, on its way to or from the editor pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ChapterText {
    pub project: ProjectRef,
    /// Relative to the project root, as [`ChapterSummary::path`] gives it.
    pub path: String,
    pub text: String,
    /// The revision the text was read at. A save carrying a revision older
    /// than the project's means the file changed underneath the editor, and
    /// the app must offer both versions rather than discard one
    /// (`AGENTS.md` §7).
    pub revision: Revision,
}

/// What happened to a save.
///
/// A save never overwrites a file whose content is not what the editor
/// loaded: that would silently discard an edit made from outside — another
/// editor, an agent, a branch checkout (`AGENTS.md` §7). The comparison is
/// by content, not by revision, because the revision moves for any change
/// to the folder and a change to a different chapter is no conflict with
/// this one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub enum SaveOutcome {
    /// Written; this is the project as it now is.
    Saved { info: ProjectInfo },
    /// The file changed underneath the editor. Nothing was written, and the
    /// person chooses: keep theirs, take the one on disk, or keep both.
    Conflict {
        /// What is on disk now.
        disk: ChapterText,
        /// What the editor tried to save.
        mine: String,
    },
}

/// Where an export should land. The app asks the user; the CLI computes it
/// from the title, which is why the destination is explicit here and absent
/// from [`crate::CompileRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ExportRequest {
    pub project: ProjectRef,
    pub revision: Revision,
    #[ts(type = "string")]
    pub destination: PathBuf,
}

/// Told to the UI when the project on disk has moved on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../app/src/lib/bindings/")]
pub struct ProjectChanged {
    pub project: ProjectRef,
    pub revision: Revision,
    /// The files that changed, relative to the project root. Empty means
    /// "enough changed that it is not worth listing" — a checkout, say —
    /// and the UI should reload everything.
    pub paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_sorted_and_unique() {
        let mut sorted = COMMANDS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.as_slice(),
            COMMANDS,
            "keep the command list sorted and free of duplicates"
        );
    }

    #[test]
    fn events_are_sorted_and_unique() {
        let mut sorted = EVENTS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.as_slice(), EVENTS);
    }

    #[test]
    fn page_urls_carry_the_revision() {
        assert_eq!(page_image_url(7, 3, 2.0, "png"), "booker://page/7/3@2x.png");
    }

    #[test]
    fn a_chapter_summary_says_what_booker_build_says() {
        let summary = ChapterSummary {
            path: "content/01-the-first-chapter.md".into(),
            title: Some("The First Chapter".into()),
            words: 63,
            headings: 1,
            images: 0,
        };
        let value = serde_json::to_value(&summary).unwrap();
        assert_eq!(value["path"], "content/01-the-first-chapter.md");
        assert_eq!(value["words"], 63);
    }

    #[test]
    fn a_chapter_with_no_heading_leaves_the_title_out() {
        let summary = ChapterSummary {
            path: "content/02.md".into(),
            title: None,
            words: 0,
            headings: 0,
            images: 0,
        };
        let value = serde_json::to_value(&summary).unwrap();
        assert!(
            value.get("title").is_none(),
            "an absent title is absent, not null"
        );
    }
}

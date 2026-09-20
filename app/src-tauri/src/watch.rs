//! Watching a project folder for changes made from outside.
//!
//! Booker is used *alongside* agents: somebody will open Claude Code in
//! their book folder and ask it to fix the spelling in thirty chapters
//! while the window is open. That is the normal case, not the exotic one
//! (`AGENTS.md` §7), and it shapes everything here:
//!
//! * **Bursts are coalesced.** Thirty files rewritten is one reload and one
//!   re-render, not thirty. A debouncer collects events for a moment before
//!   anything is told.
//! * **Our own writes are ignored.** Saving a chapter changes the folder
//!   too, and reloading because of it would fight the editor. A write we
//!   made is remembered for a moment and the echo dropped.
//! * **Generated folders are not watched at all.** `build/` and `.booker/`
//!   change constantly and mean nothing to the text.
//!
//! What this does *not* do is resolve a conflict: an outside edit arriving
//! while a chapter has unsaved keystrokes. Saving eagerly makes that rare,
//! and Wave 2 track E is where both versions are offered rather than one
//! discarded.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use booker_core::{display_path, ProjectChanged, ProjectRef};
use notify::RecursiveMode;
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};

/// How long a burst of changes is collected before anything is told.
///
/// Long enough that a branch checkout or an agent's rewrite arrives as one
/// event; short enough that saving in another editor feels immediate.
const DEBOUNCE: Duration = Duration::from_millis(250);

/// How long a write of our own is remembered, so its echo can be dropped.
///
/// The filesystem event arrives after the write, by a margin that depends
/// on the platform. This is generous because the cost of it being too short
/// is a wasted reload, and the cost of it being too long is missing a real
/// outside edit to the same file within the same fraction of a second.
const ECHO_WINDOW: Duration = Duration::from_secs(2);

/// Folders whose contents never mean the book changed.
const IGNORED: [&str; 2] = ["build", ".booker"];

/// Writes this process made, so their echoes can be recognised.
///
/// Shared with whatever does the writing: a save records the file here
/// immediately before writing it.
#[derive(Default)]
pub struct OwnWrites {
    seen: Mutex<HashMap<PathBuf, Instant>>,
}

impl OwnWrites {
    /// Remember that we are about to write this file.
    pub fn record(&self, path: &Path) {
        if let Ok(mut seen) = self.seen.lock() {
            seen.insert(path.to_path_buf(), Instant::now());
        }
    }

    /// Whether this change is the echo of a write we just made. Consumes
    /// the record, so a second change to the same file is a real one.
    pub fn is_echo(&self, path: &Path) -> bool {
        let Ok(mut seen) = self.seen.lock() else {
            return false;
        };
        seen.retain(|_, at| at.elapsed() < ECHO_WINDOW);
        seen.remove(path).is_some()
    }
}

/// A running watch. Dropping it stops the watch.
pub struct Watch {
    _debouncer: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
}

/// Whether a path is one a reader of the book would care about.
///
/// Anything inside a generated folder is not, and neither is a directory
/// itself — its contents will report themselves.
pub fn is_interesting(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        // Outside the project entirely: not ours to care about.
        return false;
    };
    if relative
        .components()
        .next()
        .is_some_and(|first| IGNORED.iter().any(|ignored| first.as_os_str() == *ignored))
    {
        return false;
    }
    !path.is_dir()
}

/// Turn a burst of filesystem events into the paths worth reporting.
///
/// Returns them sorted and deduplicated, project-relative, so two runs of
/// the same change produce the same event (`PLAN.md` §11.5).
///
/// **The paths are deduplicated before echoes are checked, and that order
/// matters.** One write produces several events — the file is created,
/// its contents change, its metadata changes — and each carries the same
/// path. Asking "is this our echo?" once per event would answer yes to the
/// first and no to the rest, so a save would still be reported as an
/// outside edit. One question per file per burst is the right number.
pub fn paths_worth_reporting(
    root: &Path,
    own: &OwnWrites,
    events: &[DebouncedEvent],
) -> Vec<String> {
    let mut changed: Vec<&Path> = events
        .iter()
        .flat_map(|event| event.paths.iter())
        .map(PathBuf::as_path)
        .filter(|path| is_interesting(root, path))
        .collect();
    changed.sort_unstable();
    changed.dedup();

    changed
        .into_iter()
        .filter(|path| !own.is_echo(path))
        .map(|path| display_path(path.strip_prefix(root).unwrap_or(path)))
        .collect()
}

/// Start watching `project`, calling `on_change` once per burst.
///
/// The callback runs on the watcher's own thread, so it should hand the
/// work on rather than do it there.
pub fn start(
    project: ProjectRef,
    own: Arc<OwnWrites>,
    on_change: impl Fn(ProjectChanged) + Send + 'static,
) -> notify::Result<Watch> {
    let root = project.root.clone();
    let watched = root.clone();

    // The parameter is annotated because inference otherwise reads the
    // type backwards from the `&[DebouncedEvent]` below and lands on an
    // unsized one.
    let mut debouncer = new_debouncer(DEBOUNCE, None, move |result: DebounceEventResult| {
        let Ok(events) = result else {
            // A watch error — a folder removed, a limit reached — is not
            // something to interrupt somebody for. The next real change
            // reports itself, and a project that has gone away will fail
            // at the next read with a message that names it.
            return;
        };
        let paths = paths_worth_reporting(&root, &own, &events);
        if paths.is_empty() {
            return;
        }
        on_change(ProjectChanged {
            project: ProjectRef::new(root.clone()),
            // The reader asks the core what the revision now is; carrying
            // a guess here would be a second answer to that question.
            revision: Default::default(),
            paths,
        });
    })?;

    debouncer.watch(&watched, RecursiveMode::Recursive)?;
    Ok(Watch {
        _debouncer: debouncer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_folders_are_not_worth_watching() {
        let root = Path::new("/books/mia");
        assert!(!is_interesting(root, &root.join("build/mia.pdf")));
        assert!(!is_interesting(root, &root.join(".booker/cache/1")));
        // A path outside the project is not ours.
        assert!(!is_interesting(root, Path::new("/elsewhere/notes.md")));
    }

    #[test]
    fn a_chapter_is_worth_watching() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let chapter = root.join("content").join("01.md");
        std::fs::create_dir_all(chapter.parent().unwrap()).unwrap();
        std::fs::write(&chapter, "# One").unwrap();

        assert!(is_interesting(root, &chapter));
        // The folder itself is not: its contents speak for it.
        assert!(!is_interesting(root, &root.join("content")));
    }

    #[test]
    fn a_write_we_made_is_recognised_once() {
        let own = OwnWrites::default();
        let path = Path::new("/books/mia/content/01.md");

        own.record(path);
        assert!(own.is_echo(path), "the echo of our own write");
        assert!(
            !own.is_echo(path),
            "a second change to the same file is somebody else's"
        );
    }

    #[test]
    fn a_file_we_never_wrote_is_never_an_echo() {
        let own = OwnWrites::default();
        assert!(!own.is_echo(Path::new("/books/mia/content/02.md")));
    }
}

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
//!   too, and reloading because of it would fight the editor. A save
//!   remembers what it wrote, and a change that leaves the file holding
//!   exactly that is dropped.
//! * **Generated folders are not watched at all.** `build/` and `.booker/`
//!   change constantly and mean nothing to the text.
//!
//! What this does *not* do is resolve a conflict: an outside edit arriving
//! while a chapter has unsaved keystrokes. Saving eagerly makes that rare,
//! and Wave 2 track E is where both versions are offered rather than one
//! discarded.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use booker_core::{display_path, ProjectChanged, ProjectRef};
use booker_project::is_write_temporary;
use notify::RecursiveMode;
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};

/// How long a burst of changes is collected before anything is told.
///
/// Long enough that a branch checkout or an agent's rewrite arrives as one
/// event; short enough that saving in another editor feels immediate.
const DEBOUNCE: Duration = Duration::from_millis(250);

/// Folders whose contents never mean the book changed.
const IGNORED: [&str; 2] = ["build", ".booker"];

/// Writes this process made, so their echoes can be recognised.
///
/// Shared with whatever does the writing: a save records the file *and the
/// bytes it is about to hold* here immediately before writing it.
///
/// **The comparison is of content, not of timing** (`AGENTS.md` §7), and
/// that is the whole point. A time window has to be long enough to cover
/// the slowest machine's echo and short enough not to swallow a real edit
/// arriving just after a save, and no window is both: one that consumed
/// its record on the first event reported the second one — and one write
/// routinely produces several, which the debouncer is free to deliver in
/// separate batches because it expires each event on its own clock. That
/// is a race whose outcome depends on how loaded the machine is, and it
/// is how this was found. Reading the file and comparing answers the same
/// way however many events one write produced, and says "somebody else"
/// the moment the bytes stop being ours.
///
/// The map holds one small entry per file we have written in this
/// session, so it is bounded by the size of the book.
#[derive(Default)]
pub struct OwnWrites {
    seen: Mutex<HashMap<PathBuf, u64>>,
}

impl OwnWrites {
    /// Remember that we are about to write these contents to this file.
    pub fn record(&self, path: &Path, contents: &str) {
        if let Ok(mut seen) = self.seen.lock() {
            seen.insert(path.to_path_buf(), digest(contents.as_bytes()));
        }
    }

    /// Whether the file now holds exactly what we last wrote to it, so
    /// this change is the echo of our own save.
    ///
    /// A file somebody else has since changed, or removed, is not an echo,
    /// and its record is dropped: what we wrote is no longer what is
    /// there, so nothing about it can be ours again until we write it
    /// again.
    pub fn is_echo(&self, path: &Path) -> bool {
        let Ok(mut seen) = self.seen.lock() else {
            return false;
        };
        let Some(ours) = seen.get(path).copied() else {
            return false;
        };
        let still_ours = std::fs::read(path).is_ok_and(|contents| digest(&contents) == ours);
        if !still_ours {
            seen.remove(path);
        }
        still_ours
    }
}

/// A hash of file contents, for "is this still what we wrote?".
///
/// Only ever compared with another digest from the same process, so the
/// standard library's hasher is enough and nothing needs a dependency.
fn digest(contents: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    hasher.finish()
}

/// A running watch. Dropping it stops the watch.
pub struct Watch {
    _debouncer: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
}

/// Whether a path is one a reader of the book would care about.
///
/// Anything inside a generated folder is not, and neither is a directory
/// itself — its contents will report themselves. Neither is the temporary
/// file an atomic write goes through: it is created next to its target
/// and renamed away within the same instant, so it changes the folder
/// twice and the book not at all. Every save makes one, which made it the
/// one change nothing could recognise as ours — its name is not the name
/// we recorded.
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
    if is_write_temporary(path) {
        return false;
    }
    !path.is_dir()
}

/// Turn a burst of filesystem events into the paths worth reporting.
///
/// Returns them sorted and deduplicated, project-relative, so two runs of
/// the same change produce the same event (`PLAN.md` §11.5).
///
/// The paths are deduplicated before echoes are checked, so one write
/// costs one question rather than one per event — a burst reports the
/// file that changed, not the three events the filesystem used to say so.
/// Correctness does not rest on that, though: `OwnWrites::is_echo` reads
/// the file, so it answers the same way however the events are grouped,
/// including when the debouncer splits one write across two batches.
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
    fn the_temporary_file_of_our_own_write_is_not_the_book() {
        let root = Path::new("/books/mia");
        assert!(!is_interesting(
            root,
            &root.join("content/.booker-a1b2c3.tmp")
        ));
        // A chapter that merely begins with a dot still is.
        assert!(is_interesting(root, &root.join("content/.hidden.md")));
    }

    /// One write, asked about as many times as the debouncer feels like
    /// splitting it into batches.
    ///
    /// This is the regression: the previous version consumed its record on
    /// the first question and called the second one somebody else's edit,
    /// which on a loaded machine turned every save into a reload that
    /// fought the editor.
    #[test]
    fn our_own_write_stays_ours_however_often_it_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let chapter = dir.path().join("01.md");
        let own = OwnWrites::default();

        own.record(&chapter, "# Saved by us\n");
        std::fs::write(&chapter, "# Saved by us\n").unwrap();

        assert!(own.is_echo(&chapter), "the echo of our own write");
        assert!(own.is_echo(&chapter), "and the rest of the same write");
        assert!(own.is_echo(&chapter), "and still");
    }

    #[test]
    fn an_edit_on_top_of_our_write_is_somebody_elses() {
        let dir = tempfile::tempdir().unwrap();
        let chapter = dir.path().join("01.md");
        let own = OwnWrites::default();

        own.record(&chapter, "# Saved by us\n");
        std::fs::write(&chapter, "# Saved by us\n").unwrap();
        assert!(own.is_echo(&chapter));

        // An agent rewrites the file a moment later. The bytes are no
        // longer ours, whatever the clock says.
        std::fs::write(&chapter, "# Rewritten by somebody else\n").unwrap();
        assert!(!own.is_echo(&chapter), "the file no longer holds our write");
        assert!(
            !own.is_echo(&chapter),
            "and the stale record does not come back"
        );
    }

    #[test]
    fn a_file_removed_after_we_wrote_it_is_not_an_echo() {
        let dir = tempfile::tempdir().unwrap();
        let chapter = dir.path().join("01.md");
        let own = OwnWrites::default();

        own.record(&chapter, "# Saved by us\n");
        std::fs::write(&chapter, "# Saved by us\n").unwrap();
        std::fs::remove_file(&chapter).unwrap();

        assert!(
            !own.is_echo(&chapter),
            "a deletion is a change, not the echo of a write"
        );
    }

    #[test]
    fn a_file_we_never_wrote_is_never_an_echo() {
        let own = OwnWrites::default();
        assert!(!own.is_echo(Path::new("/books/mia/content/02.md")));
    }
}

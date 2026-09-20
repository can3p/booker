//! The watcher, against a real folder and a real filesystem.
//!
//! The unit tests in `watch.rs` cover the decisions — what is worth
//! reporting, what is an echo. This covers the thing those cannot: that an
//! edit made by somebody else actually arrives, once, with the right path
//! on it.
//!
//! **No test here asserts that nothing arrives, and none asserts what the
//! *first* event contains.** Both are assertions about how fast a machine
//! is. Creating the fixture writes eight files, and a watch started right
//! afterwards can still be handed them: macOS CI reported `AGENTS.md` and
//! the template's chapter in a test that had written neither, which is
//! what this file is shaped around.
//!
//! Two **sentinels** replace both assertions, and they work because a
//! watch delivers in the order things happened. Writing a file and
//! waiting for it to be reported therefore proves that everything written
//! before it has already been reported too. So the fixture writes one to
//! drain whatever its own creation produced, and each test ends with
//! another: collect until it arrives, and whatever was supposed to be
//! silent has had its whole chance to speak. "It never appeared" becomes
//! a fact rather than a race, on any machine, at any speed.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use booker_app::watch::{self, OwnWrites};
use booker_core::{ProjectChanged, ProjectRef};
use booker_project::{Project, Template};

/// Long enough that a loaded machine still reports; short enough that a
/// genuine failure does not hold the suite up.
const WAIT: Duration = Duration::from_secs(10);

/// A book, a watch on it, and the events that watch produces.
struct Watched {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
    own: Arc<OwnWrites>,
    events: mpsc::Receiver<ProjectChanged>,
    _watch: watch::Watch,
}

impl Watched {
    fn chapter(&self) -> std::path::PathBuf {
        self.root.join("content").join("01-the-first-chapter.md")
    }

    /// The change every test ends with: a file nothing else touches,
    /// whose arrival means the watcher has caught up with everything
    /// written before it.
    fn touch_sentinel(&self) {
        std::fs::write(self.root.join("content").join("zz-sentinel.md"), "# Go\n")
            .expect("the write");
    }

    /// Wait until the folder's own creation has finished being reported.
    ///
    /// Everything a test does happens after this, so a leaked event from
    /// building the fixture can never be mistaken for one — which is the
    /// mistake macOS CI made, reporting the template's chapter in a test
    /// whose whole assertion was that that chapter stayed quiet.
    fn settle(&self) {
        std::fs::write(self.root.join("content").join("zz-settled.md"), "# Ready\n")
            .expect("the write");
        while !self
            .events
            .recv_timeout(WAIT)
            .expect("the watch reports the file that says it is running")
            .paths
            .iter()
            .any(|path| path.ends_with("zz-settled.md"))
        {}
    }

    /// Collect reported paths until the sentinel arrives.
    ///
    /// Returns everything seen along the way, so a test can assert both
    /// what must be there and what must not.
    fn paths_until_sentinel(&self) -> BTreeSet<String> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        while !seen.iter().any(|path| path.ends_with("zz-sentinel.md")) {
            let changed = self
                .events
                .recv_timeout(WAIT)
                .expect("the sentinel change is reported");
            seen.extend(changed.paths);
        }
        seen
    }
}

fn watched_book() -> Watched {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("mia");
    Project::create(&root, Template::Novel, Some("The Secret Garden of Mia")).expect("a new book");
    let root = root.canonicalize().expect("the folder is there");

    let own = Arc::new(OwnWrites::default());
    let (sender, events) = mpsc::channel();
    let watch = watch::start(ProjectRef::new(&root), Arc::clone(&own), move |changed| {
        let _ = sender.send(changed);
    })
    .expect("the watch starts");

    let watched = Watched {
        _dir: dir,
        root,
        own,
        events,
        _watch: watch,
    };
    watched.settle();
    watched
}

/// Whether any reported path is this file.
fn was_reported(paths: &BTreeSet<String>, file: &Path) -> bool {
    let name = file
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a file name");
    paths.iter().any(|path| path.ends_with(name))
}

#[test]
fn an_edit_made_by_somebody_else_arrives() {
    let watched = watched_book();
    let chapter = watched.chapter();

    std::fs::write(&chapter, "# Rewritten\n\nBy somebody else.\n").expect("the write");
    watched.touch_sentinel();

    let paths = watched.paths_until_sentinel();
    assert!(
        was_reported(&paths, &chapter),
        "an outside edit is reported: {paths:?}"
    );
    assert!(
        paths.iter().all(|path| !path.contains('\\')),
        "paths that cross to the UI use forward slashes: {paths:?}"
    );
}

#[test]
fn thirty_files_rewritten_at_once_is_a_handful_of_events_rather_than_thirty() {
    const FILES: usize = 30;

    let watched = watched_book();
    let content = watched.root.join("content");

    // An agent asked to fix the spelling in a whole book.
    let expected: BTreeSet<String> = (0..FILES)
        .map(|index| {
            let name = format!("{index:02}-chapter.md");
            std::fs::write(
                content.join(&name),
                format!("# Chapter {index}\n\nRewritten.\n"),
            )
            .expect("the write");
            format!("content/{name}")
        })
        .collect();

    // Collect until the burst has been reported in full, counting the
    // events it took.
    //
    // This deliberately does *not* assert a single event. Whether thirty
    // writes land inside one debounce window depends on how fast the
    // machine is, and a CI runner is not fast — asserting "exactly one"
    // asserts something about the hardware. What matters is the property
    // the watcher exists for: the work is proportional to bursts, not to
    // files, because each event costs a reload and a re-render.
    let mut events = 0;
    let mut reported = BTreeSet::new();
    while !expected.is_subset(&reported) {
        let changed = watched
            .events
            .recv_timeout(WAIT)
            .expect("every file that changed is reported");
        events += 1;
        reported.extend(changed.paths);
    }

    // A subset, not an equality: paths are deduplicated within a burst,
    // and a file written just as one burst ends and touched again as the
    // next begins legitimately appears in both. Linux found that — one
    // chapter of the thirty arrived twice — and a file reported twice is
    // not the failure. Losing one would be. Nor does the test care that
    // the watch may also have caught the fixture being created, which is
    // what macOS does and what an equality assertion turned into a
    // failure.
    assert!(
        events <= 5,
        "thirty files should coalesce into a handful of reloads, not {events} ({reported:?})"
    );
}

#[test]
fn a_write_we_made_ourselves_is_not_reported() {
    let watched = watched_book();
    let chapter = watched.chapter();
    let saved = "# Saved By Us\n\nFrom the editor pane.\n";

    // Exactly what `OpenProject::save_chapter` does, through the writer it
    // uses. Writing the file directly would miss the half of a save that
    // the watcher finds hardest: the temporary file an atomic write makes
    // and renames away inside the folder it is watching.
    watched.own.record(&chapter, saved);
    booker_project::write_if_changed(&chapter, saved).expect("the write");
    watched.touch_sentinel();

    let paths = watched.paths_until_sentinel();
    assert!(
        !was_reported(&paths, &chapter),
        "the echo of our own save must not turn into a reload that fights the editor: {paths:?}"
    );
    assert!(
        !paths.iter().any(|path| path.ends_with(".tmp")),
        "nor must the temporary file the save was written through: {paths:?}"
    );
}

#[test]
fn somebody_editing_the_file_we_just_saved_is_still_reported() {
    let watched = watched_book();
    let chapter = watched.chapter();
    let saved = "# Saved By Us\n\nFrom the editor pane.\n";

    watched.own.record(&chapter, saved);
    booker_project::write_if_changed(&chapter, saved).expect("the write");

    // An agent rewrites the same chapter a moment later. Recognising our
    // own write must not turn into ignoring the file for a while
    // afterwards — that would lose somebody's work.
    std::fs::write(&chapter, "# Rewritten\n\nBy somebody else.\n").expect("the write");
    watched.touch_sentinel();

    let paths = watched.paths_until_sentinel();
    assert!(
        was_reported(&paths, &chapter),
        "an edit on top of our save is reported: {paths:?}"
    );
}

#[test]
fn a_build_does_not_look_like_a_change_to_the_book() {
    let watched = watched_book();
    let build = watched.root.join("build");
    std::fs::create_dir_all(&build).expect("the folder");

    std::fs::write(build.join("mia.pdf"), b"%PDF-1.7 not really").expect("the write");
    std::fs::create_dir_all(watched.root.join(".booker")).expect("the folder");
    std::fs::write(watched.root.join(".booker").join("cache"), b"x").expect("the write");
    watched.touch_sentinel();

    let paths = watched.paths_until_sentinel();
    assert!(
        !paths
            .iter()
            .any(|path| path.starts_with("build/") || path.starts_with(".booker/")),
        "generated folders change constantly and mean nothing to the text: {paths:?}"
    );
}

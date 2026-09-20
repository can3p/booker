//! The watcher, against a real folder and a real filesystem.
//!
//! The unit tests in `watch.rs` cover the decisions — what is worth
//! reporting, what is an echo. This covers the thing those cannot: that an
//! edit made by somebody else actually arrives, once, with the right path
//! on it.
//!
//! Timing is involved, so every wait here is generous. A test that fails
//! because a machine was busy teaches nothing.

use std::collections::BTreeSet;
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

    Watched {
        _dir: dir,
        root,
        own,
        events,
        _watch: watch,
    }
}

#[test]
fn an_edit_made_by_somebody_else_arrives() {
    let watched = watched_book();
    let chapter = watched.root.join("content").join("01-the-first-chapter.md");

    std::fs::write(&chapter, "# Rewritten\n\nBy somebody else.\n").expect("the write");

    let changed = watched
        .events
        .recv_timeout(WAIT)
        .expect("an outside edit is reported");
    assert!(
        changed
            .paths
            .iter()
            .any(|path| path.ends_with("01-the-first-chapter.md")),
        "the event names the file that changed: {:?}",
        changed.paths
    );
    assert!(
        changed.paths.iter().all(|path| !path.contains('\\')),
        "paths that cross to the UI use forward slashes: {:?}",
        changed.paths
    );
}

#[test]
fn thirty_files_rewritten_at_once_is_a_handful_of_events_rather_than_thirty() {
    const FILES: usize = 30;

    let watched = watched_book();
    let content = watched.root.join("content");

    // An agent asked to fix the spelling in a whole book.
    for index in 0..FILES {
        std::fs::write(
            content.join(format!("{index:02}-chapter.md")),
            format!("# Chapter {index}\n\nRewritten.\n"),
        )
        .expect("the write");
    }

    // Collect everything the burst produces, then stop when the folder has
    // been quiet for longer than the debounce window.
    //
    // This deliberately does *not* assert a single event. Whether thirty
    // writes land inside one debounce window depends on how fast the
    // machine is, and a CI runner is not fast — asserting "exactly one"
    // asserts something about the hardware. What matters is the property
    // the watcher exists for: the work is proportional to bursts, not to
    // files, because each event costs a reload and a re-render.
    let mut events = 0;
    let mut reported = BTreeSet::new();
    while let Ok(changed) = watched.events.recv_timeout(Duration::from_secs(2)) {
        events += 1;
        reported.extend(changed.paths);
        if reported.len() >= FILES {
            break;
        }
    }

    // A set, not a list: paths are deduplicated within a burst, and a file
    // written just as one burst ends and touched again as the next begins
    // legitimately appears in both. Linux found that — one chapter of the
    // thirty arrived twice — and the file having been reported twice is not
    // the failure. Losing one would be.
    assert_eq!(
        reported.len(),
        FILES,
        "every file that changed should be reported, got {reported:?}"
    );
    assert!(
        events <= 5,
        "thirty files should coalesce into a handful of reloads, not {events}"
    );
}

#[test]
fn a_write_we_made_ourselves_is_not_reported() {
    let watched = watched_book();
    let chapter = watched.root.join("content").join("01-the-first-chapter.md");

    // What `OpenProject::save_chapter` does: say so, then write.
    watched.own.record(&chapter);
    std::fs::write(&chapter, "# Saved By Us\n\nFrom the editor pane.\n").expect("the write");

    assert!(
        watched.events.recv_timeout(Duration::from_secs(2)).is_err(),
        "the echo of our own save must not turn into a reload that fights the editor"
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

    assert!(
        watched.events.recv_timeout(Duration::from_secs(2)).is_err(),
        "generated folders change constantly and mean nothing to the text"
    );
}

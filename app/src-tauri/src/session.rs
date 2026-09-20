//! What the window is holding: the open project, and the engine laying it
//! out.
//!
//! One rule shapes this file. **Nothing here is state that exists only in
//! memory** (`AGENTS.md` §7): everything below can be thrown away and read
//! again from the folder, because an agent may rewrite the project while
//! the window is open and the folder is always right. What is kept is kept
//! for speed, not because it is the truth.
//!
//! The engine is the reason a session exists at all. Typst's compiler is
//! memoized, so the second compile of a book costs a fraction of the first
//! — but only if the same engine does it (`docs/FINDINGS.md`). One engine
//! per open project, alive for as long as the project is open.

use std::path::Path;

use booker_core::{ProjectInfo, Result};
use booker_project::Project;
use booker_typst::Engine;

/// A project the window has open.
pub struct OpenProject {
    project: Project,
    engine: Engine,
}

impl OpenProject {
    /// Load a folder and stand up an engine for it.
    ///
    /// A project with faults in it opens anyway: the faults arrive as
    /// diagnostics in [`OpenProject::info`]. `Err` is for a folder that is
    /// not a project at all — the disk said no.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let project = Project::load(root)?;
        let engine = Engine::open(project.reference().clone())?;
        Ok(Self { project, engine })
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    /// The engine, which callers mutate — `set_book` before every compile.
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Everything the UI needs to describe this project.
    pub fn info(&self) -> ProjectInfo {
        self.project.info()
    }

    /// Hand the engine the book as it currently stands, ready to compile.
    ///
    /// Called after every change rather than once: the engine's memoized
    /// state is what makes the second call cheap, so there is no saving in
    /// being clever about when to skip it.
    pub fn refresh_engine(&mut self) -> Result<()> {
        let chapters: Vec<(&str, &booker_doc::Document)> = self
            .project
            .chapters()
            .iter()
            .map(|chapter| (chapter.source(), chapter.document()))
            .collect();
        let config = self.project.config().clone();
        self.engine.set_book(&config, &chapters)
    }
}

/// The window's state: at most one open project, for now.
///
/// Wave 1 opens one folder at a time. The type is here rather than a bare
/// `Option` so that opening a second window later is a change in one place.
#[derive(Default)]
pub struct Session {
    open: Option<OpenProject>,
}

impl Session {
    /// Open a folder, replacing whatever was open before.
    ///
    /// Replacing drops the previous engine and its memoized layout, which
    /// is correct: that cache belongs to a book nobody is looking at.
    pub fn open(&mut self, root: impl AsRef<Path>) -> Result<ProjectInfo> {
        let mut opened = OpenProject::open(root)?;
        opened.refresh_engine()?;
        let info = opened.info();
        self.open = Some(opened);
        Ok(info)
    }

    /// Close whatever is open. Closing an empty session is not an error —
    /// the window may ask twice.
    pub fn close(&mut self) {
        self.open = None;
    }

    pub fn current(&self) -> Option<&OpenProject> {
        self.open.as_ref()
    }

    pub fn current_mut(&mut self) -> Option<&mut OpenProject> {
        self.open.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use booker_project::Template;

    /// A real project in a temporary folder, the way `booker new` makes one.
    fn a_book() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let root = dir.path().join("mia");
        Project::create(&root, Template::Novel, Some("The Secret Garden of Mia"))
            .expect("a new book");
        dir
    }

    #[test]
    fn opening_a_folder_describes_the_book_in_it() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");

        assert_eq!(info.config.title, "The Secret Garden of Mia");
        assert_eq!(info.chapters.len(), 1);
        assert!(
            info.chapters[0].path.contains('/'),
            "paths that cross to the UI are project-relative with forward slashes, got {}",
            info.chapters[0].path
        );
        assert!(session.current().is_some());
    }

    #[test]
    fn closing_forgets_the_project() {
        let dir = a_book();
        let mut session = Session::default();
        session.open(dir.path().join("mia")).expect("it opens");
        session.close();
        assert!(session.current().is_none());
        // Closing twice is what a window does when the user is quick.
        session.close();
    }

    #[test]
    fn a_folder_that_is_not_a_project_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let mut session = Session::default();
        assert!(session.open(dir.path().join("nothing-here")).is_err());
    }
}

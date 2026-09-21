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

use std::path::{Path, PathBuf};
use std::sync::Arc;

use booker_core::{
    display_path, ChapterText, CompileRequest, CompileResult, CompileTarget, Error, PagePoint,
    ProjectInfo, Result, SaveOutcome, SourceLocation,
};
use booker_project::Project;
use booker_typst::{Compilation, Engine};

use crate::watch::{OwnWrites, Watch};

/// A project the window has open.
pub struct OpenProject {
    project: Project,
    engine: Engine,
    /// The writes this process made, so the watcher can drop their echoes.
    own_writes: Arc<OwnWrites>,
    /// The running watch on the folder, if one was started. Dropping it
    /// stops watching, which is what closing a project does.
    watch: Option<Watch>,
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
        Ok(Self {
            project,
            engine,
            own_writes: Arc::new(OwnWrites::default()),
            watch: None,
        })
    }

    /// The record of our own writes, which the watcher needs to tell an
    /// outside edit from the echo of a save.
    pub fn own_writes(&self) -> Arc<OwnWrites> {
        Arc::clone(&self.own_writes)
    }

    /// Hold on to a running watch for as long as this project is open.
    pub fn set_watch(&mut self, watch: Watch) {
        self.watch = Some(watch);
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

    /// Lay the book out.
    ///
    /// A book with errors in it still returns a result — with no pages and
    /// the errors in `diagnostics`. `Err` is for a request the engine
    /// cannot answer at all.
    pub fn compile(&mut self, target: CompileTarget) -> Result<Compilation> {
        self.refresh_engine()?;
        let request = CompileRequest {
            project: self.project.reference().clone(),
            target,
            revision: self.project.revision(),
        };
        self.engine.compile(&request)
    }

    /// Lay the book out and write the PDF where the user asked for it.
    ///
    /// The destination comes from a file dialog rather than from us, so it
    /// is the one path in the application that legitimately points outside
    /// the project folder.
    pub fn export_pdf(&mut self, destination: &Path) -> Result<CompileResult> {
        let compilation = self.compile(CompileTarget::Pdf)?;
        let Some(pdf) = compilation.pdf else {
            // The export failed, and the compilation says why. Handing back
            // the diagnostics beats inventing a message here.
            return Ok(compilation.result);
        };
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|error| Error::Project {
                path: parent.to_path_buf(),
                message: format!("could not create the folder: {error}"),
            })?;
        }
        booker_project::write_atomic(destination, &pdf)?;
        Ok(compilation.result)
    }

    /// One chapter's source, on its way to the editor pane.
    pub fn read_chapter(&self, path: &str) -> Result<ChapterText> {
        let Some(chapter) = self.project.chapter(path) else {
            return Err(Error::Project {
                path: PathBuf::from(path),
                message: "this book has no such chapter".to_string(),
            });
        };
        Ok(ChapterText {
            project: self.project.reference().clone(),
            path: path.to_string(),
            text: chapter.source().to_string(),
            revision: self.project.revision(),
        })
    }

    /// Write a chapter back — unless the file changed underneath.
    ///
    /// `base` is the text the editor last loaded or saved. When the file on
    /// disk is neither that nor `text`, somebody else changed it — another
    /// editor, an agent, a branch checkout — and writing now would silently
    /// throw their change away. Nothing is written, and both versions go
    /// back to the window for the person to choose (`AGENTS.md` §7). The
    /// file is read fresh from disk for this, because the watcher's reload
    /// may not have arrived yet; that race is exactly the case that matters.
    ///
    /// Saving an unchanged chapter writes nothing and moves nothing, so the
    /// eager autosave the editor does costs one comparison rather than a
    /// write and a reload.
    pub fn save_chapter(&mut self, path: &str, text: &str, base: &str) -> Result<SaveOutcome> {
        let Some(chapter) = self.project.chapter(path) else {
            return Err(Error::Project {
                path: PathBuf::from(path),
                message: "this book has no such chapter".to_string(),
            });
        };
        let file = chapter.path().to_path_buf();
        let disk = match std::fs::read_to_string(&file) {
            Ok(disk) => disk,
            // Deleted underneath: that is a change too, and "keep mine"
            // puts the file back.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };
        if disk != base && disk != text {
            self.project.reload()?;
            return Ok(SaveOutcome::Conflict {
                disk: ChapterText {
                    project: self.project.reference().clone(),
                    path: path.to_string(),
                    text: disk,
                    revision: self.project.revision(),
                },
                mine: text.to_string(),
            });
        }

        // Say what we are about to write before writing it, so the change
        // event this causes is recognised as ours and does not turn into a
        // reload that fights the editor.
        self.own_writes.record(&file, text);
        self.project.write_chapter(path, text)?;
        Ok(SaveOutcome::Saved {
            info: Box::new(self.project.info()),
        })
    }

    /// What was drawn at `point`, as a place in a chapter — click-to-source.
    /// `None` for a margin, a page number, the table of contents.
    ///
    /// Answered from the last layout: the window only offers pages it has
    /// already been shown, so there is always one.
    pub fn source_at(&self, point: &PagePoint) -> Option<SourceLocation> {
        let (chapter, offset) = self.engine.source_at(point)?;
        let chapter = self.project.chapters().get(chapter)?;
        Some(chapter.location(booker_doc::Span::new(offset, offset)))
    }

    /// Where a place in a chapter landed on the pages — the cursor's page.
    /// Uses the byte span when the location has one, and the line and
    /// column otherwise. Empty when nothing there is printed.
    pub fn pages_at(&self, location: &SourceLocation) -> Vec<PagePoint> {
        let file = display_path(&location.file);
        let Some(index) = self
            .project
            .chapters()
            .iter()
            .position(|chapter| display_path(chapter.relative_path()) == file)
        else {
            return Vec::new();
        };
        let chapter = &self.project.chapters()[index];
        let offset = match location.span {
            Some((start, _)) => Some(start),
            None => chapter.offset_at(location.line, location.column),
        };
        offset
            .map(|offset| self.engine.pages_at(index, offset))
            .unwrap_or_default()
    }

    // The chapter tree. These rewrite `book.toml` (and, for `add` and
    // `rename`, a chapter file) without recording the write for the
    // watcher: the echo arrives as an ordinary reload, which is harmless —
    // the window compares what it reloads with what it holds, and nothing
    // unsaved is at stake in a sidebar edit.

    /// Add a chapter; returns its path and the book as it now is.
    pub fn add_chapter(
        &mut self,
        after: Option<&str>,
        title: &str,
        text: Option<&str>,
    ) -> Result<(String, ProjectInfo)> {
        let path = self.project.add_chapter(after, title, text)?;
        Ok((path, self.project.info()))
    }

    pub fn move_chapter(&mut self, path: &str, index: usize) -> Result<ProjectInfo> {
        self.project.move_chapter(path, index)?;
        Ok(self.project.info())
    }

    /// Take a chapter out of the book; its file stays in the folder.
    pub fn remove_chapter(&mut self, path: &str) -> Result<ProjectInfo> {
        self.project.remove_chapter(path)?;
        Ok(self.project.info())
    }

    pub fn rename_chapter(&mut self, path: &str, title: &str) -> Result<ProjectInfo> {
        self.project.rename_chapter(path, title)?;
        Ok(self.project.info())
    }

    /// Read the folder again, after something outside changed it.
    pub fn reload(&mut self) -> Result<ProjectInfo> {
        self.project.reload()?;
        // The engine keeps its memoized state: most of a book is unchanged
        // even when a file was rewritten, and that is what makes the
        // re-render after an outside edit as cheap as one after a
        // keystroke.
        self.refresh_engine()?;
        Ok(self.project.info())
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
    fn a_book_lays_out_into_pages() {
        let dir = a_book();
        let mut session = Session::default();
        session.open(dir.path().join("mia")).expect("it opens");

        let compilation = session
            .current_mut()
            .unwrap()
            .compile(CompileTarget::Layout)
            .expect("it compiles");

        assert!(!compilation.result.pages.is_empty(), "a book has pages");
        assert!(!compilation.has_errors(), "the starter book has no errors");
        assert!(
            compilation.pdf.is_none(),
            "a layout was asked for, not a file"
        );
    }

    #[test]
    fn exporting_writes_a_pdf_where_it_was_asked_to() {
        let dir = a_book();
        let mut session = Session::default();
        session.open(dir.path().join("mia")).expect("it opens");
        // Somewhere outside the project, as a file dialog would give us,
        // and inside a folder that does not exist yet.
        let destination = dir.path().join("exports").join("mia.pdf");

        let result = session
            .current_mut()
            .unwrap()
            .export_pdf(&destination)
            .expect("it exports");

        assert!(!result.pages.is_empty());
        let bytes = std::fs::read(&destination).expect("the file is there");
        assert!(bytes.starts_with(b"%PDF"), "and it is a PDF");
    }

    #[test]
    fn a_chapter_can_be_read_and_written_back() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");
        let path = info.chapters[0].path.clone();

        let chapter = session
            .current()
            .unwrap()
            .read_chapter(&path)
            .expect("the chapter the sidebar named");
        assert!(chapter.text.contains("The First Chapter"));

        let outcome = session
            .current_mut()
            .unwrap()
            .save_chapter(
                &path,
                "# Chapter the First\n\nIt was a bright day.\n",
                &chapter.text,
            )
            .expect("it saves");
        let SaveOutcome::Saved { info: after } = outcome else {
            panic!("nothing changed underneath, so it saves: {outcome:?}");
        };

        assert_eq!(
            after.chapters[0].title.as_deref(),
            Some("Chapter the First"),
            "what comes back describes the book as it now is"
        );
        assert!(after.revision > chapter.revision);
    }

    #[test]
    fn a_save_over_a_file_changed_underneath_is_refused_and_both_versions_come_back() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");
        let path = info.chapters[0].path.clone();
        let open = session.current_mut().unwrap();
        let base = open.read_chapter(&path).unwrap().text;

        // An agent rewrites the chapter while the window has unsaved typing.
        let file = dir.path().join("mia").join(&path);
        std::fs::write(&file, "# Theirs\n\nFixed the spelling.\n").unwrap();

        let outcome = open
            .save_chapter(&path, "# Mine\n\nNew paragraph.\n", &base)
            .expect("a conflict is an answer, not a failure");
        let SaveOutcome::Conflict { disk, mine } = outcome else {
            panic!("the file changed underneath: {outcome:?}");
        };
        assert_eq!(disk.text, "# Theirs\n\nFixed the spelling.\n");
        assert_eq!(mine, "# Mine\n\nNew paragraph.\n");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "# Theirs\n\nFixed the spelling.\n",
            "nothing was written over their change"
        );

        // Keeping mine is a save against what is on disk now.
        let kept = open
            .save_chapter(&path, "# Mine\n\nNew paragraph.\n", &disk.text)
            .unwrap();
        assert!(matches!(kept, SaveOutcome::Saved { .. }), "{kept:?}");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "# Mine\n\nNew paragraph.\n"
        );
    }

    #[test]
    fn saving_what_is_already_on_disk_is_never_a_conflict() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");
        let path = info.chapters[0].path.clone();
        let open = session.current_mut().unwrap();
        // The same edit made in two places — the file already says it.
        let file = dir.path().join("mia").join(&path);
        std::fs::write(&file, "# Same\n").unwrap();
        let outcome = open.save_chapter(&path, "# Same\n", "# Old\n").unwrap();
        assert!(matches!(outcome, SaveOutcome::Saved { .. }), "{outcome:?}");
    }

    #[test]
    fn keeping_both_versions_adds_mine_as_a_chapter_after_theirs() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");
        let path = info.chapters[0].path.clone();
        let (added, after) = session
            .current_mut()
            .unwrap()
            .add_chapter(
                Some(&path),
                "The First Chapter (my version)",
                Some("# Mine\n"),
            )
            .unwrap();
        assert_eq!(after.chapters.len(), 2);
        assert_eq!(after.chapters[1].path, added);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("mia").join(&added)).unwrap(),
            "# Mine\n"
        );
    }

    #[test]
    fn a_line_leads_to_its_page_and_the_page_back_to_the_line() {
        let dir = a_book();
        let mut session = Session::default();
        let info = session.open(dir.path().join("mia")).expect("it opens");
        let open = session.current_mut().unwrap();
        open.compile(CompileTarget::Layout).unwrap();

        let here = SourceLocation {
            file: PathBuf::from(&info.chapters[0].path),
            line: 3,
            column: 1,
            span: None,
        };
        let pages = open.pages_at(&here);
        assert_eq!(pages.len(), 1, "{pages:?}");
        assert_eq!(
            pages[0].page, 0,
            "a one-chapter book starts on the first page"
        );

        // A hair inside the first glyph of that line.
        let click = PagePoint {
            page: 0,
            x: booker_core::Length::mm(pages[0].x.to_mm() + 0.4),
            y: booker_core::Length::mm(pages[0].y.to_mm() - 1.0),
        };
        let back = open.source_at(&click).expect("text is under the click");
        assert_eq!(back.file, PathBuf::from(&info.chapters[0].path));
        assert_eq!(back.line, 3);
    }

    #[test]
    fn a_place_in_a_file_that_is_not_a_chapter_is_on_no_page() {
        let dir = a_book();
        let mut session = Session::default();
        session.open(dir.path().join("mia")).expect("it opens");
        let open = session.current_mut().unwrap();
        open.compile(CompileTarget::Layout).unwrap();
        let nowhere = SourceLocation {
            file: PathBuf::from("content/nope.md"),
            line: 1,
            column: 1,
            span: None,
        };
        assert!(open.pages_at(&nowhere).is_empty());
    }

    #[test]
    fn reading_a_chapter_outside_the_book_is_refused() {
        let dir = a_book();
        let mut session = Session::default();
        session.open(dir.path().join("mia")).expect("it opens");

        let error = session
            .current()
            .unwrap()
            .read_chapter("../../../etc/passwd")
            .expect_err("not a chapter of this book");
        assert!(error.to_string().contains("no such chapter"));
    }

    #[test]
    fn a_folder_that_is_not_a_project_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let mut session = Session::default();
        assert!(session.open(dir.path().join("nothing-here")).is_err());
    }
}

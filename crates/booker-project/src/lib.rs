//! Loading, watching and writing a project folder.
//!
//! Owned by Wave 0 track E. The rules this crate exists to enforce
//! (`AGENTS.md` §7): writes are atomic and targeted, comments and unknown
//! keys survive a round trip, opening a project changes nothing on disk, and
//! a broken project still loads as far as it can.
//!
//! ```no_run
//! use booker_project::Project;
//!
//! let project = Project::load("my-book")?;
//! println!("{} — {} chapters", project.config().title, project.chapters().len());
//! for diagnostic in project.diagnostics() {
//!     println!("{}", booker_project::format_diagnostic(diagnostic));
//! }
//! # Ok::<(), booker_core::Error>(())
//! ```

mod config;
mod content;
mod edit;
mod migrate;
pub mod rules;
mod template;
mod toml_tree;
mod write;

use std::path::{Path, PathBuf};

pub use booker_core::{
    display_path, BookConfig, ChapterSummary, Diagnostic, Error, ProjectInfo, ProjectRef, Result,
    Revision, Severity,
};

pub use config::{ConfigFile, BOOK_TOML};
pub use content::Chapter;
pub use edit::ConfigEditor;
pub use migrate::{Migration, Migrations};
pub use template::{Created, Template};
pub use write::{is_write_temporary, write_atomic, write_if_changed, Written};

/// A project folder, loaded.
///
/// Loading never refuses because of what is *in* the project: a missing
/// `book.toml`, a value that makes no sense, a chapter that is not there —
/// each becomes a [`Diagnostic`] with a file, a line and a column, and the
/// rest still loads. That is exactly when a person needs to see the errors
/// (`PLAN.md` §11.1).
pub struct Project {
    reference: ProjectRef,
    config: BookConfig,
    config_file: ConfigFile,
    chapters: Vec<Chapter>,
    diagnostics: Vec<Diagnostic>,
    revision: Revision,
}

impl Project {
    /// Load the project rooted at `root`.
    ///
    /// The only errors are about the folder itself — it is not there, or it
    /// is not a folder. Everything else is a diagnostic.
    pub fn load(root: impl AsRef<Path>) -> Result<Project> {
        let given = root.as_ref();
        if !given.exists() {
            return Err(Error::Project {
                path: given.to_path_buf(),
                message: "there is no folder here".to_string(),
            });
        }
        if !given.is_dir() {
            return Err(Error::Project {
                path: given.to_path_buf(),
                message: "a Booker project is a folder, and this is a file".to_string(),
            });
        }
        // Canonicalise, so every path we hand out is absolute and
        // comparable whatever the caller passed in.
        let root = given.canonicalize().unwrap_or_else(|_| given.to_path_buf());

        let mut diagnostics = Vec::new();
        let config_file = ConfigFile::load(&root, &mut diagnostics);
        let (config, config_diagnostics) = config_file.typed();
        diagnostics.extend(config_diagnostics);

        let chapters = content::discover(&root, &config, &config_file, &mut diagnostics);
        for chapter in &chapters {
            content::check_images(&root, chapter, &mut diagnostics);
            content::check_document(chapter, &mut diagnostics);
        }

        sort_diagnostics(&mut diagnostics);

        Ok(Project {
            reference: ProjectRef::new(root),
            config,
            config_file,
            chapters,
            diagnostics,
            revision: Revision::default(),
        })
    }

    /// Create a starter project and load it.
    pub fn create(
        root: impl AsRef<Path>,
        template: Template,
        title: Option<&str>,
    ) -> Result<(Created, Project)> {
        let root = root.as_ref();
        std::fs::create_dir_all(root).map_err(|error| Error::Project {
            path: root.to_path_buf(),
            message: format!("could not create the folder: {error}"),
        })?;
        let created = template::create(root, template, title)?;
        let project = Project::load(root)?;
        Ok((created, project))
    }

    pub fn root(&self) -> &Path {
        &self.reference.root
    }

    pub fn reference(&self) -> &ProjectRef {
        &self.reference
    }

    /// The typed view of `book.toml`, with defaults filled in for anything
    /// that could not be read.
    pub fn config(&self) -> &BookConfig {
        &self.config
    }

    pub fn config_file(&self) -> &ConfigFile {
        &self.config_file
    }

    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    /// Everything wrong with the project, in the order a person reads a
    /// file: by file, then by line. Always sorted, so two runs produce the
    /// same output (`PLAN.md` §11.5).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// One chapter's headline numbers, in the shape that crosses to the UI.
    ///
    /// The window's sidebar and `booker build`'s chapter list are the same
    /// question asked twice, so they are answered once, here (`AGENTS.md`
    /// §6).
    pub fn chapter_summaries(&self) -> Vec<ChapterSummary> {
        self.chapters
            .iter()
            .map(|chapter| ChapterSummary {
                path: display_path(chapter.relative_path()),
                title: chapter.title(),
                words: chapter.word_count(),
                headings: chapter.document().headings().len(),
                images: chapter.document().images().len(),
            })
            .collect()
    }

    /// Everything the application needs to describe a project it has just
    /// opened, before anything has been laid out.
    ///
    /// A project with faults in it still produces one of these, with the
    /// faults in `diagnostics`: refusing to open is never right
    /// (`AGENTS.md` §7).
    pub fn info(&self) -> ProjectInfo {
        ProjectInfo {
            project: self.reference().clone(),
            config: self.config().clone(),
            chapters: self.chapter_summaries(),
            diagnostics: self.diagnostics().to_vec(),
            revision: self.revision(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    /// Which revision of the project this is. Bumped by every write we make;
    /// the file watcher (Wave 1) bumps it for changes made from outside.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// The chapter at this project-relative path, if the book has one.
    ///
    /// The path is matched against the chapters the project actually has,
    /// which is also the containment check: a path that is not a chapter of
    /// this book is not found, whatever it points at (`PLAN.md` §11.3).
    pub fn chapter(&self, relative: &str) -> Option<&Chapter> {
        self.chapters
            .iter()
            .find(|chapter| display_path(chapter.relative_path()) == relative)
    }

    /// Write a chapter's text and re-read the project.
    ///
    /// The write is atomic — a temporary file and a rename — so a reader
    /// coming in at the wrong moment sees the old file or the new one, never
    /// half of either (`AGENTS.md` §7). The re-read afterwards is what makes
    /// the word counts, the diagnostics and the revision agree with what is
    /// now on disk.
    ///
    /// Writing nothing is not a write: an unchanged chapter leaves the file
    /// and the revision alone, which is what stops a save the user did not
    /// make from waking every watcher in the system.
    pub fn write_chapter(&mut self, relative: &str, text: &str) -> Result<Written> {
        let Some(chapter) = self.chapter(relative) else {
            return Err(Error::Project {
                path: PathBuf::from(relative),
                message: "this book has no such chapter".to_string(),
            });
        };
        let path = chapter.path().to_path_buf();
        let written = write_if_changed(&path, text)?;
        if written.changed() {
            self.reload()?;
        }
        Ok(written)
    }

    /// Read the project from disk again, keeping the revision counter
    /// moving forward.
    ///
    /// This is what an outside change arrives as: an agent rewriting files,
    /// a branch checkout, our own write. Everything the project holds is
    /// derived from the folder, so there is nothing to merge — the folder
    /// is simply right (`AGENTS.md` §7).
    pub fn reload(&mut self) -> Result<()> {
        let reloaded = Project::load(self.root())?;
        let revision = self.revision.next();
        *self = Project {
            revision,
            ..reloaded
        };
        Ok(())
    }

    /// Make a targeted change to `book.toml`, in memory.
    ///
    /// Nothing reaches the disk until [`Project::save`]. The typed view is
    /// re-read afterwards, so `config()` and the file never disagree.
    pub fn edit_config(&mut self, edit: impl FnOnce(&mut ConfigEditor<'_>)) {
        {
            let mut editor = ConfigEditor::new(self.config_file.document_mut());
            edit(&mut editor);
        }
        self.config_file.refresh_spans();
        self.reread_config();
    }

    /// Run any migrations this build has for the project's format version.
    ///
    /// Nothing reaches the disk until [`Project::save`]. Returns what ran —
    /// empty for every project today, because format 1 is the first format
    /// and [`Migrations::builtin`] is deliberately empty.
    pub fn migrate(&mut self) -> Result<Vec<&'static str>> {
        if !self.config_file.is_parsed() {
            return Err(Error::Project {
                path: PathBuf::from(BOOK_TOML),
                message: "cannot migrate a file that could not be parsed; fix the TOML first"
                    .to_string(),
            });
        }
        let from = self.config.format;
        let applied = Migrations::builtin().run(self.config_file.document_mut(), from)?;
        if !applied.is_empty() {
            self.config_file.refresh_spans();
            self.reread_config();
        }
        Ok(applied)
    }

    /// Write `book.toml` back, atomically, and only if something changed.
    ///
    /// Refuses to write a file that did not parse: replacing what a person
    /// wrote with our idea of an empty document would lose their work, and
    /// the diagnostic already says where the syntax error is.
    pub fn save(&mut self) -> Result<Written> {
        if !self.config_file.is_parsed() {
            return Err(Error::Project {
                path: PathBuf::from(BOOK_TOML),
                message: format!(
                    "`{BOOK_TOML}` could not be parsed ({}), so Booker will not write over it",
                    rules::INVALID_TOML
                ),
            });
        }
        let rendered = self.config_file.rendered();
        let written = write_if_changed(self.config_file.path(), &rendered)?;
        if written.changed() {
            self.config_file.mark_written(rendered);
            self.revision = self.revision.next();
        }
        Ok(written)
    }

    /// The path of `file` as it should appear to a user: relative to the
    /// project root.
    pub fn relative<'a>(&self, file: &'a Path) -> &'a Path {
        self.reference.relative(file)
    }

    fn reread_config(&mut self) {
        let (config, mut diagnostics) = self.config_file.typed();
        self.config = config;
        // Keep the diagnostics that did not come from `book.toml`; the ones
        // that did are replaced by what the edited document says now.
        self.diagnostics.retain(|diagnostic| {
            diagnostic
                .source
                .as_ref()
                .map(|source| source.file != Path::new(BOOK_TOML))
                .unwrap_or(true)
        });
        self.diagnostics.append(&mut diagnostics);
        sort_diagnostics(&mut self.diagnostics);
    }
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by_cached_key(|diagnostic| {
        let source = diagnostic.source.as_ref();
        (
            source.map(|source| source.file.clone()).unwrap_or_default(),
            source.map_or(0, |source| source.line),
            source.map_or(0, |source| source.column),
            diagnostic.rule.clone(),
        )
    });
}

/// One diagnostic on one line, the way the CLI, CI and an agent all want to
/// read it: `content/03.md:145:1: error[BK-REF-001] image … does not exist`.
pub fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };
    let place = match &diagnostic.source {
        Some(source) => format!(
            "{}:{}:{}",
            display_path(&source.file),
            source.line,
            source.column
        ),
        None => "<project>".to_string(),
    };
    format!(
        "{place}: {severity}[{}] {}",
        diagnostic.rule, diagnostic.message
    )
}

//! The [`World`] Typst compiles against: one project folder, plus whatever
//! the editor is holding unsaved.
//!
//! Two rules shape this file, both from `AGENTS.md` §7:
//!
//! * **Nothing is locked and nothing lives only in memory.** The folder on
//!   disk is the truth. An unsaved buffer is an *override* on top of it, and
//!   dropping the override falls straight back to the file.
//! * **Reads stay inside the project.** A `#include "../../.ssh/id_rsa"` in a
//!   book gets an error, not the file.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use booker_core::{Error, Result};
use typst::diag::{FileError, FileResult, PackageError};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::files::{FileLoader, FileStore};

use crate::fonts::Fonts;

/// The entry file of a project when nothing else is configured.
///
/// Wave 0 takes Typst source directly. From Wave 1 this file is generated
/// from the document model instead of being written by hand.
pub const DEFAULT_ENTRYPOINT: &str = "main.typ";

/// A Typst world over one Booker project.
pub struct BookerWorld {
    root: PathBuf,
    entrypoint: PathBuf,
    main: FileId,
    library: LazyHash<Library>,
    fonts: Fonts,
    files: FileStore<ProjectFiles>,
    now: Time,
}

impl BookerWorld {
    /// Opens a project folder.
    ///
    /// The entry point is a path inside the project; it does not have to
    /// exist yet. A project whose entry file is missing must still open — the
    /// missing file comes back as a diagnostic from the first compile, which
    /// is exactly when the user needs to be told about it.
    pub fn new(root: impl Into<PathBuf>, entrypoint: impl AsRef<Path>) -> Result<Self> {
        let root = root.into();
        let entrypoint = entrypoint.as_ref();
        let vpath = virtual_path(&root, entrypoint)?;
        let entrypoint = PathBuf::from(vpath.get_without_slash());
        let main = RootedPath::new(VirtualRoot::Project, vpath).intern();

        Ok(Self {
            fonts: Fonts::load(&root),
            files: FileStore::new(ProjectFiles {
                root: root.clone(),
                overrides: HashMap::new(),
            }),
            root,
            entrypoint,
            main,
            library: LazyHash::new(Library::default()),
            now: Time::system(),
        })
    }

    /// The project folder.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The entry file, relative to the project folder.
    pub fn entrypoint(&self) -> &Path {
        &self.entrypoint
    }

    /// The faces this project can use.
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// Hands Typst the text an editor is holding for a file instead of what
    /// is on disk. Call it on every keystroke; the compiler edits its parsed
    /// copy in place rather than reparsing the file.
    pub fn set_override(&mut self, path: impl AsRef<Path>, text: impl Into<String>) -> Result<()> {
        let id = self.id_for(path.as_ref())?;
        let bytes = Bytes::from_string(text.into());
        self.files.loader_mut().overrides.insert(id, bytes);
        Ok(())
    }

    /// Forgets an unsaved buffer, so the file on disk speaks for itself again.
    /// Returns whether there was one.
    pub fn clear_override(&mut self, path: impl AsRef<Path>) -> Result<bool> {
        let id = self.id_for(path.as_ref())?;
        Ok(self.files.loader_mut().overrides.remove(&id).is_some())
    }

    /// Forgets every unsaved buffer at once — what a project reload does.
    pub fn clear_overrides(&mut self) {
        self.files.loader_mut().overrides.clear();
    }

    /// Whether a file currently has an unsaved buffer over it.
    pub fn has_override(&self, path: impl AsRef<Path>) -> bool {
        match self.id_for(path.as_ref()) {
            Ok(id) => self.files.loader().overrides.contains_key(&id),
            Err(_) => false,
        }
    }

    /// Marks every file stale before a compile, so that anything an outside
    /// agent rewrote is picked up.
    ///
    /// Parsed sources are kept and edited in place rather than thrown away;
    /// that is what makes the second compile cheap.
    pub fn reset(&mut self) {
        self.files.reset();
        self.now.reset();
    }

    /// The file id of a path inside the project.
    pub fn id_for(&self, path: impl AsRef<Path>) -> Result<FileId> {
        let vpath = virtual_path(&self.root, path.as_ref())?;
        Ok(RootedPath::new(VirtualRoot::Project, vpath).intern())
    }

    /// How a file id is named in output: relative to the project root, so two
    /// machines print the same thing (`PLAN.md` §11.5).
    pub fn display_path(&self, id: FileId) -> PathBuf {
        match id.root() {
            VirtualRoot::Project => PathBuf::from(id.vpath().get_without_slash()),
            VirtualRoot::Package(spec) => {
                PathBuf::from(format!("{spec}{}", id.vpath().get_with_slash()))
            }
        }
    }
}

impl World for BookerWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<typst::syntax::Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.face(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.now.today(offset)
    }
}

/// Serves files from the project folder, and from unsaved buffers on top of it.
struct ProjectFiles {
    root: PathBuf,
    overrides: HashMap<FileId, Bytes>,
}

impl FileLoader for ProjectFiles {
    fn load(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(bytes) = self.overrides.get(&id) {
            return Ok(bytes.clone());
        }

        match id.root() {
            VirtualRoot::Project => {}
            VirtualRoot::Package(spec) => {
                // Packages would mean downloading code from the internet
                // while laying out someone's book. If Booker ever ships
                // packages they will be vendored into the build, not fetched.
                return Err(FileError::Package(PackageError::Other(Some(
                    format!("`{spec}` is a Typst package; Booker does not download packages")
                        .into(),
                ))));
            }
        }

        let path = id.vpath().realize(&self.root)?;
        let metadata = std::fs::metadata(&path).map_err(|err| FileError::from_io(err, &path))?;
        if metadata.is_dir() {
            return Err(FileError::IsDirectory);
        }
        std::fs::read(&path)
            .map(Bytes::new)
            .map_err(|err| FileError::from_io(err, &path))
    }
}

/// Turns a path a caller gave us into a path inside the project.
///
/// Absolute paths must be under the root; relative paths are taken to be
/// relative to it. `..` is refused rather than normalised away, because a
/// path that climbs out of the project is a mistake worth naming.
fn virtual_path(root: &Path, path: &Path) -> Result<VirtualPath> {
    let escapes = |message: &str| Error::Project {
        path: path.to_path_buf(),
        message: message.to_string(),
    };

    let relative = if path.is_absolute() {
        path.strip_prefix(root)
            .map_err(|_| escapes("this file is outside the project folder"))?
    } else {
        path
    };

    let mut segments = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| escapes("file names must be valid UTF-8"))?;
                segments.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(escapes("this file is outside the project folder"));
            }
        }
    }

    if segments.is_empty() {
        return Err(escapes("this is the project folder, not a file in it"));
    }

    VirtualPath::new(segments.join("/")).map_err(|err| escapes(&err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relative_path_names_a_file_in_the_project() {
        let vpath = virtual_path(Path::new("/books/mia"), Path::new("content/01.typ")).unwrap();
        assert_eq!(vpath.get_without_slash(), "content/01.typ");
    }

    #[test]
    fn an_absolute_path_inside_the_project_is_accepted() {
        let vpath = virtual_path(
            Path::new("/books/mia"),
            Path::new("/books/mia/content/01.typ"),
        )
        .unwrap();
        assert_eq!(vpath.get_without_slash(), "content/01.typ");
    }

    #[test]
    fn a_path_that_climbs_out_of_the_project_is_refused() {
        for path in ["../secrets.typ", "content/../../secrets.typ"] {
            let err = virtual_path(Path::new("/books/mia"), Path::new(path)).unwrap_err();
            assert!(
                err.to_string().contains("outside the project"),
                "{path}: {err}"
            );
        }
    }

    #[test]
    fn an_absolute_path_elsewhere_is_refused() {
        let err = virtual_path(Path::new("/books/mia"), Path::new("/etc/passwd")).unwrap_err();
        assert!(err.to_string().contains("outside the project"), "{err}");
    }

    #[test]
    fn the_same_file_always_gets_the_same_id() {
        let world = BookerWorld::new("/books/mia", DEFAULT_ENTRYPOINT).unwrap();
        assert_eq!(world.main(), world.id_for("main.typ").unwrap());
        assert_eq!(
            world.id_for("content/01.typ").unwrap(),
            world.id_for("./content/01.typ").unwrap()
        );
        assert_ne!(
            world.id_for("content/01.typ").unwrap(),
            world.id_for("content/02.typ").unwrap()
        );
    }

    #[test]
    fn file_ids_come_back_as_project_relative_paths() {
        let world = BookerWorld::new("/books/mia", DEFAULT_ENTRYPOINT).unwrap();
        let id = world.id_for("content/01.typ").unwrap();
        assert_eq!(world.display_path(id), Path::new("content/01.typ"));
    }
}

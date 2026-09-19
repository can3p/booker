//! Which fonts a project can use.
//!
//! Two sources, in this order:
//!
//! 1. the project's own `assets/fonts/`, so a book carries its typefaces with
//!    it and renders the same on someone else's machine;
//! 2. the fonts bundled with Booker, so a project that configures nothing
//!    still produces a book.
//!
//! System fonts are deliberately **not** searched. A book that looks right
//! only on the machine it was written on is a bug we would rather not ship;
//! `PLAN.md` §6 gives this to the problems panel instead ("this project uses
//! a system-only font — copy it into the project"), which arrives with the
//! style work in a later wave.

use std::path::{Path, PathBuf};

use booker_core::diagnostic::{Diagnostic, Severity, SourceLocation};
use booker_core::display_path;
use typst::foundations::Bytes;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;

/// The fonts Booker ships with. Licences are in `fonts/NOTICE.txt`.
const BUNDLED: &[(&str, &[u8])] = &[
    (
        "LibertinusSerif-Regular.otf",
        include_bytes!("../fonts/LibertinusSerif-Regular.otf"),
    ),
    (
        "LibertinusSerif-Italic.otf",
        include_bytes!("../fonts/LibertinusSerif-Italic.otf"),
    ),
    (
        "LibertinusSerif-Bold.otf",
        include_bytes!("../fonts/LibertinusSerif-Bold.otf"),
    ),
    (
        "LibertinusSerif-BoldItalic.otf",
        include_bytes!("../fonts/LibertinusSerif-BoldItalic.otf"),
    ),
    (
        "DejaVuSansMono.ttf",
        include_bytes!("../fonts/DejaVuSansMono.ttf"),
    ),
];

/// Where a project keeps the fonts it ships with itself.
pub const PROJECT_FONT_DIR: &str = "assets/fonts";

/// How deep below `assets/fonts` we look. Deep enough for `assets/fonts/
/// MyFamily/Weight/file.otf`, shallow enough that a symlink loop cannot hang
/// the app.
const MAX_DEPTH: usize = 6;

/// A font file a project shipped that no font engine could read.
pub const RULE_UNREADABLE_FONT: &str = "BK-FONT-001";

/// Every face the project may use, plus what went wrong while collecting them.
pub struct Fonts {
    book: LazyHash<FontBook>,
    faces: Vec<Font>,
    project_faces: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Fonts {
    /// Collects the project's fonts and Booker's own.
    ///
    /// Never fails: a font file that cannot be read becomes a warning, because
    /// a book with one broken font file must still open (`AGENTS.md` §7).
    pub fn load(project_root: &Path) -> Self {
        let mut faces = Vec::new();
        let mut diagnostics = Vec::new();

        // Project fonts first, so that a project shipping its own cut of a
        // family is preferred over ours when the family names collide.
        let dir = project_root.join(PROJECT_FONT_DIR);
        for path in font_files(&dir) {
            match std::fs::read(&path) {
                Ok(data) => {
                    let before = faces.len();
                    faces.extend(Font::iter(Bytes::new(data)));
                    if faces.len() == before {
                        diagnostics.push(unreadable(
                            project_root,
                            &path,
                            "it is not a font file Booker can read (TrueType, \
                             OpenType and their collections are supported)",
                        ));
                    }
                }
                Err(err) => {
                    diagnostics.push(unreadable(project_root, &path, &err.to_string()));
                }
            }
        }
        let project_faces = faces.len();

        for (name, data) in BUNDLED {
            let bundled = Font::iter(Bytes::new(*data));
            let before = faces.len();
            faces.extend(bundled);
            debug_assert!(
                faces.len() > before,
                "bundled font {name} did not parse; it is checked into the repository, \
                 so this is a build problem, not a user's project"
            );
        }

        Self {
            book: LazyHash::new(FontBook::from_fonts(&faces)),
            faces,
            project_faces,
            diagnostics,
        }
    }

    /// What Typst asks when it is matching a family name to a face.
    pub fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// What Typst asks once it has picked one.
    pub fn face(&self, index: usize) -> Option<Font> {
        self.faces.get(index).cloned()
    }

    /// Every family name a document in this project may name, sorted.
    pub fn families(&self) -> Vec<String> {
        let mut families: Vec<String> = self
            .book
            .families()
            .map(|(name, _)| name.to_string())
            .collect();
        families.sort_unstable();
        families.dedup();
        families
    }

    /// How many faces are loaded in total.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// How many of them came out of the project's own `assets/fonts`.
    pub fn project_face_count(&self) -> usize {
        self.project_faces
    }

    /// Font files the project ships that could not be read. These ride along
    /// with every compile, so the problems panel shows them.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

fn unreadable(root: &Path, path: &Path, why: &str) -> Diagnostic {
    // Project-relative and forward-slashed, so the warning reads the same on
    // every platform: a diagnostic goes to the problems panel, to the CLI
    // and, from Wave 7, to an agent comparing our output against the CLI's.
    let relative = PathBuf::from(display_path(path.strip_prefix(root).unwrap_or(path)));
    Diagnostic::error(
        RULE_UNREADABLE_FONT,
        format!("`{}` was ignored: {why}", relative.display()),
    )
    .with_severity(Severity::Warning)
    .at(SourceLocation {
        file: relative,
        line: 1,
        column: 1,
        span: None,
    })
}

/// Every plausible font file under `dir`, in a stable order so that two
/// machines load the same faces in the same sequence.
fn font_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(dir, 0, &mut found);
    found.sort();
    found
}

fn collect(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        // No font directory at all is the common case, not a problem.
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => collect(&path, depth + 1, found),
            Ok(_) if is_font_file(&path) => found.push(path),
            _ => {}
        }
    }
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_project_with_no_fonts_still_has_the_bundled_ones() {
        let fonts = Fonts::load(Path::new("/does/not/exist"));
        assert_eq!(fonts.project_face_count(), 0);
        assert!(fonts.diagnostics().is_empty());
        let families = fonts.families();
        assert!(
            families.iter().any(|f| f == "Libertinus Serif"),
            "Typst's default text family must be present or a book with no \
             configuration renders nothing: {families:?}"
        );
        assert!(
            families.iter().any(|f| f == "DejaVu Sans Mono"),
            "Typst's default raw family must be present: {families:?}"
        );
    }

    #[test]
    fn only_font_files_are_picked_up() {
        assert!(is_font_file(Path::new("a/b/Thing.OTF")));
        assert!(is_font_file(Path::new("a/b/Thing.ttc")));
        assert!(!is_font_file(Path::new("a/b/readme.md")));
        assert!(!is_font_file(Path::new("a/b/fonts")));
    }
}

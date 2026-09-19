//! The seam toward the layout engine.
//!
//! ## This is a seam, not an implementation
//!
//! Producing a PDF needs `booker-typst`, which is Wave 0 track C and is
//! being built in parallel with this crate. Until it exposes `compile`,
//! `booker build` does everything except the last step: it loads the
//! project, parses the Markdown, reports what it found, and says what it
//! *would* build.
//!
//! **To close the seam**, when `booker_typst::compile(&CompileRequest) ->
//! Result<CompileResult>` exists, this is the only file to change: replace
//! the body of [`compile`] with the call, and delete
//! [`Outcome::EngineNotHereYet`]. Nothing else in the CLI knows how a PDF is
//! made — the request and the result are the Wave 0 contracts
//! (`booker_core::compile`), which both sides already agree on.

use std::path::{Path, PathBuf};

use booker_core::{CompileRequest, CompileResult};

/// What a build attempt did.
pub enum Outcome {
    /// The engine ran.
    Compiled(Box<CompileResult>),
    /// The engine is not in this build yet. Carries what it would have been
    /// asked to do, so the CLI can say so precisely.
    EngineNotHereYet { output: PathBuf },
}

/// Ask the layout engine to lay the book out and write `output`.
pub fn compile(request: &CompileRequest, output: &Path) -> Outcome {
    // SEAM (Wave 0 track C): becomes
    //
    //     Outcome::Compiled(Box::new(booker_typst::compile(request, output)?))
    //
    // `booker-typst` today re-exports the contracts and nothing else, so
    // calling it would not compile. Everything up to this line is real.
    let _ = request;
    Outcome::EngineNotHereYet {
        output: output.to_path_buf(),
    }
}

/// Where a build writes, given a project root and a title.
///
/// `build/` is generated and git-ignored (`PLAN.md` §5.1).
pub fn output_path(root: &Path, title: &str) -> PathBuf {
    root.join("build").join(format!("{}.pdf", slug(title)))
}

/// A file name made from a title: lowercase, words joined by `-`, nothing
/// that needs quoting on any platform.
fn slug(title: &str) -> String {
    let mut slug = String::new();
    let mut wants_dash = false;
    for character in title.chars() {
        if character.is_alphanumeric() {
            for lower in character.to_lowercase() {
                if wants_dash && !slug.is_empty() {
                    slug.push('-');
                }
                wants_dash = false;
                slug.push(lower);
            }
        } else {
            wants_dash = true;
        }
    }
    if slug.is_empty() {
        "book".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_become_file_names_that_need_no_quoting() {
        assert_eq!(slug("The Secret Garden of Mia"), "the-secret-garden-of-mia");
        assert_eq!(slug("Grand-mère écrit"), "grand-mère-écrit");
        assert_eq!(slug("!!!"), "book");
        assert_eq!(slug(""), "book");
    }

    #[test]
    fn a_build_writes_into_the_generated_folder() {
        let path = output_path(Path::new("/books/mia"), "Mia");
        assert_eq!(path, Path::new("/books/mia/build/mia.pdf"));
    }
}

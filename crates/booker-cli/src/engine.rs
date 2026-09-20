//! The seam toward the layout engine.
//!
//! `booker build` asks [`compile`] for a PDF. Everything about *how* a book
//! becomes pages lives behind this one function: the CLI knows the request
//! and the result (the `booker_core` contracts) and nothing else.
//!
//! The translation from the document model to Typst lives in
//! `booker_typst::book`, behind [`Engine::set_book`]; the application calls
//! the same one, so a book cannot lay out differently in the window and in
//! the terminal (`AGENTS.md` §6). What is left here is the part that is the
//! CLI's alone: deciding where the file goes and writing it.

use std::path::{Path, PathBuf};

use booker_core::{BookConfig, CompileRequest, CompileResult};
use booker_doc::Document;
use booker_typst::Engine;

/// What a build attempt did.
pub enum Outcome {
    /// The engine ran and the PDF is on disk.
    Compiled(Box<CompileResult>),
    /// The engine could not answer. Carries the reason, already phrased for
    /// a person: a failed build must say what to do about it.
    Failed(String),
}

/// Lay the book out and write `output`.
///
/// One engine, used once: a build is a single compile and then the process
/// exits. The application keeps its engine alive instead, which is what
/// makes an edit re-render in milliseconds (`docs/FINDINGS.md`).
pub fn compile(
    request: &CompileRequest,
    config: &BookConfig,
    chapters: &[(&str, &Document)],
    output: &Path,
) -> Outcome {
    let mut engine = match Engine::open(request.project.clone()) {
        Ok(engine) => engine,
        Err(error) => return Outcome::Failed(error.to_string()),
    };
    if let Err(error) = engine.set_book(config, chapters) {
        return Outcome::Failed(error.to_string());
    }

    let compilation = match engine.compile(request) {
        Ok(compilation) => compilation,
        Err(error) => return Outcome::Failed(error.to_string()),
    };

    if let Some(pdf) = compilation.pdf {
        if let Some(parent) = output.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                return Outcome::Failed(format!("{}: {error}", parent.display()));
            }
        }
        if let Err(error) = std::fs::write(output, pdf) {
            return Outcome::Failed(format!("{}: {error}", output.display()));
        }
    }

    Outcome::Compiled(Box::new(compilation.result))
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

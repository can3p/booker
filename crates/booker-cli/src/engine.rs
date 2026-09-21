//! The seam toward the layout engine.
//!
//! Every command that needs pages asks [`lay_out`] for them. Everything about *how* a book
//! becomes pages lives behind this one function: the CLI knows the request
//! and the result (the `booker_core` contracts) and nothing else.
//!
//! The translation from the document model to Typst lives in
//! `booker_typst::book`, behind [`Engine::set_book`]; the application calls
//! the same one, so a book cannot lay out differently in the window and in
//! the terminal (`AGENTS.md` §6). What is left here is the part that is the
//! CLI's alone: deciding where the file goes and writing it.

use std::path::{Path, PathBuf};

use booker_core::{CompileRequest, CompileTarget};
use booker_project::Project;
use booker_typst::{Compilation, Engine};

/// Lay a loaded project out, once.
///
/// One engine, used for one command: the process exits afterwards. The
/// application keeps its engine alive instead, which is what makes an edit
/// re-render in milliseconds (`docs/FINDINGS.md`). The engine comes back
/// with the compilation so that `where` and `page` can ask it about the
/// pages it just made.
///
/// `Err` is a reason already phrased for a person: the engine could not
/// answer at all. Anything wrong with the *book* is in the compilation's
/// diagnostics instead.
pub fn lay_out(project: &Project, target: CompileTarget) -> Result<(Engine, Compilation), String> {
    let mut engine = Engine::open(project.reference().clone()).map_err(|e| e.to_string())?;
    let chapters: Vec<(&str, &booker_doc::Document)> = project
        .chapters()
        .iter()
        .map(|chapter| (chapter.source(), chapter.document()))
        .collect();
    engine
        .set_book(project.config(), &chapters)
        .map_err(|e| e.to_string())?;
    let compilation = engine
        .compile(&CompileRequest {
            project: project.reference().clone(),
            target,
            revision: project.revision(),
        })
        .map_err(|e| e.to_string())?;
    Ok((engine, compilation))
}

/// Write the PDF a compilation produced.
pub fn write_pdf(pdf: &[u8], output: &Path) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(output, pdf).map_err(|e| format!("{}: {e}", output.display()))
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

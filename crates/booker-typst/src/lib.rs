//! The layout engine: Typst, embedded in process.
//!
//! Owned by Wave 0 track C. Typst types stay inside this crate (`AGENTS.md`
//! §6) so that a Typst upgrade is a change to one crate, gated by the golden
//! tests. Nothing here takes Markdown: this crate compiles Typst source.
//! Turning Booker's document model into that source is Wave 1.
//!
//! # The shape of it
//!
//! ```no_run
//! use booker_core::{CompileRequest, CompileTarget, ProjectRef, Revision};
//! use booker_typst::Engine;
//!
//! # fn main() -> booker_core::Result<()> {
//! let project = ProjectRef::new("/books/mia");
//! let mut engine = Engine::open(project.clone())?;
//!
//! // What the editor is holding, which may not be on disk yet.
//! engine.set_source("main.typ", "= Chapter One\n\nIt was a bright day.")?;
//!
//! let compiled = engine.compile(&CompileRequest {
//!     project,
//!     target: CompileTarget::Pdf,
//!     revision: Revision(1),
//! })?;
//!
//! println!("{} pages", compiled.result.pages.len());
//! for problem in &compiled.result.diagnostics {
//!     println!("{}: {}", problem.rule, problem.message);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Two things to know
//!
//! * **Keep the [`Engine`] alive.** Its memoized state is what makes the
//!   second compile of a 200-page book take milliseconds instead of seconds.
//! * **A broken book is not an error.** [`Engine::compile`] returns `Err`
//!   only when the request itself makes no sense. Anything wrong with the
//!   user's content arrives as [`booker_core::Diagnostic`] values with a file
//!   and a line, and never as a panic.

mod engine;
mod world;

pub mod diagnostics;
pub mod fonts;

pub use booker_core::{CompileRequest, CompileResult, Error, RenderRequest, Result};
pub use engine::{Compilation, Engine};
pub use fonts::{Fonts, PROJECT_FONT_DIR};
pub use world::{BookerWorld, DEFAULT_ENTRYPOINT};

/// The Typst release this build is pinned to.
///
/// Typst is pre-1.0 and its layout changes between releases, so the version
/// is exact in `Cargo.toml` and named here for the about box, `booker
/// project_info` and bug reports. A book laid out by two different Typst
/// versions is two slightly different books.
pub const TYPST_VERSION: &str = "0.15.1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pinned_typst_version_is_the_one_we_compiled_against() {
        // If someone relaxes the `=` pin in Cargo.toml, this is what notices.
        let compiler = typst::syntax::package::PackageVersion::compiler();
        assert_eq!(
            TYPST_VERSION,
            format!("{}.{}.{}", compiler.major, compiler.minor, compiler.patch)
        );
    }
}

//! The layout engine: document model plus styles to Typst, compiled and
//! rendered in process.
//!
//! Owned by Wave 0 track C. Typst types stay inside this crate (`AGENTS.md`
//! §6) so that a Typst upgrade is a change to one crate, gated by the
//! golden tests.

pub use booker_core::{CompileRequest, CompileResult, Error, RenderRequest, Result};

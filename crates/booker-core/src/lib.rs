//! The contracts every other crate and the app agree on.
//!
//! Nothing here reaches for the filesystem, Typst, or the UI. These are the
//! shapes that cross crate boundaries, so that tracks can be built in
//! parallel against them. They are frozen for the duration of a wave; see
//! `AGENTS.md` §3.

pub mod compile;
pub mod diagnostic;
pub mod error;
pub mod geometry;
pub mod ipc;
pub mod path;
pub mod project;

pub use compile::{
    CompileRequest, CompileResult, CompileTarget, PageInfo, RenderFormat, RenderRequest,
};
pub use diagnostic::{Diagnostic, Fix, LayoutLocation, Severity, SourceLocation};
pub use error::{Error, Result};
pub use geometry::{Length, Margins, PageSize, Unit};
pub use path::display_path;
pub use project::{BookConfig, PageConfig, ProjectRef, Revision, FORMAT_VERSION};

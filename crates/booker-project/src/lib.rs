//! Loading, watching and writing a project folder.
//!
//! Owned by Wave 0 track E. The rules this crate exists to enforce
//! (`AGENTS.md` §7): writes are atomic and targeted, comments and unknown
//! keys survive a round trip, opening a project changes nothing on disk, and
//! a broken project still loads as far as it can.

pub use booker_core::{BookConfig, Error, ProjectRef, Result, Revision};

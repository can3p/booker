//! The Booker application: a window around the same core the CLI uses.
//!
//! Owned by Wave 1 track B. What lives here is only what a window needs and
//! a terminal does not — holding an open project, answering the UI, serving
//! page images. Everything about what a book *is* comes from the core
//! crates, through the contracts in [`booker_core::ipc`], and anything the
//! window can say about a project the CLI can print (`AGENTS.md` §6).
//!
//! The commands are in [`commands`] and the state they act on is
//! [`Session`]. `main.rs` is a thin shell over [`run`] so that the commands
//! can be tested without starting a window.

use std::sync::Mutex;

pub mod commands;
pub mod session;

pub use session::{OpenProject, Session};

/// Build the application and run it.
///
/// Separate from `main` so the desktop entry point stays three lines, and
/// so a test can build the same command set without a window.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Mutex::new(Session::default()))
        .invoke_handler(tauri::generate_handler![
            commands::close_project,
            commands::open_project,
            commands::project_info,
        ])
        .run(tauri::generate_context!())
        .expect("the application failed to start");
}

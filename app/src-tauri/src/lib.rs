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
pub mod menu;
pub mod protocol;
pub mod recent;
pub mod session;
pub mod watch;

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
        .menu(menu::build)
        .on_menu_event(menu::on_event)
        // Page images are fetched by the webview, not returned through
        // IPC: see `protocol` for why the revision is in the URL.
        .register_uri_scheme_protocol("booker", |context, request| {
            protocol::handle(context.app_handle(), request)
        })
        .invoke_handler(tauri::generate_handler![
            commands::close_project,
            commands::compile,
            commands::export_pdf,
            commands::open_project,
            commands::project_info,
            commands::read_chapter,
            commands::recent_projects,
            commands::reload_project,
            commands::render_page,
            commands::save_chapter,
        ])
        .run(tauri::generate_context!())
        .expect("the application failed to start");
}

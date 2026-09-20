//! The commands the UI invokes, one function per name in
//! [`booker_core::ipc::COMMANDS`].
//!
//! Every one of them is thin on purpose. A command locks the session, asks
//! the core, and hands back a contract type; the moment one starts deciding
//! what a book means, that decision belongs in a core crate where the CLI
//! can reach it too (`AGENTS.md` §6).
//!
//! **Errors are strings here, and that is not a shortcut.** What crosses to
//! the UI is a sentence for a person, already naming the file it came from
//! (`AGENTS.md` §6). Anything wrong with the user's *book* is not an error
//! at all: it is a diagnostic inside a perfectly successful answer, because
//! a broken project must still open (`AGENTS.md` §7).

use std::path::PathBuf;
use std::sync::Mutex;

use booker_core::{ChapterText, CompileResult, CompileTarget, ExportRequest, ProjectInfo};
use tauri::{AppHandle, Manager, State};

use crate::recent;
use crate::session::Session;

/// What a command hands back when it cannot answer: a sentence, not a type
/// the UI has to interpret.
pub type CommandResult<T> = std::result::Result<T, String>;

/// Open a project folder. Returns what is in it, faults included.
#[tauri::command]
pub fn open_project(
    app: AppHandle,
    session: State<'_, Mutex<Session>>,
    root: PathBuf,
) -> CommandResult<ProjectInfo> {
    let info = {
        let mut session = session.lock().map_err(poisoned)?;
        session.open(&root).map_err(|error| error.to_string())?
    };
    // Remember it under the path the project itself reports, which is
    // canonical — two different spellings of one folder are one entry.
    if let Ok(dir) = app.path().app_config_dir() {
        recent::remember(&dir, &info.project.root);
    }
    Ok(info)
}

/// Describe the project that is already open.
///
/// Asking with nothing open is not an error — the window asks on startup,
/// before anyone has chosen a folder.
#[tauri::command]
pub fn project_info(session: State<'_, Mutex<Session>>) -> CommandResult<Option<ProjectInfo>> {
    let session = session.lock().map_err(poisoned)?;
    Ok(session.current().map(|open| open.info()))
}

/// Close the open project and let go of its engine.
#[tauri::command]
pub fn close_project(session: State<'_, Mutex<Session>>) -> CommandResult<()> {
    let mut session = session.lock().map_err(poisoned)?;
    session.close();
    Ok(())
}

/// The folders this window has opened before, newest first.
#[tauri::command]
pub fn recent_projects(app: AppHandle) -> CommandResult<Vec<PathBuf>> {
    let Ok(dir) = app.path().app_config_dir() else {
        // No configuration folder means nothing was ever remembered, which
        // is an empty list rather than a failure.
        return Ok(Vec::new());
    };
    Ok(recent::read(&dir))
}

/// Lay the book out, without writing anything.
///
/// A book with errors comes back with no pages and the errors in
/// `diagnostics`; that is a successful answer, not a failure.
#[tauri::command]
pub fn compile(session: State<'_, Mutex<Session>>) -> CommandResult<CompileResult> {
    let mut session = session.lock().map_err(poisoned)?;
    let open = session.current_mut().ok_or_else(nothing_open)?;
    open.compile(CompileTarget::Layout)
        .map(|compilation| compilation.result)
        .map_err(|error| error.to_string())
}

/// Lay the book out and write a PDF where the user chose.
#[tauri::command]
pub fn export_pdf(
    session: State<'_, Mutex<Session>>,
    request: ExportRequest,
) -> CommandResult<CompileResult> {
    let mut session = session.lock().map_err(poisoned)?;
    let open = session.current_mut().ok_or_else(nothing_open)?;
    open.export_pdf(&request.destination)
        .map_err(|error| error.to_string())
}

/// One chapter's source, for the editor pane.
#[tauri::command]
pub fn read_chapter(
    session: State<'_, Mutex<Session>>,
    path: String,
) -> CommandResult<ChapterText> {
    let session = session.lock().map_err(poisoned)?;
    let open = session.current().ok_or_else(nothing_open)?;
    open.read_chapter(&path).map_err(|error| error.to_string())
}

/// Write a chapter back and describe the project as it now stands.
#[tauri::command]
pub fn save_chapter(
    session: State<'_, Mutex<Session>>,
    path: String,
    text: String,
) -> CommandResult<ProjectInfo> {
    let mut session = session.lock().map_err(poisoned)?;
    let open = session.current_mut().ok_or_else(nothing_open)?;
    open.save_chapter(&path, &text)
        .map_err(|error| error.to_string())
}

/// Asked something about a project when none is open. The window should not
/// let this happen, so the message is for whoever is debugging it.
fn nothing_open() -> String {
    "no book is open".to_string()
}

/// A poisoned lock means another command panicked while holding it. Say so
/// plainly rather than panicking a second time on top of the first.
fn poisoned<T>(_: std::sync::PoisonError<T>) -> String {
    "the open project is in an unknown state after an earlier failure; close it and open it again"
        .to_string()
}

#[cfg(test)]
mod tests {
    use booker_core::ipc::COMMANDS;

    /// Every command this crate implements is named in the contract, and
    /// every name in the contract is one the UI may call. The list is
    /// append-only within a wave, so this test is what notices a track
    /// implementing something nobody agreed to (`AGENTS.md` §3).
    #[test]
    fn the_commands_implemented_here_are_in_the_contract() {
        for implemented in [
            "close_project",
            "compile",
            "export_pdf",
            "open_project",
            "project_info",
            "read_chapter",
            "recent_projects",
            "save_chapter",
        ] {
            assert!(
                COMMANDS.contains(&implemented),
                "`{implemented}` is not in booker_core::ipc::COMMANDS"
            );
        }
    }

    /// The Wave 1 commands still to be written, so that the gap between the
    /// contract and this crate is visible rather than forgotten. Delete a
    /// name from here when its command lands.
    ///
    /// Both are track C's: page images are served over the `booker://`
    /// protocol rather than through IPC, so they arrive with the preview.
    #[test]
    fn what_is_left_to_implement_is_written_down() {
        for name in ["page_image_url", "render_page"] {
            assert!(
                COMMANDS.contains(&name),
                "`{name}` was removed from the contract; remove it from this list too"
            );
        }
    }
}

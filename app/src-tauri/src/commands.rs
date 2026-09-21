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

use booker_core::{
    ChapterText, CompileResult, CompileTarget, ExportRequest, ProjectInfo, RenderRequest,
};
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::recent;
use crate::session::Session;
use crate::watch;

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
    let (info, own_writes) = {
        let mut session = session.lock().map_err(poisoned)?;
        let info = session.open(&root).map_err(|error| error.to_string())?;
        let own = session
            .current()
            .expect("a project was just opened")
            .own_writes();
        (info, own)
    };

    // Watch the folder, so an edit made in another editor — or by an agent
    // rewriting the whole book — reaches the window (`AGENTS.md` §7). A
    // folder that cannot be watched is not a reason to refuse to open it:
    // the book is there and readable, it simply will not follow along.
    let emitter = app.clone();
    match watch::start(info.project.clone(), own_writes, move |changed| {
        let _ = emitter.emit("project-changed", changed);
    }) {
        Ok(started) => {
            let mut session = session.lock().map_err(poisoned)?;
            if let Some(open) = session.current_mut() {
                open.set_watch(started);
            }
        }
        Err(error) => eprintln!(
            "booker: not watching {}: {error}",
            info.project.root.display()
        ),
    }

    // Remember it under the path the project itself reports, which is
    // canonical — two different spellings of one folder are one entry.
    if let Ok(dir) = app.path().app_config_dir() {
        recent::remember(&dir, &info.project.root);
    }
    Ok(info)
}

/// Read the project from disk again, after something outside changed it.
///
/// The window calls this when a `project-changed` event arrives, rather
/// than being handed the new state with the event: the event says *that*
/// something changed, and this says what the book now is. One answer to
/// one question.
#[tauri::command]
pub fn reload_project(session: State<'_, Mutex<Session>>) -> CommandResult<Option<ProjectInfo>> {
    let mut session = session.lock().map_err(poisoned)?;
    let Some(open) = session.current_mut() else {
        // The project was closed between the event and this call.
        return Ok(None);
    };
    open.reload().map(Some).map_err(|error| error.to_string())
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

/// One page, rendered.
///
/// The preview does not use this — it fetches `booker://` URLs, because
/// base64 through a JSON channel is the slowest thing in the application.
/// This is for the times a caller wants the bytes themselves, and it is
/// what `booker render --page` will be built on (Wave 7). The reply is a
/// raw body rather than a JSON array, so the bytes are not re-encoded.
#[tauri::command]
pub fn render_page(
    session: State<'_, Mutex<Session>>,
    request: RenderRequest,
) -> CommandResult<Response> {
    let mut session = session.lock().map_err(poisoned)?;
    let open = session.current_mut().ok_or_else(nothing_open)?;
    open.engine_mut()
        .render(&request)
        .map(Response::new)
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
            "reload_project",
            "render_page",
            "save_chapter",
        ] {
            assert!(
                COMMANDS.contains(&implemented),
                "`{implemented}` is not in booker_core::ipc::COMMANDS"
            );
        }
    }

    /// The commands Wave 2's contracts step added and its tracks have not
    /// implemented yet. A track that implements one moves it from here to
    /// the list above; the wave is not finished while this list has
    /// anything in it (`docs/waves/wave-2.md`).
    const NOT_YET_IMPLEMENTED: &[&str] = &[
        "add_chapter",    // track C
        "move_chapter",   // track C
        "pages_at",       // track D
        "remove_chapter", // track C
        "rename_chapter", // track C
        "source_at",      // track D
    ];

    #[test]
    fn every_name_in_the_contract_is_implemented_or_listed_as_not_yet() {
        let implemented = [
            "close_project",
            "compile",
            "export_pdf",
            "open_project",
            "project_info",
            "read_chapter",
            "recent_projects",
            "reload_project",
            "render_page",
            "save_chapter",
        ];
        for name in COMMANDS {
            assert!(
                implemented.contains(name)
                    || NOT_YET_IMPLEMENTED.contains(name)
                    || *name == "page_image_url",
                "`{name}` is in the contract but neither implemented nor listed as pending"
            );
        }
    }

    /// `page_image_url` is in the contract as the URL *shape* the preview
    /// uses, built by `booker_core::ipc::page_image_url` and answered by
    /// `crate::protocol` — there is no `#[tauri::command]` for it and there
    /// should not be, because the whole point is that page images do not
    /// travel through IPC.
    #[test]
    fn the_one_name_without_a_command_is_the_protocol_url() {
        assert!(COMMANDS.contains(&"page_image_url"));
    }
}

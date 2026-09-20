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

use booker_core::ProjectInfo;
use tauri::State;

use crate::session::Session;

/// What a command hands back when it cannot answer: a sentence, not a type
/// the UI has to interpret.
pub type CommandResult<T> = std::result::Result<T, String>;

/// Open a project folder. Returns what is in it, faults included.
#[tauri::command]
pub fn open_project(
    session: State<'_, Mutex<Session>>,
    root: PathBuf,
) -> CommandResult<ProjectInfo> {
    let mut session = session.lock().map_err(poisoned)?;
    session.open(root).map_err(|error| error.to_string())
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
        for implemented in ["close_project", "open_project", "project_info"] {
            assert!(
                COMMANDS.contains(&implemented),
                "`{implemented}` is not in booker_core::ipc::COMMANDS"
            );
        }
    }

    /// The Wave 1 commands still to be written, so that the gap between the
    /// contract and this crate is visible rather than forgotten. Delete a
    /// name from here when its command lands.
    #[test]
    fn what_is_left_to_implement_is_written_down() {
        let outstanding = [
            "compile",
            "export_pdf",
            "page_image_url",
            "read_chapter",
            "recent_projects",
            "render_page",
            "save_chapter",
        ];
        for name in outstanding {
            assert!(
                COMMANDS.contains(&name),
                "`{name}` was removed from the contract; remove it from this list too"
            );
        }
    }
}

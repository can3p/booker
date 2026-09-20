//! The application menu.
//!
//! The menu does no work itself. Each item emits an event the window
//! listens for, so "Open a book…" from the menu and the button in the
//! window run exactly the same code — there is no second path through the
//! application that only the menu can reach.
//!
//! "Check for updates…" is the one item `AGENTS.md` §4 requires by name:
//! every wave ships through the updater, and a person must be able to ask
//! for an update rather than wait to be offered one.

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// The event a menu item sends to the window. The payload is the item's id,
/// so one listener in the UI covers every item.
pub const MENU_EVENT: &str = "menu";

/// Item ids. These are the vocabulary the UI matches on, so they are
/// spelled once here and imported there through the event payload.
pub const OPEN: &str = "open";
pub const CLOSE: &str = "close";
pub const EXPORT_PDF: &str = "export-pdf";
pub const CHECK_FOR_UPDATES: &str = "check-for-updates";

/// Build the menu for this platform.
///
/// On macOS the first submenu is the application menu and carries About,
/// Services, Hide and Quit; elsewhere those live under File and Help. The
/// predefined items give each platform its own conventions and translations
/// for free, which is why they are used rather than hand-made ones.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let open = MenuItem::with_id(app, OPEN, "Open a Book…", true, Some("CmdOrCtrl+O"))?;
    let close = MenuItem::with_id(app, CLOSE, "Close Book", true, Some("CmdOrCtrl+W"))?;
    let export = MenuItem::with_id(app, EXPORT_PDF, "Export PDF…", true, Some("CmdOrCtrl+E"))?;
    let updates = MenuItem::with_id(
        app,
        CHECK_FOR_UPDATES,
        "Check for Updates…",
        true,
        None::<&str>,
    )?;

    let menu = Menu::new(app)?;

    #[cfg(target_os = "macos")]
    {
        let about = AboutMetadata {
            name: Some("Booker".into()),
            ..Default::default()
        };
        menu.append(&Submenu::with_items(
            app,
            "Booker",
            true,
            &[
                &PredefinedMenuItem::about(app, None, Some(about))?,
                &updates,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::show_all(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, None)?,
            ],
        )?)?;
    }

    menu.append(&Submenu::with_items(
        app,
        "File",
        true,
        &[&open, &close, &PredefinedMenuItem::separator(app)?, &export],
    )?)?;

    // Cut, copy, paste and undo are what a text pane needs, and the
    // predefined items wire them to the webview's own handling.
    menu.append(&Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?)?;

    #[cfg(not(target_os = "macos"))]
    {
        let about = AboutMetadata {
            name: Some("Booker".into()),
            ..Default::default()
        };
        menu.append(&Submenu::with_items(
            app,
            "Help",
            true,
            &[
                &updates,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::about(app, None, Some(about))?,
            ],
        )?)?;
    }

    Ok(menu)
}

/// Pass the item's id to the window, which does the work.
pub fn on_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    // A window that has gone away is not an error worth reporting: the user
    // is closing the application.
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit(MENU_EVENT, event.id().0.clone());
    }
}

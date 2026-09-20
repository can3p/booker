//! The `booker://` protocol, against a real book.
//!
//! `protocol.rs`'s unit tests cover reading the URL. This covers the rest
//! of the path: a request arriving, a page being rendered, and the three
//! answers that are not an image — no book open, a revision that has moved
//! on, and a URL that is not a page.
//!
//! `tauri::test` builds an application without a window, so this runs as an
//! ordinary test — **except on Windows**, where the binary will not start.
//!
//! What happens there is that the test executable exits with
//! `STATUS_ENTRYPOINT_NOT_FOUND` before `main` runs: it lives in
//! `target/debug/deps/`, and the WebView2 loader Tauri links against is not
//! resolvable from there. It is a loader problem, not a Booker one — the
//! same crate's unit tests, which link the same libraries, run on Windows
//! without complaint.
//!
//! What Windows loses by skipping this is the end-to-end path, which is
//! platform-independent: rendering is covered per-platform by
//! `booker-typst`'s own tests. What it keeps is the part that genuinely
//! differs there — Windows rewrites `booker://page/…` to
//! `http://booker.localhost/page/…`, and `protocol.rs`'s unit tests assert
//! both spellings parse, on every platform.
#![cfg(not(windows))]

use std::sync::Mutex;

use booker_app::protocol;
use booker_app::session::Session;
use booker_core::ipc::page_image_url;
use booker_project::{Project, Template};
use tauri::http::{Request, StatusCode};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::{AppHandle, Manager};

/// An application with a session in it, and a book in a temporary folder.
fn app_with_a_book() -> (AppHandle<tauri::test::MockRuntime>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("mia");
    Project::create(&root, Template::Novel, Some("The Secret Garden of Mia")).expect("a new book");

    let app = mock_builder()
        .manage(Mutex::new(Session::default()))
        .build(mock_context(noop_assets()))
        .expect("the application builds");

    let handle = app.handle().clone();
    {
        let state = handle.state::<Mutex<Session>>();
        let mut session = state.lock().unwrap();
        session.open(&root).expect("the book opens");
    }
    (handle, dir)
}

fn request(uri: &str) -> Request<Vec<u8>> {
    Request::builder()
        .uri(uri)
        .body(Vec::new())
        .expect("a well-formed request")
}

/// The revision the open project is currently at.
fn current_revision(app: &AppHandle<tauri::test::MockRuntime>) -> u64 {
    let state = app.state::<Mutex<Session>>();
    let session = state.lock().unwrap();
    session
        .current()
        .expect("a book is open")
        .project()
        .revision()
        .0
}

#[test]
fn a_page_comes_back_as_a_png() {
    let (app, _dir) = app_with_a_book();
    let url = page_image_url(current_revision(&app), 0, 2.0, "png");

    let response = protocol::handle(&app, request(&url));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("Content-Type").unwrap(), "image/png");
    assert!(
        response.body().starts_with(&[0x89, b'P', b'N', b'G']),
        "the body should be a PNG"
    );
    // An image is immutable for its revision, so the webview may keep it.
    let caching = response.headers().get("Cache-Control").unwrap();
    assert!(caching.to_str().unwrap().contains("immutable"));
}

#[test]
fn the_same_page_as_svg() {
    let (app, _dir) = app_with_a_book();
    let url = page_image_url(current_revision(&app), 0, 1.0, "svg");

    let response = protocol::handle(&app, request(&url));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/svg+xml"
    );
    assert!(String::from_utf8_lossy(response.body()).contains("<svg"));
}

#[test]
fn a_page_from_a_revision_that_has_moved_on_is_refused_rather_than_substituted() {
    let (app, _dir) = app_with_a_book();
    let stale = page_image_url(current_revision(&app) + 7, 0, 2.0, "png");

    let response = protocol::handle(&app, request(&stale));

    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "quietly serving a different page than the one asked for is how a preview starts lying"
    );
}

#[test]
fn a_page_that_does_not_exist_says_so() {
    let (app, _dir) = app_with_a_book();
    let url = page_image_url(current_revision(&app), 999, 2.0, "png");

    let response = protocol::handle(&app, request(&url));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // And the message is a sentence, not a panic.
    assert!(!response.body().is_empty());
}

#[test]
fn a_url_that_is_not_a_page_is_a_bad_request() {
    let (app, _dir) = app_with_a_book();

    let response = protocol::handle(&app, request("booker://something/else"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn asking_with_no_book_open_is_not_a_crash() {
    let app = mock_builder()
        .manage(Mutex::new(Session::default()))
        .build(mock_context(noop_assets()))
        .expect("the application builds");
    let handle = app.handle().clone();

    let response = protocol::handle(&handle, request(&page_image_url(0, 0, 1.0, "png")));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

//! Page images, served over the `booker://` protocol.
//!
//! A preview scrolls fast, and base64 through a JSON channel is the slowest
//! thing in the application (`PLAN.md` §4), so pages do not travel as IPC
//! results. They are fetched by the webview like any other image, from a
//! protocol this process answers.
//!
//! The URL shape is `booker_core::ipc::page_image_url`:
//!
//! ```text
//! booker://page/<revision>/<page>@<scale>x.<png|svg>
//! ```
//!
//! **The revision is in the path on purpose.** An image is immutable for a
//! given revision, so the webview may cache it forever and a stale one can
//! never be mistaken for a current one: after an edit the URLs are simply
//! different. That is also why a request for a revision the engine no
//! longer holds is answered with 409 rather than with the current page —
//! quietly serving a different page than the one asked for is how a preview
//! starts lying.

use std::sync::Mutex;

use booker_core::{RenderFormat, RenderRequest, Revision};
use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Manager, Runtime};

use crate::session::Session;

/// What a `booker://` URL asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageUrl {
    pub revision: Revision,
    /// 0-based, matching `PageInfo::index`.
    pub page: u32,
    /// Pixels per CSS pixel: 1.0 is 96 dpi, 2.0 a retina display.
    pub scale: f32,
    pub format: RenderFormat,
}

/// Read a `booker://` URL.
///
/// Both spellings are accepted, because the webview does not use the same
/// one on every platform: macOS and Linux see `booker://page/…`, where
/// `page` is parsed as the host, while Windows rewrites the whole thing to
/// `http://booker.localhost/page/…`. Looking for the `page/` segment rather
/// than for a fixed prefix covers both without caring which is which.
pub fn parse(uri: &str) -> Option<PageUrl> {
    // Drop a query string or fragment a webview may have added.
    let uri = uri.split(['?', '#']).next()?;
    let rest = uri.split("page/").nth(1)?;

    let (revision, rest) = rest.split_once('/')?;
    let revision = Revision(revision.parse().ok()?);

    // `@` survives as `%40` when something along the way encodes it.
    let (page, rest) = match rest.split_once('@') {
        Some(split) => split,
        None => rest.split_once("%40")?,
    };
    let page = page.parse().ok()?;

    let (scale, extension) = rest.rsplit_once('.')?;
    let scale: f32 = scale.strip_suffix(['x', 'X'])?.parse().ok()?;
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }

    let format = match extension.to_ascii_lowercase().as_str() {
        "png" => RenderFormat::Png,
        "svg" => RenderFormat::Svg,
        _ => return None,
    };

    Some(PageUrl {
        revision,
        page,
        scale,
        format,
    })
}

/// Answer one request for a page image.
pub fn handle<R: Runtime>(app: &AppHandle<R>, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let Some(url) = parse(&request.uri().to_string()) else {
        return plain(StatusCode::BAD_REQUEST, "not a page URL");
    };

    let state = app.state::<Mutex<Session>>();
    let Ok(mut session) = state.lock() else {
        return plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the open project is in an unknown state",
        );
    };
    let Some(open) = session.current_mut() else {
        return plain(StatusCode::NOT_FOUND, "no book is open");
    };

    if open.project().revision() != url.revision {
        // The book moved on — or back. The caller is looking at a preview
        // that no longer exists; it should ask again with current URLs.
        return plain(StatusCode::CONFLICT, "that revision is no longer current");
    }

    let project = open.project().reference().clone();
    let render = RenderRequest {
        project,
        revision: url.revision,
        page: url.page,
        scale: url.scale,
        format: url.format,
    };

    match open.engine_mut().render(&render) {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type(url.format))
            // Immutable for this revision, so never ask again.
            .header("Cache-Control", "public, max-age=31536000, immutable")
            .body(bytes)
            .unwrap_or_else(|_| {
                plain(StatusCode::INTERNAL_SERVER_ERROR, "could not build a reply")
            }),
        // A page that will not render is a message, not a panic: the book
        // may be mid-edit and broken, which is normal.
        Err(error) => plain(StatusCode::NOT_FOUND, &error.to_string()),
    }
}

fn content_type(format: RenderFormat) -> &'static str {
    match format {
        RenderFormat::Png => "image/png",
        RenderFormat::Svg => "image/svg+xml",
    }
}

fn plain(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(message.as_bytes().to_vec())
        .expect("a plain-text response is always well formed")
}

#[cfg(test)]
mod tests {
    use super::*;

    use booker_core::ipc::page_image_url;

    #[test]
    fn reads_the_url_the_contract_builds() {
        let url = parse(&page_image_url(7, 3, 2.0, "png")).expect("the contract's own URL");
        assert_eq!(
            url,
            PageUrl {
                revision: Revision(7),
                page: 3,
                scale: 2.0,
                format: RenderFormat::Png,
            }
        );
    }

    #[test]
    fn reads_the_windows_spelling_too() {
        // Windows rewrites the custom scheme to an http host, so the `page`
        // segment moves from the authority into the path.
        let url = parse("http://booker.localhost/page/7/3@2x.png").expect("the Windows form");
        assert_eq!(url.revision, Revision(7));
        assert_eq!(url.page, 3);
    }

    #[test]
    fn reads_an_encoded_at_sign() {
        let url = parse("booker://page/1/0%402x.png").expect("an encoded @");
        assert_eq!(url.page, 0);
        assert_eq!(url.scale, 2.0);
    }

    #[test]
    fn reads_a_fractional_scale_and_svg() {
        let url = parse(&page_image_url(2, 11, 1.5, "svg")).expect("a fractional scale");
        assert_eq!(url.scale, 1.5);
        assert_eq!(url.format, RenderFormat::Svg);
        assert_eq!(url.page, 11);
    }

    #[test]
    fn ignores_a_query_string() {
        assert!(parse("booker://page/1/0@1x.png?cachebust=3").is_some());
    }

    #[test]
    fn refuses_anything_that_is_not_a_page_url() {
        for bad in [
            "booker://page/1/0@1x.gif",      // not an image we render
            "booker://page/1/0.png",         // no scale
            "booker://page/x/0@1x.png",      // revision is not a number
            "booker://page/1/0@0x.png",      // a zero scale renders nothing
            "booker://page/1/0@-2x.png",     // nor a negative one
            "booker://page/1/0@2.png",       // the `x` is part of the shape
            "booker://elsewhere/1/0@1x.png", // not a page at all
            "booker://page/1",               // truncated
            "",
        ] {
            assert!(parse(bad).is_none(), "{bad} should not parse");
        }
    }
}

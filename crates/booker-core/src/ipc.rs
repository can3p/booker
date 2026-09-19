//! The command surface the application exposes to its UI.
//!
//! Two rules (`AGENTS.md` §3, §6):
//!
//! * This list is **append-only** within a wave, and is edited only in an
//!   integration commit, so that parallel tracks never conflict over it.
//! * Every command here must have an equivalent in the CLI. The agent-facing
//!   surface is a thin layer over the same core, never a second
//!   implementation (`PLAN.md` §11.3).

/// Names of the commands the app exposes. Keep them sorted; add, never
/// renumber or reuse.
pub const COMMANDS: &[&str] = &[
    "compile",
    "open_project",
    "page_image_url",
    "project_info",
    "render_page",
];

/// Page images are served over a custom protocol rather than passed through
/// IPC, because a preview scrolls fast and base64 through a JSON channel is
/// the slowest thing in the app (`PLAN.md` §4).
///
/// Shape: `booker://page/<revision>/<page>@<scale>x.<png|svg>`
///
/// The revision is in the path on purpose: an image is immutable for a given
/// revision, so the webview may cache it forever, and a stale one can never
/// be mistaken for a current one.
pub fn page_image_url(revision: u64, page: u32, scale: f32, format: &str) -> String {
    format!("booker://page/{revision}/{page}@{scale}x.{format}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_sorted_and_unique() {
        let mut sorted = COMMANDS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.as_slice(),
            COMMANDS,
            "keep the command list sorted and free of duplicates"
        );
    }

    #[test]
    fn page_urls_carry_the_revision() {
        assert_eq!(page_image_url(7, 3, 2.0, "png"), "booker://page/7/3@2x.png");
    }
}

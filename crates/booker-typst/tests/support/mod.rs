//! Shared scaffolding for the engine's integration tests.
//!
//! Each test file is its own crate, so not every helper is used by every one
//! of them.
#![allow(dead_code)]

use std::path::Path;

use booker_core::{
    CompileRequest, CompileTarget, ProjectRef, RenderFormat, RenderRequest, Revision,
};
use tempfile::TempDir;

/// A fixture project checked into the repository, under `fixtures/engine/`.
pub fn fixture(name: &str) -> ProjectRef {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/engine")
        .join(name);
    assert!(
        root.is_dir(),
        "missing fixture {}; the engine tests need it",
        root.display()
    );
    ProjectRef::new(root)
}

/// A throwaway project on disk, written from `(relative path, contents)`
/// pairs. The directory lives as long as the returned handle.
pub fn temp_project(files: &[(&str, &str)]) -> (TempDir, ProjectRef) {
    let dir = tempfile::tempdir().expect("a temporary directory");
    for (path, contents) in files {
        let path = dir.path().join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("a directory for the fixture file");
        }
        std::fs::write(&path, contents).expect("to write the fixture file");
    }
    let project = ProjectRef::new(dir.path().to_path_buf());
    (dir, project)
}

pub fn compile_request(project: &ProjectRef, target: CompileTarget) -> CompileRequest {
    CompileRequest {
        project: project.clone(),
        target,
        revision: Revision(1),
    }
}

pub fn render_request(
    project: &ProjectRef,
    page: u32,
    scale: f32,
    format: RenderFormat,
) -> RenderRequest {
    RenderRequest {
        project: project.clone(),
        revision: Revision(1),
        page,
        scale,
        format,
    }
}

/// The width and height a PNG declares in its header.
pub fn png_size(bytes: &[u8]) -> (u32, u32) {
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "not a PNG: {:?}",
        &bytes[..bytes.len().min(8)]
    );
    let number =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    (number(16), number(20))
}

/// A book of roughly `chapters * 5` pages of prose, generated rather than
/// checked in: 200 pages of text is a lot of repository for something a
/// function can produce identically every time.
///
/// The text is deterministic (a fixed-seed shuffle of a small word list), so
/// two runs lay out to exactly the same pages.
pub fn generated_book(chapters: usize) -> String {
    const WORDS: &[&str] = &[
        "garden", "door", "candle", "ivy", "hinge", "letter", "morning", "river", "kettle",
        "sparrow", "thread", "window", "path", "bell", "orchard", "rain", "shoe", "clock",
        "brother", "lantern", "pebble", "meadow", "cousin", "fence",
    ];

    let mut out = String::from(
        "#set page(width: 148mm, height: 210mm, margin: (x: 18mm, y: 20mm), numbering: \"1\")\n\
         #set text(font: \"Libertinus Serif\", size: 11pt, lang: \"en\")\n\
         #set par(justify: true)\n\n",
    );

    let mut seed: u64 = 0x5eed_1234;
    let mut next = move || {
        // xorshift64: small, deterministic, and good enough to make prose
        // that does not compress into one identical paragraph.
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    for chapter in 1..=chapters {
        out.push_str(&format!("\n= Chapter {chapter}\n\n"));
        for _ in 0..14 {
            for word in 0..90 {
                if word > 0 {
                    out.push(' ');
                }
                out.push_str(WORDS[(next() % WORDS.len() as u64) as usize]);
            }
            out.push_str(".\n\n");
        }
    }
    out
}

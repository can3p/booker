//! What makes an embedded Typst worth having: the second compile.
//!
//! The engine keeps the compiler's memoized state alive between compiles, so
//! a typed character re-lays-out the paragraph it landed in rather than the
//! whole book. This file both proves that and measures it; the measurement
//! from the big run is written up in `docs/FINDINGS.md`.
//!
//! Run the 200-page measurement by hand, in release, where the numbers mean
//! something:
//!
//! ```text
//! cargo test -p booker-typst --release --test incremental -- --ignored --nocapture
//! ```

mod support;

use std::time::Instant;

use booker_core::{CompileTarget, ProjectRef, RenderFormat};
use booker_typst::Engine;
use support::{compile_request, generated_book, render_request, temp_project};

/// One run of the experiment.
struct Timings {
    pages: usize,
    open_ms: u128,
    cold_ms: u128,
    edit_ms: u128,
    unchanged_ms: u128,
}

impl Timings {
    fn report(&self, label: &str) {
        println!(
            "\n{label}: {} pages\n  \
             open (font loading)      {:>6} ms\n  \
             cold compile             {:>6} ms\n  \
             recompile, one character {:>6} ms  ({:.0}x faster)\n  \
             recompile, no change     {:>6} ms  ({:.0}x faster)\n",
            self.pages,
            self.open_ms,
            self.cold_ms,
            self.edit_ms,
            self.cold_ms as f64 / self.edit_ms.max(1) as f64,
            self.unchanged_ms,
            self.cold_ms as f64 / self.unchanged_ms.max(1) as f64,
        );
    }
}

/// Compiles a generated book cold, then again after a one-character edit in
/// its last chapter, then again with nothing changed at all.
///
/// Also checks that the edit actually landed: the pages before it are byte
/// for byte what they were, the pages after it are not.
fn run(chapters: usize) -> Timings {
    let book = generated_book(chapters);
    let (_dir, project) = temp_project(&[("main.typ", &book)]);

    // A cold compile means a cold compiler: drop every memoized layout that
    // an earlier test or an earlier run left behind. Twice, because one
    // round only ages the entries.
    comemo::evict(0);
    comemo::evict(0);

    let open_started = Instant::now();
    let mut engine = Engine::open(project.clone()).expect("to open the project");
    let open_ms = open_started.elapsed().as_millis();

    let cold_started = Instant::now();
    let cold = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("the generated book to compile");
    let cold_ms = cold_started.elapsed().as_millis();
    assert!(
        cold.result.diagnostics.is_empty(),
        "the generated book should be clean: {:?}",
        cold.result.diagnostics
    );
    let pages = cold.result.pages.len();

    let first_page_before = page(&mut engine, &project, 0);
    let tail_before = tail(&mut engine, &project, pages);

    // One character, typed into the last chapter.
    let edited = type_one_character(&book);
    assert_eq!(edited.len(), book.len() + 1);
    engine
        .set_source("main.typ", edited)
        .expect("to set the buffer");

    let edit_started = Instant::now();
    let warm = engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to recompile");
    let edit_ms = edit_started.elapsed().as_millis();
    assert!(warm.result.diagnostics.is_empty());

    assert_eq!(
        page(&mut engine, &project, 0),
        first_page_before,
        "an edit in the last chapter must not disturb the first page"
    );
    assert_ne!(
        tail(&mut engine, &project, warm.result.pages.len()),
        tail_before,
        "the edited text must actually reach the pages"
    );

    let unchanged_started = Instant::now();
    engine
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to recompile");
    let unchanged_ms = unchanged_started.elapsed().as_millis();

    Timings {
        pages,
        open_ms,
        cold_ms,
        edit_ms,
        unchanged_ms,
    }
}

/// The last chapter's worth of pages — where an edit at the top of the last
/// chapter has to show up.
fn tail(engine: &mut Engine, project: &ProjectRef, pages: usize) -> Vec<Vec<u8>> {
    let from = pages.saturating_sub(6);
    (from..pages)
        .map(|index| page(engine, project, index as u32))
        .collect()
}

fn page(engine: &mut Engine, project: &ProjectRef, index: u32) -> Vec<u8> {
    engine
        .render(&render_request(project, index, 1.0, RenderFormat::Png))
        .unwrap_or_else(|err| panic!("to render page {index}: {err}"))
}

/// Inserts a single character into the last chapter's first word.
fn type_one_character(book: &str) -> String {
    let at = book
        .rfind("= Chapter")
        .and_then(|heading| {
            book[heading..]
                .find("\n\n")
                .map(|offset| heading + offset + 2)
        })
        .expect("the generated book to have chapters");
    let mut edited = String::with_capacity(book.len() + 1);
    edited.push_str(&book[..at]);
    edited.push('X');
    edited.push_str(&book[at..]);
    edited
}

#[test]
fn a_one_character_edit_costs_a_fraction_of_a_cold_compile() {
    // Small enough to stay a test (this runs unoptimised in CI), big enough
    // that the difference is the incremental compiler and not the noise.
    let timings = run(8);
    timings.report("incremental compile");

    assert!(
        timings.pages > 15,
        "expected a book of some size, got {} pages",
        timings.pages
    );
    assert!(
        timings.edit_ms * 3 < timings.cold_ms,
        "a one-character edit took {} ms against a cold compile of {} ms; \
         the memoized state is not being reused",
        timings.edit_ms,
        timings.cold_ms
    );
}

/// The number that goes in `docs/FINDINGS.md`. Ignored by default: 200 pages
/// unoptimised is slow enough to annoy, and the figure is only meaningful in
/// a release build anyway.
#[test]
#[ignore = "measurement, not a check; run with --release --ignored --nocapture"]
fn two_hundred_pages() {
    let timings = run(50);
    timings.report("200-page novel");
    assert!(timings.pages > 150, "{} pages", timings.pages);
}

//! Rendering a page to the bytes the preview draws.

mod support;

use booker_core::{CompileTarget, Error, RenderFormat};
use booker_typst::Engine;
use support::{compile_request, fixture, png_size, render_request, temp_project};

/// A CSS pixel is 1/96 inch, a typographic point is 1/72 inch: an A5 page is
/// 148 mm = 419.53 pt = 559.37 CSS pixels wide, and 210 mm = 793.70 tall.
const A5_CSS_WIDTH: u32 = 559;
const A5_CSS_HEIGHT: u32 = 794;

/// What `RenderRequest::scale` means: pixels per CSS pixel.
fn css_pixels(mm: f64, scale: f64) -> u32 {
    (mm / 25.4 * 96.0 * scale).round() as u32
}

#[test]
fn a_page_renders_to_a_png_at_the_size_the_scale_asks_for() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let single = engine
        .render(&render_request(&project, 1, 1.0, RenderFormat::Png))
        .expect("to render page 1");
    assert_eq!(
        png_size(&single),
        (A5_CSS_WIDTH, A5_CSS_HEIGHT),
        "at scale 1 a page is its own size in CSS pixels"
    );

    for scale in [0.5, 2.0, 3.0] {
        let bytes = engine
            .render(&render_request(
                &project,
                1,
                scale as f32,
                RenderFormat::Png,
            ))
            .expect("to render page 1");
        assert_eq!(
            png_size(&bytes),
            (css_pixels(148.0, scale), css_pixels(210.0, scale)),
            "at scale {scale}"
        );
    }
}

#[test]
fn rendering_does_not_need_a_compile_first_but_agrees_with_one() {
    let project = fixture("minimal");

    let mut fresh = Engine::open(project.clone()).expect("to open the fixture");
    let without = fresh
        .render(&render_request(&project, 0, 1.0, RenderFormat::Png))
        .expect("render on its own compiles what it needs");

    let mut compiled_first = Engine::open(project.clone()).expect("to open the fixture");
    compiled_first
        .compile(&compile_request(&project, CompileTarget::Layout))
        .expect("to compile");
    let with = compiled_first
        .render(&render_request(&project, 0, 1.0, RenderFormat::Png))
        .expect("to render");

    assert_eq!(without, with, "rendering must be deterministic");
}

#[test]
fn each_page_renders_to_something_different() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let title = engine
        .render(&render_request(&project, 0, 1.0, RenderFormat::Png))
        .expect("to render the title page");
    let chapter = engine
        .render(&render_request(&project, 1, 1.0, RenderFormat::Png))
        .expect("to render the first chapter");

    assert_ne!(title, chapter, "page 0 and page 1 are not the same page");
}

#[test]
fn a_page_renders_to_svg() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let bytes = engine
        .render(&render_request(&project, 1, 1.0, RenderFormat::Svg))
        .expect("to render page 1 as SVG");
    let svg = String::from_utf8(bytes).expect("SVG is text");

    assert!(svg.starts_with("<svg"), "{}", &svg[..svg.len().min(80)]);
    assert!(
        svg.contains("419.52") && svg.contains("595.27"),
        "the SVG should carry the page size in points"
    );
    // Typst draws text as glyph outlines, so there is no searchable string in
    // here — what there is, is geometry.
    assert!(svg.contains("<path"), "the page should have been drawn");

    let title = engine
        .render(&render_request(&project, 0, 1.0, RenderFormat::Svg))
        .expect("to render the title page as SVG");
    assert_ne!(title, svg.into_bytes(), "each page draws its own content");
}

#[test]
fn asking_for_a_page_that_does_not_exist_says_how_many_there_are() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    let err = engine
        .render(&render_request(&project, 99, 1.0, RenderFormat::Png))
        .expect_err("there is no page 99");

    match err {
        Error::NoSuchPage { requested, total } => {
            assert_eq!(requested, 99);
            assert_eq!(total, 4);
        }
        other => panic!("expected a NoSuchPage error, got {other:?}"),
    }
    assert!(err.to_string().contains("the document has 4"), "{err}");
}

#[test]
fn a_nonsense_scale_is_refused_rather_than_attempted() {
    let project = fixture("minimal");
    let mut engine = Engine::open(project.clone()).expect("to open the fixture");

    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let err = engine
            .render(&render_request(&project, 0, scale, RenderFormat::Png))
            .expect_err("a scale of {scale} makes no sense");
        assert!(
            err.to_string().contains("positive number"),
            "scale {scale}: {err}"
        );
    }
}

#[test]
fn a_page_of_a_book_that_does_not_compile_is_an_error_not_a_panic() {
    let (_dir, project) = temp_project(&[("main.typ", "#let x = undefined_thing\n")]);
    let mut engine = Engine::open(project.clone()).expect("to open the project");

    let err = engine
        .render(&render_request(&project, 0, 1.0, RenderFormat::Png))
        .expect_err("there is nothing to render");
    assert!(
        err.to_string().contains("unknown variable"),
        "the error should repeat what is actually wrong: {err}"
    );
}

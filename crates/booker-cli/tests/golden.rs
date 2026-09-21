//! Golden tests: each book in `fixtures/golden/` is laid out, every page is
//! rendered, and the pictures are compared with the snapshots committed
//! beside it. This is the backbone of layout work (`AGENTS.md` §6): a
//! change to how books look is caught as the page it moved, not by a
//! reader months later.
//!
//! To accept a change that was meant, regenerate — `cargo xtask golden`,
//! which runs this test with `BOOKER_UPDATE_GOLDEN=1` — and look at every
//! image before committing. On a failure, the new render and a picture of
//! the difference are written under `target/golden/` and the message says
//! where.

use std::path::{Path, PathBuf};

use booker_core::{CompileTarget, RenderFormat, RenderRequest};
use booker_project::Project;

/// 48 pixels to the inch: small snapshots, and a margin that moves by a
/// millimetre still moves by two pixels.
const SCALE: f32 = 0.5;

/// A pixel differs when a channel is off by more than this, out of 255 —
/// enough to ignore anti-aliasing noise, not enough to miss a moved glyph.
const CHANNEL_TOLERANCE: u8 = 24;

/// A page fails when more than this share of its pixels differ.
const PAGE_TOLERANCE: f64 = 0.001;

const BOOKS: &[&str] = &["novel", "picture-book", "poetry", "paper"];

/// The repository, spelled without `..` so the paths in a failure message
/// can be pasted as they are.
fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/booker-cli sits two levels down")
}

fn golden(book: &str) -> PathBuf {
    workspace().join("fixtures/golden").join(book)
}

fn failures(book: &str) -> PathBuf {
    workspace().join("target/golden").join(book)
}

struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode(bytes: &[u8]) -> Image {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().expect("a PNG");
    let mut buffer = vec![0; reader.output_buffer_size().expect("a size")];
    let info = reader.next_frame(&mut buffer).expect("a frame");
    buffer.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => buffer
            .chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buffer
            .chunks(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buffer.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
        png::ColorType::Indexed => unreachable!("EXPAND turns palettes into colour"),
    };
    Image {
        width: info.width,
        height: info.height,
        rgba,
    }
}

fn encode(image: &Image) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("a header");
    writer.write_image_data(&image.rgba).expect("the pixels");
    drop(writer);
    out
}

/// How many pixels differ, and a picture of where: the expected page faded,
/// with every differing pixel in red.
fn compare(expected: &Image, actual: &Image) -> (usize, Image) {
    let mut differing = 0;
    let mut picture = Vec::with_capacity(expected.rgba.len());
    for (e, a) in expected.rgba.chunks(4).zip(actual.rgba.chunks(4)) {
        let differs = e
            .iter()
            .zip(a)
            .any(|(x, y)| x.abs_diff(*y) > CHANNEL_TOLERANCE);
        if differs {
            differing += 1;
            picture.extend([230, 0, 0, 255]);
        } else {
            let faded = 255 - (255 - e[0]) / 4;
            picture.extend([faded, faded, faded, 255]);
        }
    }
    (
        differing,
        Image {
            width: expected.width,
            height: expected.height,
            rgba: picture,
        },
    )
}

/// Every page of a golden book, rendered.
fn render(book: &str) -> Vec<Vec<u8>> {
    let project = Project::load(golden(book)).expect("the golden book loads");
    let (mut engine, compilation) =
        booker_cli::engine::lay_out(&project, CompileTarget::Layout).expect("it lays out");
    assert!(
        !compilation.has_errors(),
        "{book} must lay out without layout errors: {:#?}",
        compilation.result.diagnostics
    );
    (0..compilation.result.pages.len() as u32)
        .map(|page| {
            engine
                .render(&RenderRequest {
                    project: project.reference().clone(),
                    revision: project.revision(),
                    page,
                    scale: SCALE,
                    format: RenderFormat::Png,
                })
                .expect("the page renders")
        })
        .collect()
}

fn snapshot(book: &str, page: usize) -> PathBuf {
    golden(book)
        .join("snapshots")
        .join(format!("page-{:02}.png", page + 1))
}

#[test]
fn every_golden_book_looks_the_way_it_did() {
    let update = std::env::var_os("BOOKER_UPDATE_GOLDEN").is_some();
    let mut problems = Vec::new();

    for book in BOOKS {
        let pages = render(book);
        let directory = golden(book).join("snapshots");

        if update {
            let _ = std::fs::remove_dir_all(&directory);
            std::fs::create_dir_all(&directory).unwrap();
            for (index, page) in pages.iter().enumerate() {
                std::fs::write(snapshot(book, index), page).unwrap();
            }
            continue;
        }

        let committed = std::fs::read_dir(&directory)
            .map(|entries| entries.count())
            .unwrap_or(0);
        if committed != pages.len() {
            problems.push(format!(
                "{book}: {} pages now, {committed} snapshots — the book reflowed; \
                 if that was meant, run `cargo xtask golden` and look at the pages",
                pages.len()
            ));
            continue;
        }

        for (index, page) in pages.iter().enumerate() {
            let expected = decode(&std::fs::read(snapshot(book, index)).unwrap());
            let actual = decode(page);
            let place = format!("{book} page {}", index + 1);
            if (expected.width, expected.height) != (actual.width, actual.height) {
                problems.push(format!(
                    "{place}: {}×{} px now, {}×{} px in the snapshot — the page size changed",
                    actual.width, actual.height, expected.width, expected.height
                ));
                continue;
            }
            let (differing, picture) = compare(&expected, &actual);
            let share = differing as f64 / f64::from(expected.width * expected.height);
            if share > PAGE_TOLERANCE {
                let out = failures(book);
                std::fs::create_dir_all(&out).unwrap();
                let stem = format!("page-{:02}", index + 1);
                std::fs::write(out.join(format!("{stem}.actual.png")), page).unwrap();
                std::fs::write(out.join(format!("{stem}.diff.png")), encode(&picture)).unwrap();
                problems.push(format!(
                    "{place}: {differing} pixels differ ({:.2}%); the new render and a \
                     picture of the difference are in {}",
                    share * 100.0,
                    out.display()
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "the books do not look the way they did:\n  {}",
        problems.join("\n  ")
    );
}

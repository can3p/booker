//! Compile and render, behind the Wave 0 contracts.

use std::path::Path;
use std::time::Instant;

use booker_core::{
    CompileRequest, CompileResult, CompileTarget, Error, Length, PageInfo, ProjectRef,
    RenderFormat, RenderRequest, Result, Severity,
};
use typst::diag::{SourceResult, Warned};
use typst::model::Numbering;
use typst::utils::Scalar;
use typst_layout::{Page, PagedDocument};
use typst_pdf::PdfOptions;
use typst_render::RenderOptions;
use typst_svg::SvgOptions;

use crate::diagnostics;
use crate::fonts::Fonts;
use crate::world::{BookerWorld, DEFAULT_ENTRYPOINT};

/// How many compilations a memoized layout survives without being used.
///
/// This is the knob behind incremental compilation: everything Typst worked
/// out last time stays in the cache, and a one-character edit only redoes the
/// parts that actually changed. Thirty is generous enough that switching
/// between two chapters and back still hits the cache, and bounded enough
/// that the cache does not grow forever.
const CACHE_GENERATIONS: usize = 30;

/// A Typst pixel is a point (1/72 inch); a CSS pixel is 1/96 inch.
/// `RenderRequest::scale` is in CSS pixels, so this is the bridge.
const POINTS_PER_CSS_PIXEL: f64 = 96.0 / 72.0;

/// A live compiler for one project.
///
/// Keep one of these per open project and call [`compile`](Self::compile) as
/// often as you like: the compiler's memoized state lives here, so the second
/// compile of a book is much cheaper than the first (see `docs/FINDINGS.md`,
/// "Typst incremental recompilation is worth keeping an engine alive for").
///
/// Dropping the engine throws that state away, so don't build one per
/// keystroke.
pub struct Engine {
    project: ProjectRef,
    world: BookerWorld,
    /// The last document that laid out successfully, kept so that rendering a
    /// page does not have to compile again.
    document: Option<PagedDocument>,
}

/// What a compile produced: the contract's result, plus the bytes when a file
/// was asked for.
#[derive(Debug, Clone)]
pub struct Compilation {
    /// Pages, diagnostics and timing — what crosses the IPC boundary.
    pub result: CompileResult,
    /// Present when the request asked for [`CompileTarget::Pdf`] and the
    /// export succeeded. A failed export is a diagnostic in `result`, not an
    /// error: the pages are still good and the user should see them.
    pub pdf: Option<Vec<u8>>,
}

impl Compilation {
    /// Whether the compile produced any error-severity diagnostic.
    pub fn has_errors(&self) -> bool {
        self.result
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

impl Engine {
    /// Opens a project, compiling `main.typ` at its root.
    pub fn open(project: ProjectRef) -> Result<Self> {
        Self::open_with_entrypoint(project, DEFAULT_ENTRYPOINT)
    }

    /// Opens a project with an entry file other than `main.typ`.
    ///
    /// The file need not exist: a project that is missing it still opens, and
    /// says so as a diagnostic on the first compile (`AGENTS.md` §7).
    pub fn open_with_entrypoint(project: ProjectRef, entrypoint: impl AsRef<Path>) -> Result<Self> {
        let world = BookerWorld::new(project.root.clone(), entrypoint)?;
        Ok(Self {
            project,
            world,
            document: None,
        })
    }

    /// The project this engine is bound to.
    pub fn project(&self) -> &ProjectRef {
        &self.project
    }

    /// The entry file, relative to the project root.
    pub fn entrypoint(&self) -> &Path {
        self.world.entrypoint()
    }

    /// The faces this project can use.
    pub fn fonts(&self) -> &Fonts {
        self.world.fonts()
    }

    /// Hands the engine the text an editor is holding for a file, instead of
    /// what is on disk. This is how an unsaved buffer reaches the preview.
    pub fn set_source(&mut self, path: impl AsRef<Path>, text: impl Into<String>) -> Result<()> {
        self.world.set_override(path, text)
    }

    /// Drops an unsaved buffer; the file on disk speaks for itself again.
    pub fn clear_source(&mut self, path: impl AsRef<Path>) -> Result<bool> {
        self.world.clear_override(path)
    }

    /// Drops every unsaved buffer — what a project reload does.
    pub fn clear_sources(&mut self) {
        self.world.clear_overrides();
    }

    /// Lays the book out, and writes a PDF when asked for one.
    ///
    /// Never returns `Err` for anything that is wrong with the *book*: a
    /// broken document comes back as diagnostics with no pages, because that
    /// is exactly when a user needs to see the errors (`AGENTS.md` §7). `Err`
    /// is reserved for a request this engine cannot answer at all, such as
    /// one aimed at a different project.
    pub fn compile(&mut self, request: &CompileRequest) -> Result<Compilation> {
        self.check_project(&request.project)?;

        let started = Instant::now();

        // Pick up anything that changed on disk, keeping parsed sources so
        // they can be edited in place instead of reparsed.
        self.world.reset();

        let Warned { output, warnings } = typst::compile::<PagedDocument>(&self.world);

        let mut diagnostics = self.world.fonts().diagnostics().to_vec();
        diagnostics.extend(diagnostics::convert(&self.world, &warnings));

        let mut pages = Vec::new();
        let mut pdf = None;
        match output {
            Ok(document) => {
                pages = document
                    .pages()
                    .iter()
                    .enumerate()
                    .map(|(index, page)| page_info(index, page))
                    .collect();

                if request.target == CompileTarget::Pdf {
                    match export_pdf(&document) {
                        Ok(bytes) => pdf = Some(bytes),
                        Err(errors) => {
                            diagnostics.extend(diagnostics::convert(&self.world, &errors));
                        }
                    }
                }

                self.document = Some(document);
            }
            Err(errors) => {
                // Don't keep the last good document around: a preview showing
                // pages that no longer match the source is worse than one
                // showing the errors.
                self.document = None;
                diagnostics.extend(diagnostics::convert(&self.world, &errors));
            }
        }

        // Keep the memoized layout for the next compile, but let go of what
        // has not been touched for a while.
        comemo::evict(CACHE_GENERATIONS);

        Ok(Compilation {
            result: CompileResult {
                revision: request.revision,
                pages,
                diagnostics,
                duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            },
            pdf,
        })
    }

    /// Renders one page to PNG or SVG.
    ///
    /// Compiles first if the engine has no current document, so a caller that
    /// only wants an image does not have to compile by hand.
    pub fn render(&mut self, request: &RenderRequest) -> Result<Vec<u8>> {
        self.check_project(&request.project)?;

        if !request.scale.is_finite() || request.scale <= 0.0 {
            return Err(Error::Layout(format!(
                "cannot render at scale {}; the scale must be a positive number",
                request.scale
            )));
        }

        if self.document.is_none() {
            let compilation = self.compile(&CompileRequest {
                project: request.project.clone(),
                target: CompileTarget::Layout,
                revision: request.revision,
            })?;
            if self.document.is_none() {
                let first = compilation
                    .result
                    .diagnostics
                    .iter()
                    .find(|d| d.severity == Severity::Error)
                    .map(|d| d.message.clone())
                    .unwrap_or_else(|| "the document did not compile".to_string());
                return Err(Error::Layout(first));
            }
        }

        let Some(document) = &self.document else {
            return Err(Error::Layout("the document did not compile".to_string()));
        };

        let total = u32::try_from(document.pages().len()).unwrap_or(u32::MAX);
        let page = usize::try_from(request.page)
            .ok()
            .and_then(|index| document.pages().get(index))
            .ok_or(Error::NoSuchPage {
                requested: request.page,
                total,
            })?;

        match request.format {
            RenderFormat::Png => {
                let options = RenderOptions {
                    pixel_per_pt: Scalar::new(f64::from(request.scale) * POINTS_PER_CSS_PIXEL),
                    render_bleed: false,
                };
                typst_render::render(page, &options)
                    .encode_png()
                    .map_err(|err| {
                        Error::Layout(format!(
                            "page {} could not be encoded as PNG: {err}",
                            request.page
                        ))
                    })
            }
            RenderFormat::Svg => {
                // SVG is resolution independent: the scale is carried by the
                // viewport, not baked into the geometry.
                let options = SvgOptions {
                    render_bleed: false,
                    pretty: false,
                };
                Ok(typst_svg::svg(page, &options).into_bytes())
            }
        }
    }

    fn check_project(&self, requested: &ProjectRef) -> Result<()> {
        if same_folder(&self.project.root, &requested.root) {
            return Ok(());
        }
        Err(Error::Project {
            path: requested.root.clone(),
            message: format!(
                "this engine is open on {}; make an engine per project",
                self.project.root.display()
            ),
        })
    }
}

fn export_pdf(document: &PagedDocument) -> SourceResult<Vec<u8>> {
    typst_pdf::pdf(document, &PdfOptions::default())
}

/// Two paths naming the same folder. Compared after canonicalisation where
/// possible, so `/books/mia` and `/books/mia/` are the same project.
fn same_folder(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn page_info(index: usize, page: &Page) -> PageInfo {
    let size = page.frame.size();
    PageInfo {
        index: u32::try_from(index).unwrap_or(u32::MAX),
        label: page_label(page),
        // Booker speaks millimetres; Typst speaks points.
        width: Length::mm(size.x.to_mm()),
        height: Length::mm(size.y.to_mm()),
        // Which chapter a page belongs to needs the document model, which
        // arrives in Wave 1 together with Markdown input.
        chapter: None,
    }
}

/// What is printed on the page: "12", "iv", "B-3", or nothing at all.
///
/// A numbering written as a function (`numbering: n => ...`) would need the
/// compiler to evaluate it, so those pages have no label rather than a wrong
/// one.
fn page_label(page: &Page) -> Option<String> {
    match page.numbering.as_ref()? {
        Numbering::Pattern(pattern) => pattern
            .apply(None, &[page.number])
            .ok()
            .map(|s| s.to_string()),
        Numbering::Func(_) => None,
    }
}

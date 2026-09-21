//! `booker` — the same core as the app, on the command line.
//!
//! The commands live here rather than in `main.rs` so that they can be run
//! against a folder in a test, with their output captured. Everything the
//! app can show about a project, this can print (`AGENTS.md` §6), and there
//! is one implementation of it, not two.

pub mod engine;

use std::io::Write;
use std::path::{Path, PathBuf};

use booker_core::{CompileTarget, Diagnostic};
use booker_project::{display_path, format_diagnostic, Project, Severity, Template};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "booker", version, about = "Author books that live in a folder")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new book project.
    New {
        /// Folder to create.
        path: PathBuf,
        /// Starter template: novel, picture-book, poetry or paper.
        #[arg(long, default_value = "novel")]
        template: String,
        /// Title of the book. Taken from the folder name if not given.
        #[arg(long)]
        title: Option<String>,
    },
    /// Lay the book out and write the output.
    Build {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Lay the book out and report every problem, writing nothing.
    Check {
        #[arg(default_value = ".")]
        project: PathBuf,
        /// `human` to read, `json` for a program or an agent.
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
    },
    /// Say which page a line of a chapter landed on: `booker where content/01-the-jar.md:12`.
    Where {
        /// A chapter file and a line, and optionally a column: `FILE:LINE[:COLUMN]`.
        location: String,
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Say which lines of which chapters are on a page: `booker page 3`.
    Page {
        /// The page, counted from 1 as a PDF viewer counts it.
        number: u32,
        #[arg(default_value = ".")]
        project: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Human,
    Json,
}

/// What the process should exit with.
///
/// `1` means the project has errors — the code CI and an agent loop look at
/// (`PLAN.md` §11.3). A command that could not run at all returns an `Err`,
/// which `main` turns into `2`.
pub const OK: i32 = 0;
pub const HAS_ERRORS: i32 = 1;

pub fn run(command: Command, out: &mut impl Write) -> anyhow::Result<i32> {
    match command {
        Command::New {
            path,
            template,
            title,
        } => new(&path, &template, title.as_deref(), out),
        Command::Build { project } => build(&project, out),
        Command::Check { project, format } => check(&project, format, out),
        Command::Where { location, project } => where_is(&project, &location, out),
        Command::Page { number, project } => page(&project, number, out),
    }
}

fn new(
    path: &Path,
    template: &str,
    title: Option<&str>,
    out: &mut impl Write,
) -> anyhow::Result<i32> {
    let template = Template::from_name(template)?;
    let (created, project) = Project::create(path, template, title)?;

    // `display_path`, not `Path::display`: these are project-relative paths,
    // and they must read the same on every platform (`docs/FINDINGS.md`).
    writeln!(
        out,
        "Created a `{}` book in {}",
        template.name(),
        display_path(&created.root)
    )?;
    for file in &created.files {
        writeln!(out, "  {}", display_path(file))?;
    }
    writeln!(out)?;
    writeln!(out, "  {} — the book's settings", booker_project::BOOK_TOML)?;
    writeln!(out, "  content/ — the text, one file per chapter")?;
    writeln!(
        out,
        "  AGENTS.md — what this folder is, for whoever opens it next"
    )?;
    writeln!(out)?;
    writeln!(out, "Next:")?;
    writeln!(out, "  booker build {}", display_path(path))?;

    // A starter project that is born with problems would be a bug in the
    // template, so say so loudly rather than quietly shipping it.
    for diagnostic in project.diagnostics() {
        writeln!(out, "  {}", format_diagnostic(diagnostic))?;
    }
    Ok(if project.has_errors() { HAS_ERRORS } else { OK })
}

fn build(root: &Path, out: &mut impl Write) -> anyhow::Result<i32> {
    let project = Project::load(root)?;
    let config = project.config();

    writeln!(
        out,
        "{} — format {}, language {}",
        if config.title.is_empty() {
            "(untitled)"
        } else {
            &config.title
        },
        config.format,
        config.language
    )?;
    if let Some(author) = &config.author {
        writeln!(out, "by {author}")?;
    }

    let (width, height) = config.page.size.dimensions();
    writeln!(
        out,
        "Page {width} × {height}{}, margins {}/{}/{}/{}",
        if config.page.facing { ", facing" } else { "" },
        config.page.margins.top,
        config.page.margins.bottom,
        config.page.margins.inside,
        config.page.margins.outside
    )?;

    // The same summaries the application's sidebar is built from, so the
    // window and the terminal cannot describe a book differently
    // (`AGENTS.md` §6).
    let summaries = project.chapter_summaries();
    let mut words = 0;
    let mut images = 0;
    writeln!(out, "Chapters: {}", summaries.len())?;
    for chapter in &summaries {
        words += chapter.words;
        images += chapter.images;
        writeln!(
            out,
            "  {:<32} {}, {}, {}{}",
            chapter.path,
            plural(chapter.words, "word"),
            plural(chapter.headings, "heading"),
            plural(chapter.images, "image"),
            match &chapter.title {
                Some(title) => format!("   “{title}”"),
                None => String::new(),
            }
        )?;
    }
    writeln!(
        out,
        "  {} and {} in all",
        plural(words, "word"),
        plural(images, "image")
    )?;

    let output = engine::output_path(project.root(), &config.title);
    let shown = display_path(project.relative(&output));
    let (_engine, compilation) = match engine::lay_out(&project, CompileTarget::Pdf) {
        Ok(laid_out) => laid_out,
        Err(reason) => {
            report(&problems(&project, &[]), out)?;
            writeln!(out, "Could not build {shown}: {reason}")?;
            return Ok(HAS_ERRORS);
        }
    };

    let problems = problems(&project, &compilation.result.diagnostics);
    report(&problems, out)?;
    if let Some(pdf) = &compilation.pdf {
        if let Err(reason) = engine::write_pdf(pdf, &output) {
            writeln!(out, "Could not build {shown}: {reason}")?;
            return Ok(HAS_ERRORS);
        }
    }
    writeln!(
        out,
        "Built {shown} — {} in {} ms",
        plural(compilation.result.pages.len(), "page"),
        compilation.result.duration_ms
    )?;

    Ok(exit_code(&problems))
}

/// `booker check`: everything `build` would report, and nothing written.
///
/// The problems are the same list the window's problems panel shows — what
/// loading the project found, then what laying it out found — so a person,
/// CI or an agent in a loop sees the same thing (`AGENTS.md` §6).
fn check(root: &Path, format: Format, out: &mut impl Write) -> anyhow::Result<i32> {
    let project = Project::load(root)?;
    let layout = match engine::lay_out(&project, CompileTarget::Layout) {
        Ok((_engine, compilation)) => compilation.result.diagnostics,
        Err(reason) => anyhow::bail!("could not lay the book out: {reason}"),
    };
    let problems = problems(&project, &layout);
    match format {
        Format::Human => report(&problems, out)?,
        Format::Json => {
            serde_json::to_writer_pretty(&mut *out, &problems)?;
            writeln!(out)?;
        }
    }
    Ok(exit_code(&problems))
}

/// `booker where FILE:LINE[:COLUMN]`: the page or pages that line is on.
fn where_is(root: &Path, location: &str, out: &mut impl Write) -> anyhow::Result<i32> {
    let project = Project::load(root)?;
    let (file, line, column) = parse_location(location)?;
    let Some(index) = project
        .chapters()
        .iter()
        .position(|chapter| display_path(chapter.relative_path()) == file)
    else {
        let chapters: Vec<String> = project
            .chapters()
            .iter()
            .map(|chapter| display_path(chapter.relative_path()))
            .collect();
        anyhow::bail!(
            "`{file}` is not a chapter of this book; its chapters are: {}",
            chapters.join(", ")
        );
    };
    let chapter = &project.chapters()[index];
    let Some(offset) = chapter.offset_at(line, column) else {
        anyhow::bail!("`{file}` has no line {line}");
    };
    let (engine, _compilation) =
        engine::lay_out(&project, CompileTarget::Layout).map_err(anyhow::Error::msg)?;

    let mut pages: Vec<u32> = engine
        .pages_at(index, offset)
        .into_iter()
        .map(|point| point.page + 1)
        .collect();
    pages.sort_unstable();
    pages.dedup();
    let place = if column > 1 {
        format!("{file}:{line}:{column}")
    } else {
        format!("{file}:{line}")
    };
    match pages.as_slice() {
        [] => {
            writeln!(out, "{place} is not on any page: nothing there is printed")?;
            Ok(HAS_ERRORS)
        }
        [page] => {
            writeln!(out, "{place} is on page {page}")?;
            Ok(OK)
        }
        many => {
            let list: Vec<String> = many.iter().map(u32::to_string).collect();
            writeln!(out, "{place} is on pages {}", list.join(", "))?;
            Ok(OK)
        }
    }
}

/// `booker page N`: which lines of which chapters are printed on page N.
fn page(root: &Path, number: u32, out: &mut impl Write) -> anyhow::Result<i32> {
    let project = Project::load(root)?;
    let (engine, compilation) =
        engine::lay_out(&project, CompileTarget::Layout).map_err(anyhow::Error::msg)?;
    let total = compilation.result.pages.len();
    let Some(sources) = number
        .checked_sub(1)
        .and_then(|index| engine.page_sources(index))
    else {
        anyhow::bail!(
            "there is no page {number}; the book has {}",
            plural(total, "page")
        );
    };

    // First and last line per chapter, in the order the chapters appear.
    let mut ranges: Vec<(usize, u32, u32)> = Vec::new();
    for (chapter, offset) in sources {
        let line = project.chapters()[chapter]
            .location(booker_doc::Span::new(offset, offset))
            .line;
        match ranges.iter_mut().find(|(c, _, _)| *c == chapter) {
            Some((_, first, last)) => {
                *first = (*first).min(line);
                *last = (*last).max(line);
            }
            None => ranges.push((chapter, line, line)),
        }
    }

    if ranges.is_empty() {
        writeln!(out, "Page {number} has no text from any chapter on it")?;
        return Ok(OK);
    }
    writeln!(out, "Page {number} shows:")?;
    for (chapter, first, last) in ranges {
        let file = display_path(project.chapters()[chapter].relative_path());
        if first == last {
            writeln!(out, "  {file}:{first}")?;
        } else {
            writeln!(out, "  {file}:{first}–{last}")?;
        }
    }
    Ok(OK)
}

/// `content/01.md:12` or `content/01.md:12:5`.
fn parse_location(location: &str) -> anyhow::Result<(String, u32, u32)> {
    let mut parts = location.rsplitn(3, ':').collect::<Vec<_>>();
    parts.reverse();
    let bad = || {
        anyhow::anyhow!(
            "`{location}` is not a place in a chapter; write the file and the line, \
             like `content/01-the-jar.md:12`"
        )
    };
    let number = |text: &str| text.parse::<u32>().ok().filter(|n| *n > 0);
    match parts.as_slice() {
        [file, line, column] if number(line).is_some() && number(column).is_some() => Ok((
            file.replace('\\', "/"),
            number(line).ok_or_else(bad)?,
            number(column).ok_or_else(bad)?,
        )),
        [file, line, column] if number(column).is_some() => Ok((
            format!("{file}:{line}").replace('\\', "/"),
            number(column).ok_or_else(bad)?,
            1,
        )),
        [file, line] => Ok((file.replace('\\', "/"), number(line).ok_or_else(bad)?, 1)),
        _ => Err(bad()),
    }
}

/// What loading found and what laying out found, in one list, ordered by
/// file and line — the order a person fixes things in.
fn problems(project: &Project, layout: &[Diagnostic]) -> Vec<Diagnostic> {
    let mut all: Vec<Diagnostic> = project.diagnostics().to_vec();
    all.extend(layout.iter().cloned());
    all.sort_by(|a, b| {
        let key = |d: &Diagnostic| {
            d.source
                .as_ref()
                .map(|s| (0, display_path(&s.file), s.line, s.column))
                .unwrap_or((1, String::new(), 0, 0))
        };
        key(a).cmp(&key(b))
    });
    all
}

fn report(problems: &[Diagnostic], out: &mut impl Write) -> anyhow::Result<()> {
    if problems.is_empty() {
        writeln!(out, "Problems: none")?;
        return Ok(());
    }
    let count = |severity| problems.iter().filter(|d| d.severity == severity).count();
    writeln!(
        out,
        "Problems: {}, {}",
        plural(count(Severity::Error), "error"),
        plural(count(Severity::Warning), "warning")
    )?;
    for diagnostic in problems {
        writeln!(out, "  {}", format_diagnostic(diagnostic))?;
    }
    Ok(())
}

fn exit_code(problems: &[Diagnostic]) -> i32 {
    if problems.iter().any(|d| d.severity == Severity::Error) {
        HAS_ERRORS
    } else {
        OK
    }
}

/// `1 word`, `2 words` — a message that says "1 headings" reads as a bug.
fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

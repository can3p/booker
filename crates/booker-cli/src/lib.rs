//! `booker` — the same core as the app, on the command line.
//!
//! The commands live here rather than in `main.rs` so that they can be run
//! against a folder in a test, with their output captured. Everything the
//! app can show about a project, this can print (`AGENTS.md` §6), and there
//! is one implementation of it, not two.

pub mod bridge;
pub mod engine;

use std::io::Write;
use std::path::{Path, PathBuf};

use booker_core::{CompileRequest, CompileTarget};
use booker_project::{format_diagnostic, Project, Severity, Template};
use clap::{Parser, Subcommand};

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
        /// Starter template to use.
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

    writeln!(
        out,
        "Created a `{}` book in {}",
        template.name(),
        created.root.display()
    )?;
    for file in &created.files {
        writeln!(out, "  {}", file.display())?;
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
    writeln!(out, "  booker build {}", path.display())?;

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

    let mut words = 0;
    let mut images = 0;
    writeln!(out, "Chapters: {}", project.chapters().len())?;
    for chapter in project.chapters() {
        let document = chapter.document();
        words += chapter.word_count();
        images += document.images().len();
        writeln!(
            out,
            "  {:<32} {}, {}, {}{}",
            chapter.relative_path().display(),
            plural(chapter.word_count(), "word"),
            plural(document.headings().len(), "heading"),
            plural(document.images().len(), "image"),
            match chapter.title() {
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

    let errors = count(&project, Severity::Error);
    let warnings = count(&project, Severity::Warning);
    if project.diagnostics().is_empty() {
        writeln!(out, "Problems: none")?;
    } else {
        writeln!(
            out,
            "Problems: {}, {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        )?;
        for diagnostic in project.diagnostics() {
            writeln!(out, "  {}", format_diagnostic(diagnostic))?;
        }
    }

    let output = engine::output_path(project.root(), &config.title);
    let request = CompileRequest {
        project: project.reference().clone(),
        target: CompileTarget::Pdf,
        revision: project.revision(),
    };
    let chapter_sources: Vec<(&str, &booker_doc::Document)> = project
        .chapters()
        .iter()
        .map(|chapter| (chapter.source(), chapter.document()))
        .collect();

    match engine::compile(&request, config, &chapter_sources, &output) {
        engine::Outcome::Compiled(result) => {
            writeln!(
                out,
                "Built {} — {} pages in {} ms",
                project.relative(&output).display(),
                result.pages.len(),
                result.duration_ms
            )?;
        }
        engine::Outcome::Failed(reason) => {
            writeln!(
                out,
                "Could not build {}: {reason}",
                project.relative(&output).display()
            )?;
            return Ok(HAS_ERRORS);
        }
    }

    Ok(if project.has_errors() { HAS_ERRORS } else { OK })
}

/// `1 word`, `2 words` — a message that says "1 headings" reads as a bug.
fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

fn count(project: &Project, severity: Severity) -> usize {
    project
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.severity == severity)
        .count()
}

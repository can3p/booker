//! `booker` — the same core as the app, on the command line.
//!
//! Everything the app can show about a project, this can print
//! (`AGENTS.md` §6). Commands arrive wave by wave; `docs/PLAN.md` §11.3 has
//! the full intended surface.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "booker", version, about = "Author books that live in a folder")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new book project.
    New {
        /// Folder to create.
        path: std::path::PathBuf,
        /// Starter template to use.
        #[arg(long, default_value = "novel")]
        template: String,
    },
    /// Lay the book out and write the output.
    Build {
        #[arg(default_value = ".")]
        project: std::path::PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::New { path, template } => {
            anyhow::bail!(
                "`booker new {} --template {template}` arrives in Wave 0 track E",
                path.display()
            )
        }
        Command::Build { project } => {
            anyhow::bail!(
                "`booker build {}` arrives in Wave 0 track E",
                project.display()
            )
        }
    }
}

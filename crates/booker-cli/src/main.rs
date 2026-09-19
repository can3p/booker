//! `booker` — the same core as the app, on the command line.
//!
//! Everything the app can show about a project, this can print
//! (`AGENTS.md` §6). Commands arrive wave by wave; `docs/PLAN.md` §11.3 has
//! the full intended surface. The commands themselves live in the library
//! beside this file, so that they are testable.

use std::io::Write;

use booker_cli::{run, Cli};
use clap::Parser;

fn main() -> std::process::ExitCode {
    let mut stdout = std::io::stdout().lock();
    match run(Cli::parse().command, &mut stdout) {
        Ok(code) => {
            let _ = stdout.flush();
            std::process::ExitCode::from(code as u8)
        }
        Err(error) => {
            let _ = stdout.flush();
            // Never a panic and never a stack trace: a person or an agent
            // reads this (`AGENTS.md` §6).
            let _ = writeln!(std::io::stderr(), "booker: {error}");
            std::process::ExitCode::from(2)
        }
    }
}

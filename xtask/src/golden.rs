//! `cargo xtask golden`: accept how the golden books look now.
//!
//! The comparison and the regeneration are one piece of code — the golden
//! test in `crates/booker-cli/tests/golden.rs` — run in two modes, so the
//! snapshots written here are exactly what the test will compare against.
//! This only asks it to write instead of compare.

use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args(["test", "-p", "booker-cli", "--test", "golden"])
        .env("BOOKER_UPDATE_GOLDEN", "1")
        .status()?;
    anyhow::ensure!(status.success(), "the golden books could not be rendered");
    println!(
        "Snapshots rewritten in fixtures/golden/*/snapshots/. Look at every changed \
         image before committing it: `git status fixtures/golden`."
    );
    Ok(())
}

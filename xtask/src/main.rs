//! Development tasks. Never shipped to anyone.
//!
//! This is where the golden-test regeneration lives, and later the agent
//! eval harness (`AGENTS.md` §6) — a harness, not a product feature, which
//! is why it is here and not in the `booker` binary.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "Development tasks for Booker")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the TypeScript bindings from the Rust contracts.
    Bindings,
    /// Regenerate golden test snapshots, for review before committing.
    Golden,
    /// Run the agent eval scenarios. Costs model time; run it deliberately.
    Eval {
        /// Run only scenarios whose name contains this.
        #[arg(long)]
        filter: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Bindings => {
            anyhow::bail!("run `cargo test -p booker-core export_bindings` for now")
        }
        Command::Golden => anyhow::bail!("golden snapshots arrive with Wave 1 track G"),
        Command::Eval { .. } => anyhow::bail!("the eval harness arrives with Wave 6 track I"),
    }
}

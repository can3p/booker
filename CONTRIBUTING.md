# Working on Booker

Read `AGENTS.md` first: it has the branching rules, the definition of done, and which document records what. This file is only the setup.

## Toolchain

- **Rust** stable, pinned by `rust-toolchain.toml`. Install with [rustup](https://rustup.rs). If `cargo` is not found, add it to the shell: `. "$HOME/.cargo/env"`.
- **Node 20+** and **pnpm** for the application UI (from Wave 0 track B onwards).

## Everyday commands

```bash
cargo test --workspace                      # tests, including the contract tests
cargo fmt --all                             # formatting
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p booker-core export_bindings   # regenerate TypeScript types from the Rust contracts
cargo xtask --help                          # development tasks (golden snapshots, evals)
```

TypeScript bindings in `app/src/lib/bindings/` are generated, not edited, and are not committed.

## Working on several tracks at once

Waves are built in parallel tracks, one git worktree each (`AGENTS.md` §5):

```bash
git worktree add ../booker-wt/w0-engine -b w0/engine wave-0
```

Two things keep this pleasant:

- **Give each worktree its own target directory.** A shared one is locked by Cargo, so parallel builds queue up instead of running together. Put `export CARGO_TARGET_DIR="$PWD/target"` in each worktree's shell, or set it per invocation.
- **Use `sccache`** (`brew install sccache`, then `export RUSTC_WRAPPER=sccache`) so the crates shared between worktrees are compiled once.

Expect a few gigabytes of build output per worktree. `cargo clean` between waves.

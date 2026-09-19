# Working on Booker

Read `AGENTS.md` first: it has the branching rules, the definition of done, and which document records what. This file is only the setup.

## Toolchain

- **Rust** stable, pinned by `rust-toolchain.toml`. Install with [rustup](https://rustup.rs). If `cargo` is not found, add it to the shell: `. "$HOME/.cargo/env"`.
- **Node 20+** and **pnpm** for the application UI (from Wave 0 track B onwards).

## Everyday commands

```bash
cargo test --workspace                      # tests, including the contract tests
cargo test -p booker-cli --test tutorials   # replay the tutorials and check every line they print
cargo fmt --all                             # formatting
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p booker-core export_bindings   # regenerate TypeScript types from the Rust contracts
cargo xtask --help                          # development tasks (golden snapshots, evals)
```

TypeScript bindings in `app/src/lib/bindings/` are generated, not edited, and are not committed.

## What CI checks, and how to run each check yourself

`.github/workflows/ci.yml` runs on every push to `main` and every pull request. Each job has a name and one job of work, so a red run says which rule was broken. Run the same command locally and you will get the same answer — if CI and this table ever disagree, CI is the one that is wrong and fixing it is part of the change.

| Job | What it proves | Locally |
|---|---|---|
| `fmt` | The code is formatted | `cargo fmt --all --check` |
| `clippy` | No warnings, in any target, including tests | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `test` | The suite passes — including the tutorial replay below — and on a pull request into `main`, on macOS, Linux **and** Windows | `cargo test --workspace --all-targets --locked --no-fail-fast` then `cargo test --workspace --doc --locked` |
| `docs` | `cargo doc` builds with no broken intra-doc links | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` |
| `bindings` | The contracts still export TypeScript | `cargo test -p booker-core --locked export_bindings` |
| `deps` | The workspace manifests parse, and the dependency tree is acceptable | `cargo metadata --locked --format-version 1 > /dev/null` then `cargo deny check bans licenses sources advisories` |

`--locked` everywhere: a change that edits a dependency without committing `Cargo.lock` fails rather than quietly resolving something else. `cargo test --all-targets` does not run doc tests, which is why the `test` job runs two commands.

`cargo deny` is a separate tool (`brew install cargo-deny`, or `cargo install cargo-deny --locked`). Its policy is `deny.toml`, and the two things it will most often ask of you are: a crate arriving under a licence not yet in the allow list — add it and say why — and a new security advisory against something deep in the Typst tree. There is no nightly advisory run (CI minutes are limited), so advisories surface on the next pull request; triage it, and if it is not reachable from anything Booker does, add it to `ignore` with the reasoning written next to it.

Three platforms run only on pull requests into `main`, which is where a wave is judged (`AGENTS.md` §4). A pull request into a `wave-N` branch runs the Linux matrix only.

## Tutorials, and why they are a test

`docs/tutorials/` teaches everything a user can do, and `AGENTS.md` §2 makes updating a
tutorial part of the change that alters what it describes. A tutorial that is wrong is worse
than none, and a tutorial goes wrong silently — nothing fails when a printed line changes,
the next reader just finds out. So the tutorials are executed:

```bash
cargo test -p booker-cli --test tutorials
```

That replays every ```console block in `docs/tutorials/` against a real `booker` binary in a
temporary folder and compares each line with what the tutorial claims. It is part of
`cargo test`, so it runs in the existing `test` job on all three platforms — no extra CI job
and no extra minutes beyond the two seconds it takes. **If you change a line Booker prints,
this is the test that goes red, and it names the tutorial file and line to fix.**

To write or change one, read the convention at the bottom of
[`docs/tutorials/index.md`](docs/tutorials/index.md). In short: ```console blocks are run and
their output compared, ```console ignore blocks are shown but not run, a block tagged
`file=<path>` is written to that path so the reader's file and the test's are the same text,
and `…` in an expected line means "anything here" — for durations, not for anything that
ought to be stable. Output is always copied from a real run, never typed from memory.

## Refusing to push to main

`main` moves only through a merged pull request (`AGENTS.md` §3). The repository carries a hook that says so before the push leaves your machine — turn it on once per clone:

```bash
git config core.hooksPath .githooks
```

It is advice, not a gate: `git push --no-verify` goes around it, and branch protection on GitHub is the real thing.

## Working on several tracks at once

Waves are built in parallel tracks, one git worktree each (`AGENTS.md` §5):

```bash
git worktree add ../booker-wt/w0-engine -b w0/engine wave-0
```

Two things keep this pleasant:

- **Give each worktree its own target directory.** A shared one is locked by Cargo, so parallel builds queue up instead of running together. Put `export CARGO_TARGET_DIR="$PWD/target"` in each worktree's shell, or set it per invocation.
- **Use `sccache`** (`brew install sccache`, then `export RUSTC_WRAPPER=sccache`) so the crates shared between worktrees are compiled once.

Expect a few gigabytes of build output per worktree. `cargo clean` between waves.

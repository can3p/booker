# Working on Booker

Read `AGENTS.md` first: it has the branching rules, the definition of done, and which document records what. This file is only the setup.

## Toolchain

- **Rust** stable, pinned by `rust-toolchain.toml`. Install with [rustup](https://rustup.rs). If `cargo` is not found, add it to the shell: `. "$HOME/.cargo/env"`.
- **Node**, pinned by `.tool-versions` — 20.19 or newer, because Vite 8 refuses anything older. With [asdf](https://asdf-vm.com) the repository selects it for you; otherwise install that version yourself.
- **pnpm**, which comes with Node through corepack. Update corepack first, because the one Node 20 ships with cannot fetch a current pnpm:
  ```bash
  npm install -g corepack@latest
  corepack enable pnpm
  ```
  Skipping the first line fails in one of two ways, neither of which mentions corepack's age: `Cannot find matching keyid` when it cannot verify the download, or `Cannot find module …/pnpm.cjs` when it half-installed one. CI does the same two commands.
- **The Linux desktop libraries**, if you build the application on Linux: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `libxdo-dev` and `patchelf`. macOS and Windows carry their webview with the operating system. CI installs the same list from `.github/actions/tauri-system-deps`.

## Everyday commands

```bash
cargo test --workspace                      # tests, including the contract tests
cargo test -p booker-cli --test tutorials   # replay the tutorials and check every line they print
cargo fmt --all                             # formatting
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p booker-core export_bindings   # regenerate TypeScript types from the Rust contracts
cargo xtask --help                          # development tasks (golden snapshots, evals)
cargo xtask third-party                     # rewrite THIRD-PARTY.md from the dependency tree
```

And in `app/`, for the application:

```bash
pnpm install                                # once, and after any change to package.json
pnpm tauri dev                              # build and open the window, reloading as you edit
pnpm check                                  # typecheck the Svelte and TypeScript
pnpm test                                   # the frontend tests
pnpm build                                  # the production frontend bundle, into app/dist/
```

TypeScript bindings in `app/src/lib/bindings/` are generated, not edited, and are not
committed — so **run `cargo test -p booker-core export_bindings` before `pnpm check`** in a
fresh clone, or the typecheck fails on imports that do not exist yet.

## What CI checks, and how to run each check yourself

`.github/workflows/ci.yml` runs on every push to `main` and every pull request. Each job has a name and one job of work, so a red run says which rule was broken. Run the same command locally and you will get the same answer — if CI and this table ever disagree, CI is the one that is wrong and fixing it is part of the change.

| Job | What it proves | Locally |
|---|---|---|
| `fmt` | The code is formatted | `cargo fmt --all --check` |
| `clippy` | No warnings, in any target, including tests | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `test` | The suite passes — including the tutorial replay below — and on a pull request into `main`, on macOS, Linux **and** Windows | `cargo test --workspace --all-targets --locked --no-fail-fast` then `cargo test --workspace --doc --locked` |
| `docs` | `cargo doc` builds with no broken intra-doc links | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` |
| `bindings` | The contracts still export TypeScript | `cargo test -p booker-core --locked export_bindings` |
| `deps` | The workspace manifests parse, the dependency tree is acceptable, and `THIRD-PARTY.md` describes it | `cargo metadata --locked --format-version 1 > /dev/null`, `cargo deny check bans licenses sources advisories`, then `cargo xtask third-party --check` |
| `app` | The application typechecks, its tests pass and its bundle builds | in `app/`: `pnpm lint`, `pnpm test`, `pnpm build` |
| `changes` | Which parts of the tree a pull request touches, so `app` is skipped when it touches none of them | — |

The Rust jobs that compile the whole workspace — `clippy`, `test` and `docs` — install the
Linux desktop libraries first, because `app/src-tauri` is a workspace member from Wave 1 on.
That is `.github/actions/tauri-system-deps`, in one place rather than pasted into three jobs.

**`THIRD-PARTY.md` is generated, not written.** `cargo xtask third-party` reads the licence
data `cargo deny` already produces and rewrites the file; the `deps` job runs it with
`--check` and fails when a dependency has changed and nobody regenerated it. Editing the file
by hand is wasted work — the next run overwrites it. The font notices in it are copied
verbatim from `crates/booker-typst/fonts/NOTICE.txt`, which *is* written by hand.

**The `app` job is skipped when nothing it covers changed.** A `changes` job diffs the pull
request against its base and looks for `app/`, `crates/booker-core/` (the contracts the
TypeScript types are generated from) or the workflow itself. A push to `main` always runs
everything, because that is what a release is cut from.

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

## Making a release

`.github/workflows/release.yml` builds the installers, and only a tag starts it — they cost
far more runner time than the rest of CI put together.

**First, the version.** The application reports the version in the workspace `Cargo.toml`,
not the tag — `tauri.conf.json` sets none — and the updater compares versions, so a tag on a
build that still carries the old number is never offered to anyone. Before tagging, a pull
request into `main` sets the new version in four places:

- `version` under `[workspace.package]` in `Cargo.toml` (then `cargo metadata` updates
  `Cargo.lock`, and `cargo xtask third-party` updates `THIRD-PARTY.md`, which lists our own
  crates too),
- `version` in `app/package.json`,
- the `booker --version` line in `docs/tutorials/01-your-first-book.md` — the tutorial test
  fails until it matches, which is how you find out,
- the `## Unreleased` heading in `CHANGELOG.md`, which becomes the release's.

Then, on `main`, once that is merged and green:

```bash
git switch main && git pull
git tag v0.2.0 && git push origin v0.2.0
```

The workflow first checks that the tag matches both versions and stops in seconds if it does
not. Then it builds bundles for macOS (Apple Silicon and Intel), Windows and Linux, signs the
update artifacts with the key in repository secrets, and creates a **draft** GitHub release
with `latest.json` attached. Nobody receives a draft.

**Then the update check** — `AGENTS.md` §4, the one check that is never skipped. Installed
copies read `releases/latest/download/latest.json`, and GitHub serves that only from a
*published* release, so the check cannot happen while the new release is still a draft:

1. Have the previous release installed (from its release page).
2. Publish the new draft.
3. In the installed copy, **Check for Updates…**: it must offer the new version, install it,
   and restart into it — confirm with **About Booker**.
4. If any of that fails, turn the release back into a draft straight away (Edit → Save as
   draft), so no one else is offered a build that cannot be updated out of.

Write the result into the wave's "Update check" line in `docs/WAVE-LOG.md` only once it is
true.

To find out whether the installers build without releasing anything, run the workflow by
hand from the Actions tab; the bundles are kept as artifacts for a week.

Two kinds of signing are involved and they are not the same thing. The **updater** key is
always used — without it an installed copy refuses the download — and it lives in
`TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. **Apple Developer ID
and Windows Authenticode** are separate, cost money, and are not set up yet; the workflow
already reads their secrets, so signed builds need no change here, only the certificates.
Until then the first launch warns on both platforms.

## Refusing to push to main

`main` moves only through a merged pull request (`AGENTS.md` §3). The repository carries a hook that says so before the push leaves your machine — turn it on once per clone:

```bash
git config core.hooksPath .githooks
```

It is advice, not a gate: `git push --no-verify` goes around it. The real gate is the "protect main" ruleset on GitHub, which refuses a direct push outright, requires a linear history, and requires `fmt`, `clippy`, `test (ubuntu-latest)`, `docs`, `bindings` and `deps` to be green before a pull request can be merged. `rebase` is the only merge method it allows, so a wave lands on `main` as its own commits.

The ruleset is invisible to the older branch-protection API — `gh api repos/can3p/booker/branches/main/protection` answers "Branch not protected". Read it with `gh api repos/can3p/booker/rules/branches/main`.

## Working on several tracks at once

Waves are built in parallel tracks, one git worktree each (`AGENTS.md` §5):

```bash
git worktree add ../booker-wt/w0-engine -b w0/engine wave-0
```

Two things keep this pleasant:

- **Give each worktree its own target directory.** A shared one is locked by Cargo, so parallel builds queue up instead of running together. Put `export CARGO_TARGET_DIR="$PWD/target"` in each worktree's shell, or set it per invocation.
- **Use `sccache`** (`brew install sccache`, then `export RUSTC_WRAPPER=sccache`) so the crates shared between worktrees are compiled once.

Expect a few gigabytes of build output per worktree. `cargo clean` between waves.

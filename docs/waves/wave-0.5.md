# Wave 0.5 — Continuous integration: the gate every later wave passes through

Branch: `wave-0.5` (off `main`) · Finishes as one pull request into `main` · No release tag: there is still nothing installable, the first tag is Wave 1's `v0.2.0`

Read `AGENTS.md` first. This brief assigns the tracks, the paths each owns, and what done means.

## Why this wave exists

`AGENTS.md` §4 says a track is done when `cargo fmt`, `cargo clippy -- -D warnings` and `cargo test` pass. Today nothing checks that but the person who typed the commands, on one machine, on macOS. There is no `.github/` directory at all.

Wave 1 runs four tracks in parallel in four worktrees and merges them; Wave 0 ran two and the merge produced a `Cargo.toml` that merged cleanly and then failed to parse, because both tracks had appended `tempfile = "3"` (`docs/WAVE-LOG.md`). Append-only files prevent textual conflicts, not semantic ones. The check that catches that has to exist **before** the wave that needs it, not inside it, which is why this wave comes first and is small.

It is also the wave that finds out whether the Wave 0 code runs anywhere but one laptop. The suite has never been executed on Linux or Windows.

**Demo at the end of the wave:** open a pull request that adds an unformatted line, a `clippy` warning, a failing test and a duplicated dependency key. CI marks it red four times, each with a message naming what is wrong. Fix them; CI goes green on macOS, Linux and Windows.

## What exists already

- A Cargo workspace of five crates plus `xtask`, with tests in `crates/*/tests` and fixtures in `fixtures/`.
- `rust-toolchain.toml` pinning stable with `rustfmt` and `clippy`.
- `cargo test -p booker-core export_bindings`, which writes the TypeScript bindings into the gitignored `app/src/lib/bindings/`.
- No workflows, no hooks, no dependency policy, and no evidence that any of it works off macOS.

## Scope, and the line around it

In: everything CI needs to check the Rust workspace and keep the repository honest.

Out: anything to do with building, bundling, signing or publishing the application. `release.yml`, `tauri-action`, the updater, code signing and the Linux WebKit system packages stay in **Wave 1 track A**, and `ci.yml` must not grow installer steps. Out too: the golden image suite (Wave 2 track G), the MCP conformance suite and `booker check --format sarif` annotations (Wave 7 tracks H and J). This wave builds the frame those hang on — a workflow file with one job per concern and a documented way to add another.

## Budget

CI minutes are limited and the Typst dependency tree is expensive to compile cold. Three rules, and they are binding:

1. **No scheduled jobs.** No `schedule:` trigger, no nightly, no cron. Everything runs on `push` to `main` and on `pull_request`, and `cargo xtask eval` stays a by-hand command (`AGENTS.md` §6).
2. **One platform by default, three where it matters.** `fmt`, `clippy` and the dependency checks are platform-independent and run once, on Linux. The test suite runs on Linux for every push and pull request, and on macOS and Windows only for pull requests targeting `main` — that is, the wave pull requests, which is exactly where `AGENTS.md` §4 wants the three-platform evidence.
3. **Never compile the same thing twice.** `Swatinem/rust-cache` keyed on `Cargo.lock`, `--locked` on every invocation, `--all-targets` so one build serves the whole job, and a `concurrency` group with `cancel-in-progress` so a superseded push stops burning minutes.

Target: a warm pull-request run finishes in about ten minutes. If it does not, cut the matrix before cutting the checks.

## Tracks

This wave is small enough that the lead can run tracks A and C alone. Track B is the one that may hold surprises. Each track works in its own worktree (`../booker-wt/w05-<track>`) on branch `w05/<track>`, and merges into `wave-0.5` by pull request.

### A. The workflow — tier M (YAML tickets: S)
**Owns:** `.github/workflows/ci.yml`, `.github/ISSUE_TEMPLATE/**`, `.github/pull_request_template.md`.

One workflow, one job per concern, each named for what it proves:

| Job | Command | Where |
|---|---|---|
| `fmt` | `cargo fmt --all --check` | Linux |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Linux |
| `test` | `cargo test --workspace --all-targets --locked` and `cargo test --workspace --doc --locked` | Linux always; macOS and Windows on pull requests into `main` |
| `bindings` | `cargo test -p booker-core export_bindings --locked` | Linux |
| `docs` | `cargo doc --workspace --no-deps --locked` with `RUSTDOCFLAGS=-D warnings` | Linux |
| `deps` | see track C | Linux |

The toolchain comes from `rust-toolchain.toml` — the workflow must not name a Rust version anywhere, so there is one place to change it. `--locked` everywhere means a pull request that edits a dependency without committing `Cargo.lock` fails instead of silently resolving something else.

`cargo test --all-targets` does not run doc tests; that is why the `test` job runs the two commands.

**Done when:** a pull request with a formatting error, a clippy warning and a failing test is red in three separate jobs whose names say which, and the same pull request with those fixed is green on all three platforms.

### B. Green on Linux and Windows — tier M
**Owns:** the test and fixture fixes this turns up, inside the crates they belong to; `docs/FINDINGS.md`.

Wave 0 was written and run on macOS only. Making the suite pass elsewhere is real work, not a formality. Expect to look at:

- **Fonts.** `booker-typst` bundles its fonts, so the engine should not need system fonts — confirm that, because the failure mode on a bare Linux runner is a compile that succeeds with the wrong glyphs rather than one that fails.
- **Line endings.** The `toml_edit` round-trip tests assert that a file comes back byte-identical. Git on Windows will hand them CRLF unless `.gitattributes` says otherwise; adding `.gitattributes` is part of this track.
- **Paths.** Separators and case sensitivity in the project loader and the CLI tests; Linux is case-sensitive where macOS is not.
- **PDF and render output.** Whether a PDF built on two platforms differs, and if so in what — a timestamp, a font hash, floating point. Write the answer down even if nothing this wave depends on it, because Wave 2's golden suite will.

Anything found here goes in `docs/FINDINGS.md` with the platform named. A future session hitting the same wall should not have to rediscover it.

**Done when:** `cargo test --workspace` passes on `ubuntu-latest`, `macos-latest` and `windows-latest`, and every platform difference found is either fixed or written down.

### C. Dependency and repository guards — tier S
**Owns:** `deny.toml`, `.githooks/**`, `CONTRIBUTING.md` (the commands and hook sections), `README.md` (the badge).

- **`cargo metadata --locked --format-version 1 > /dev/null`** as the first step of the `deps` job. This is the Wave 0 bug: it catches a workspace manifest that is textually merged and semantically broken. *(Moved here from Wave 1 track D, where it was listed — it has to exist before Wave 1's four-track merge, not during it. `docs/PLAN.md` §8 and `docs/waves/wave-1.md` are updated accordingly.)*
- **`cargo deny check`** with a `deny.toml` covering advisories, licences and duplicate versions. Two things make it worth a job: it is the gate on a dependency with a known vulnerability, and its licence output is what Wave 1 track D assembles `THIRD-PARTY.md` from, so the data exists once. Pin `cargo-deny` to a version and install it from a cached binary, not from source — building it is minutes we do not have.
- **A pre-push hook refusing a direct push to `main`** (`AGENTS.md` §3), in `.githooks/pre-push`, enabled with `git config core.hooksPath .githooks` and documented in `CONTRIBUTING.md`. It is advisory — a hook is not branch protection, which only the owner can enable (Q-10).
- **`CONTRIBUTING.md` gains one table**: what CI runs, and the exact command to run each check locally before pushing. If the two ever disagree, CI is the one that is wrong.
- **`README.md` gains the CI badge**, and nothing else — CI does not change what the software does.

**Done when:** a branch that duplicates a dependency key fails `deps` with a readable message, `cargo deny check` is clean on the current tree, and a direct push to `main` is refused locally on a fresh clone that ran the documented `git config` line.

## Order of work

1. Wave lead: create `wave-0.5` off `main`, land a workflow skeleton with the `fmt` job only, and confirm Actions is enabled on the repository and a run appears.
2. A and C in parallel. B starts as soon as A's `test` job exists on Linux, because its whole job is reading those failures.
3. Integration on `wave-0.5`: the deliberately-broken pull request, proving each check red, then green.
4. Write the `docs/WAVE-LOG.md` entry with the wall-clock time of a cold run and a warm run — Wave 1 will want to know what its matrix costs. Answer or re-file Q-10. Open the pull request into `main`.

## Exit criteria

- Every pull request into `wave-N` and every push to `main` runs fmt, clippy, tests, doc build, bindings generation and the dependency checks.
- `cargo test --workspace` passes on macOS, Linux and Windows.
- Each check has been seen to fail on purpose, and its failure message names the problem.
- `CONTRIBUTING.md` lists every check with the local command that reproduces it; `README.md` carries the badge.
- `docs/PLAN.md` §8, `docs/waves/wave-1.md`, `docs/WAVE-LOG.md` and `docs/FINDINGS.md` are accurate.

## Which of the `AGENTS.md` §4 wave criteria apply

Criteria 4, 5 and 6 — installers on three platforms, the previous release updating itself, and project-format compatibility across versions — **do not apply to this wave**: there is no application and no release yet, exactly as in Wave 0. They start with Wave 1, which is the first wave that ships something a person can install. Criterion 3 (the MCP suite and evals) starts at Wave 7.

Criteria 1, 2 and 7 apply as written, and criterion 1 is the one this wave exists to make checkable by a machine: the wave ends with a pull request into `main` and a tree where a fresh clone, following `CONTRIBUTING.md`, gets a green `cargo test --workspace` and a working `booker new` / `booker build`. CI proving that on three platforms is the deliverable.

## Not in this wave

Release and bundle workflows, signing, the updater (Wave 1 A). Golden image snapshots (Wave 2 G). Frontend lint and test steps — there is no `package.json` yet; Wave 1 track B adds one and the `pnpm lint` / `pnpm test` jobs with it, in the file this wave creates. MSRV enforcement: the workspace declares `rust-version = "1.92"` because Typst needs it, and Booker ships as a binary rather than as a library anyone compiles, so a dedicated MSRV job is not worth a slot today.

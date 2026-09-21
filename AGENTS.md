# Booker — how to work in this repository

Booker is a desktop book-authoring application: Rust core, Typst as the layout engine, Tauri 2 + Svelte 5 for the app. Projects are plain folders that live in git. The audience is amateurs making kids books, fiction and small non-fiction, so **defaults must produce a good-looking book with no configuration**, and every automatic decision must be overridable.

Booker is also used *by* agents: people will open Claude Code in their book folder and ask it to fix spelling, adjust styling, or repair a broken layout. Two consequences run through everything below — the project must survive being rewritten from outside while the app is open, and every way a book can break must have a named, locatable diagnostic (`docs/PLAN.md` §11).

This file is the standing instruction for any agent session. Read it first, follow it without being asked.

---

## 1. Read these, in this order

| Document | What it is | Who writes it |
|---|---|---|
| `docs/requirements.md` | What the product must do. Changes only when the owner says so. | Owner (agents may add clarifications they were given) |
| `docs/PLAN.md` | Architecture and the plan for the waves still to come. The design source of truth, and **future work only** — §2. | Agents, when a design decision changes |
| `docs/OPEN-QUESTIONS.md` | Everything undecided, with who must decide it and by when | Agents append; owner answers |
| `docs/WAVE-LOG.md` | What each finished wave actually shipped | Agents, at the end of a wave |
| `docs/FINDINGS.md` | Things learned along the way that a future session would otherwise rediscover | Agents, as they learn them |
| `docs/waves/wave-N.md` | The brief for one wave: tracks, owned paths, contracts, done criteria | Wave lead, before tracks start |
| `docs/tutorials/*.md` | Tutorials: a real task walked through end to end, for someone making their first book | Anyone who adds or changes a user-visible capability |
| `README.md` | How to get Booker running and what it can do **today** | Anyone who changes what a user can do |
| `CONTRIBUTING.md` | How to set up and work on the repository | Anyone who changes the setup or the commands |

**Starting a session — ten minutes, in this order.** The point of the order is that you see the software work before you read a line of its source.

1. **Put Rust on the PATH.** It lives in `~/.cargo/bin`, which is not there by default, and nothing below works without it: `. "$HOME/.cargo/env"`.
2. **Read "Where things stand today"**, just below — then the newest entry in `docs/WAVE-LOG.md` (what the last wave shipped and how it deviated), then the brief for the wave you are about to work on in `docs/waves/`. `docs/PLAN.md` §8 is what is still to come; it deliberately says nothing about what already happened (§2).
3. **Run it.** Three minutes, and you know what Booker actually does:

   ```bash
   cargo build --release -p booker-cli     # a few minutes cold, seconds warm
   BOOKER="$PWD/target/release/booker"
   cd "$(mktemp -d)" && "$BOOKER" new demo --title "Demo" && "$BOOKER" build demo
   ```

4. **To find out what a command prints, read `docs/tutorials/01-your-first-book.md`, not `crates/booker-cli/src/lib.rs`.** Every command block in it is a verbatim transcript of a real run, and `cargo test` fails if one drifts — so it is both the shortest and the most reliable description of what the software does today, including what its diagnostics look like.
5. **`git branch -a` and `gh pr list`** — what is in flight. `gh` is installed and authenticated.
6. Say which wave and track you are on before you start changing files.

Scratch books, experiments and rendered pages go in a temporary folder **outside** the repository, never inside it and never committed.

**The repository in one screen:**

| Path | What it is |
|---|---|
| `crates/booker-core` | The contracts every other crate and the app agree on: geometry, project config, diagnostics, compile and render requests. Reaches for nothing — no filesystem, no Typst, no UI. The TypeScript types in `app/src/lib/bindings/` are generated from here with `ts-rs` and are not committed. |
| `crates/booker-doc` | Markdown to the document model. Every node carries the byte range it came from, because click-to-source and every diagnostic depend on it. |
| `crates/booker-project` | Loading, watching and writing a project folder: `book.toml` round-tripping with comments and unknown keys preserved, chapters, templates, the diagnostics about all of it. The rules in §7 are this crate's job. |
| `crates/booker-typst` | The layout engine: Typst embedded in process, the one translation from a book to Typst (`book.rs`, with the span map click-to-source reads), the four built-in themes (`themes.rs`) and the bundled fonts. Typst types never leave this crate (§6), so a Typst upgrade is a change to one crate. |
| `crates/booker-cli` | The `booker` binary. The commands live in `src/lib.rs` rather than `main.rs` so tests can run them and capture their output. |
| `app/` | The Tauri 2 + Svelte 5 application. `src-tauri/` is a workspace member like any other crate — `session.rs` holds one engine per open project, `protocol.rs` answers `booker://` page-image requests, `watch.rs` notices outside edits — and `src/` is the window. The generated TypeScript types land in `src/lib/bindings/` and are not committed. |
| `xtask/` | Development tasks — golden-snapshot regeneration, and from Wave 7 the agent eval harness. Never shipped to anyone. |
| `fixtures/` | Projects the tests load. `broken/` is the one with faults in it on purpose. |
| `.github/workflows/ci.yml` | Every check, one job per concern. `CONTRIBUTING.md` gives the local command for each, and how to run the tutorial replay. |
| `.github/workflows/release.yml` | What a tag does: refuses a tag that differs from the version in `Cargo.toml` and `app/package.json`, then builds installers for four targets, signed update artifacts and a **draft** release. The update check (§4) happens straight after a person publishes it, because installed copies only see published releases; a failed check sends it back to draft. `CONTRIBUTING.md`, "Making a release", is the procedure. |
| `THIRD-PARTY.md` | Generated by `cargo xtask third-party` from the licence data `cargo deny` produces. Never edited by hand; CI fails when it is stale. |

Tests sit next to the code as `#[cfg(test)]` modules and per-crate in `crates/*/tests/`. Three in `booker-cli` are worth knowing about before you write a fourth: `tests/commands.rs` calls the commands in process, `tests/tutorials.rs` replays the tutorials against the real binary, and `tests/golden.rs` renders every book in `fixtures/golden/` and compares the pages with committed snapshots — when a change to how books look is meant, `cargo xtask golden` rewrites them, and you look at every changed image before committing.

**Where things stand today:** Waves 0 to 2 are complete and merged. **Booker is developed locally and not distributed for now** — the owner's decision of 2026-09-21 (`docs/OPEN-QUESTIONS.md` Q-13) — so nothing has been released and there is no installer to download; both the `booker` command and the application run from a source checkout. The release pipeline and the signed updater are in place and a hand-run dry run builds all four targets, but no release has exercised them. There are two ways to use Booker and they share one layout engine. The `booker` command makes a book from one of four templates (`novel`, `picture-book`, `poetry`, `paper`), builds it into a PDF that is typeset like a book — contents, chapters on the right, justified and hyphenated text, typographic quotes and dashes — and answers `check`, `where` and `page`. The application edits the book in a styled Markdown editor beside its pages, goes from a click on a page to the line that made it and back, adds and reorders chapters, and offers both versions when an outside edit meets unsaved typing. CI checks fmt, clippy, the test suite (golden page images and the tutorial replay included) on macOS, Linux and Windows, the doc build, the TypeScript bindings, the dependency policy and third-party notices, and the application's own lint, tests and bundle on every pull request; three tutorials teach everything Booker can currently do. **Wave 3 is next** — "styles", `docs/PLAN.md` §8 — and it has no brief yet; writing `docs/waves/wave-3.md` is the first thing its lead does.

**To run the application** rather than the command line: `cargo test -p booker-core export_bindings` once (the TypeScript types are generated and not committed), then `cd app && pnpm install && pnpm tauri dev`. Node 20.19 is pinned in `.tool-versions` and pnpm comes from `corepack enable pnpm`; on Linux you also need the packages `.github/actions/tauri-system-deps` installs. `CONTRIBUTING.md` has the rest, including how a release is cut.

## 2. Which document gets the write

Put every kind of output in exactly one place:

- **A design decision changes** → edit `docs/PLAN.md` in place. Never keep a competing plan in a new file, never leave the plan describing something the code no longer does.
- **`docs/PLAN.md` holds future work only.** It says what is going to be built and why it is designed that way. Nothing retrospective goes in it: not what a wave shipped, not how it deviated, not what it measured, not what it cost. When a wave finishes, its section in §8 is replaced by a line pointing at `docs/WAVE-LOG.md` — never annotated with an account of what happened. A plan that carries its own history becomes a second, diverging one, and then two documents describe the same wave and a reader cannot tell which is true. The one exception is a decision taken in a finished wave that still constrains what comes next; that is design, and it belongs in the plan stated as a constraint, with the measurement behind it left in `docs/FINDINGS.md`.
- **Something needs the owner to decide** → append to `docs/OPEN-QUESTIONS.md` and keep going with a stated default. Do not block a wave on an unanswered question unless the question makes the work meaningless.
- **A wave finishes** → add its entry to `docs/WAVE-LOG.md` (what shipped, what deviated from the plan, the release tag and PR, what was deferred).
- **A hard-won fact** (a Typst behaviour, a Tauri quirk, a toolchain trap, a benchmark, a dead end and why) → append to `docs/FINDINGS.md`. The test: would a future session waste an hour without this?
- **You had to research something to get oriented** — which crate owns a thing, what a command actually prints, how to run the software at all → put the answer in §1 above, in the same change. Ramp-up is paid once per session and forever; anything a session had to work out for itself, the next one should be able to read. This is not the same as `docs/FINDINGS.md`, which is for hard-won facts about how a dependency or a platform behaves; §1 is for the shape of this repository and how to see it running.
- **A process rule changes** → edit this file, in the same change that introduces the rule.
- **A user-visible capability arrives or changes** → the tutorials must teach it, in the same change (`docs/requirements.md`). Extend the tutorial that covers the surrounding task when there is one, and add a new `docs/tutorials/NN-<name>.md` when the capability is a task of its own. Write it for someone making their first book: a real thing to make, every command or click, and what the screen actually says back. A feature nobody can be walked through is not finished, and a tutorial describing something the software no longer does is worse than none — if a change makes a tutorial wrong, fixing it is part of the change.
- **Anything a user can see or type changes** → update `README.md` in the same change. A new or renamed command, a changed flag or default, a new requirement to install, a different output path, a changed project layout, a capability that starts or stops working: the README must describe what the software does *today*, never what it will do. If a change makes a sentence in the README wrong, fixing that sentence is part of the change, not a follow-up.
- **The setup or the everyday commands change** → update `CONTRIBUTING.md` the same way.

Documentation is part of the work, not a chore after it. A pull request that changes behaviour and leaves the documents describing the old behaviour is incomplete, and so is one that documents something that does not exist yet. When in doubt, describe what you can demonstrate.

Keep every document in this table accurate at the end of every wave. A wave whose documents are stale is not finished.

**And `main` never holds a stale statement, not even briefly.** Some sentences go false on merge rather than on edit — "where things stand today", a `✅ done` marker, a wave log claiming something shipped, anything written in the future tense about work that has since landed. The change that merges carries their update with it; a pull request that would leave `main` describing a state of affairs that ended when it merged is incomplete, and the fix belongs in that pull request rather than in the next one. If staleness reaches `main` anyway, correct it in its own small pull request straight away — do not wait for unrelated work to carry it.

## 3. Branching, waves and pull requests

- **Never commit to `main`.** `main` only moves through a merged pull request, and the server enforces it: the "protect main" ruleset forbids deletion and force-pushes, requires a linear history, and requires `fmt`, `clippy`, `test (ubuntu-latest)`, `docs`, `bindings` and `deps` to be green. A red pull request cannot be merged. (`test (macos-latest)`, `test (windows-latest)` and `app` run and are read, but are not part of the gate — do not merge past them because the ruleset let you.)
- **A pull request into `main` merges by rebase**, the only method the ruleset allows. A wave therefore lands on `main` as its own commits, which is what lets the wave log and the plan point at them — so keep a wave's history worth reading, and rebase rather than merging when a base moves (below).
- **Every new branch is cut from the newest unmerged branch it builds on, and from `main` only when there is nothing unmerged to build on.** This is the general rule; the wave and track rules below are the special cases of it. Before branching, look at what is in flight — `git branch -a` and `gh pr list` — and start from the branch whose work yours continues, even when your change looks unrelated to it. Branching everything off `main` while a wave is open is what makes two lines of work edit the same paragraph of the same document and meet as a conflict later. The same applies to a one-line documentation fix: if a wave is in flight, it belongs on the wave branch, not beside it.
- A wave lives on `wave-N`, which branches **off `main`**, or **off `wave-(N-1)` when the previous wave is not merged yet**. Check before branching: `git log --oneline main..wave-(N-1)` — if it has commits, branch off the wave branch.
- **When your base moves, rebase onto it** rather than merging it in: `git rebase <base>`, resolve, force-push the branch with `--force-with-lease`. Rebasing keeps the pull request showing only your own commits, and keeps the history of a wave linear so that the wave's single pull request into `main` is readable.
- Tracks within a wave live on `wN/<track>` branches off `wave-N`, each in its own git worktree (see §5). Tracks merge back into `wave-N` through a pull request, rebasing on `wave-N` at least daily.
- **A wave finishes as one pull request** from `wave-N` into `main`, whose description lists what shipped, links the wave brief and the log entry, and names the release tag.
- The wave's contracts (shared types, schemas, IPC definitions) land on `wave-N` **before** track branches start, and are frozen for the wave. Changing a contract mid-wave means telling every track.
- Shared files (workspace `Cargo.toml`, the IPC command registry, generated TypeScript types, `CHANGELOG.md`) are edited only in integration commits on `wave-N`, and are kept append-only so merges stay textual.

## 4. Definition of done

A **track** is done when: the feature works, it has tests (§6), `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `pnpm lint` and `pnpm test` all pass, **the documents from §2 — the README and the tutorials included — describe what the software now does**, and the pull request into `wave-N` is green.

A **wave** is done when, on top of every track being done:

1. **It finishes as a pull request, and the project it leaves behind is clean and runnable.** Clean: no half-finished migration, no feature parked behind a flag nobody sets, no commented-out branch of code waiting for the next wave, no generated output or scratch file committed. Runnable: a person who clones the repository fresh and follows `CONTRIBUTING.md` gets a green `cargo test --workspace` and can run what the wave says it built — the CLI for Waves 0 and 0.5, the application from Wave 1 on — without knowing a trick. If a wave cannot end that way, it ends smaller: cut scope to the last point where everything works and move the rest to the next wave, writing down what moved. This applies to every wave, including preliminary ones that ship no installer.
2. The demo named in `docs/PLAN.md` for that wave can be performed end to end by a person who did not build it.
3. **Every capability the wave added can be learned from a tutorial** in `docs/tutorials/`, updated or written in this wave, and the tutorials still describe software that exists. A wave that shipped a feature and no way to learn it is not done.
4. From Wave 7 on: the MCP conformance suite is green in CI, and the eval scenarios — including the one this wave added — have been run by hand, with the result recorded in the wave log.
**While distribution is paused (Q-13), criteria 5 and 6 are not checked** and a wave is not tagged: waves are judged by what a person can do from a source checkout. The release workflow stays in place and must not be broken on purpose. When distribution resumes, both criteria apply again, starting with the first release.

5. Installers build on macOS, Windows and Linux in CI, signed where signing is configured.
6. **The previous release updates itself to the new one.** Install the previous version, run the updater, confirm it lands on the new build. Every wave ships through the updater; this is the one check that must never be skipped.
7. A project made by the previous version still opens, and a project made by this version opens in the previous version without losing data (unknown keys are preserved, §7).
8. `docs/WAVE-LOG.md`, `docs/OPEN-QUESTIONS.md` and `docs/FINDINGS.md` are updated, and the wave pull request is merged to `main` (and tagged, once distribution resumes).

## 5. Parallel work with worktrees

```bash
git worktree add ../booker-wt/w0-engine -b w0/engine wave-0
git worktree list
git worktree remove ../booker-wt/w0-engine     # after the merge
```

- Worktrees live in `../booker-wt/`, never inside the repository.
- Give each worktree its own `CARGO_TARGET_DIR` and use `sccache`; a shared target directory is locked by Cargo and serialises builds.
- Use `pnpm`, whose shared store keeps `node_modules` cheap per worktree.
- One agent per track, working only inside the paths its wave brief assigns to it. If you need to change a file another track owns, say so and hand it to the integration step instead of editing it.
- Keep a track under roughly a day of work between merges.

## 6. Testing and code expectations

- **Every feature ships with tests.** Rust: unit tests next to the code, integration tests per crate. UI: component tests for logic, not for pixels.
- **Golden tests are the backbone of layout work**: fixture project → rendered pages → PNG snapshot compared with a pixel tolerance. Each track owns its own fixture folder (snapshots are binary and merge badly). Regenerate deliberately, review the images before committing.
- Bug fixes start with a failing test.
- **Integration tests cover the agent surface, not just the core.** From Wave 7 on, every pull request runs the MCP conformance suite: a real client over stdio, the tool list checked against the capability manifest (they must not drift), every tool schema valid, each read tool's MCP result equal to the CLI's JSON for the same question, refusals for paths outside the project, clean errors instead of panics, and deterministic output — sorted keys, project-relative paths, no timestamps. It runs on all three platforms, because Windows differs in paths and stdio buffering.
- **Agent evals live in `evals/` in this repository and are never part of the shipped application** — they are a development harness, like the golden tests, run with `cargo xtask eval`. **By hand, at the end of a wave and before a release, never on a schedule** (each run costs model time and needs an API key). Each scenario is a fixture project, a prompt phrased as a person would phrase it, and a programmatic success check, so the assertion stays deterministic even though the agent is not. A scenario should pass in at least four runs out of five; the result goes in the wave log, and a drop in the pass rate is a reason not to release. The harness also records how many turns were used and which discovery commands (`capabilities`, `explain`, `recipe`) the agent reached for — that is how we find out where the vocabulary is failing people.
- **A wave that adds a user-facing capability adds at least one eval scenario for it**, phrased as a user request rather than as an instruction to use a feature. If a capability cannot be described as something a person would ask for, question whether it belongs.
- CI runs fmt, clippy with warnings denied, tests, the golden suite, the frontend lint and test steps, and the three installer builds.
- Crate boundaries are real: Typst types stay inside `booker-typst`; file writing stays inside `booker-project`; the UI talks to the core only through typed IPC commands generated with `ts-rs`. Do not widen these because it is convenient.
- No `unwrap()` or `panic!` on a path that a user's project content can reach — a broken project must produce a message in the problems panel, not a crash.
- Every user-visible error names the file and, where it applies, the line it came from.
- **Every feature ships its diagnostics with it.** If a feature can break a book in a way `booker check` cannot name and locate in the source, the feature is not finished. Rules get stable IDs (`BK-LAYOUT-003`), a severity, a source location, a layout location, and a fix when the fix is mechanical.
- Anything the app can show about a project, the CLI must be able to print. The agent surface is a thin layer over the core, never a second implementation.
- **A feature that adds vocabulary must make that vocabulary discoverable in the same change**: the JSON Schema (generated from the parsing structs, never hand-written), the capability manifest, an `explain` entry, and a recipe when the feature answers a common request. Nobody should have to guess a key name, and no agent should have to read our source to find one.
- **Unknown input suggests the right input.** An unrecognised key, style name, class or selector must produce "did you mean …" plus the `booker explain` topic. This is the loop that lets both people and agents recover from a wrong guess in one step.

## 7. Rules that protect users' projects

- The project format has a version in `book.toml`. Changing the format means bumping it and writing a migration, in the same change.
- **Unknown keys in project files are preserved and written back unchanged**, with a warning. An older Booker must never silently delete what a newer one wrote.
- Writes to a user's files are targeted: keep comments, key order and formatting (`toml_edit`), and touch only what changed. Opening a project must change nothing on disk.
- Layout attaches to content through stable IDs, never page numbers.
- The app ID `com.github.can3p.booker` must not change after the first public release; changing it breaks updates for everyone who installed it.

**Rules that keep the app safe to use alongside agents** (these are as binding as the ones above):

- Never hold an exclusive lock on a project file, and never keep state that exists only in memory. `.booker/` is a derived cache that must be safe to delete at any moment.
- All writes are atomic (temporary file, then rename) and targeted; the watcher ignores the echo of our own writes by comparing hashes.
- Save eagerly, so an outside edit rarely meets an unsaved buffer; when it does, offer both versions rather than discarding one.
- Handle bulk external change (an agent rewriting thirty files, a branch checkout) as one coalesced reload with one re-render, keeping the page position.
- A broken project must still open: load what loads, report the rest as diagnostics. Refusing to open is never right — that is exactly when the errors need to be visible.
- Write tools for agents exist only where the operation needs the layout engine in the loop (applying a diagnostic's patch, fit-and-iterate adjustments, canonical formatting). Anything expressible as a text edit stays a text edit — the project is plain text on purpose.

## 8. Commits and pull requests

- Conventional-style subjects (`feat(frames): …`, `fix(typst): …`, `docs: …`), imperative mood, and the wave/track in the body when it is not obvious.
- Commit only when work is at a sensible point; never commit generated caches, `build/`, or `.booker/`.
- End commit messages with:

  ```
  Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
  ```

- End pull request descriptions with:

  ```
  🤖 Generated with [Claude Code](https://claude.com/claude-code)
  ```

## 9. Working style

- Ask the owner only about things the owner alone can decide (accounts, money, naming, product direction). Everything else: pick the sensible option, write it down, keep moving.
- When you deviate from `docs/PLAN.md`, update the plan in the same change and note it in the wave log. The plan must always describe the software we intend to build; what was actually built, and how it differed, is the wave log's job (§2).
- Report honestly: if a test fails, say so with the output; if a step was skipped, say which.

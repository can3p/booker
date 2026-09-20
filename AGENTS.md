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

**Starting a session:** read `docs/WAVE-LOG.md` (what is finished), then `docs/PLAN.md` §8 (the waves), then `git branch -a` (what is in flight), then the brief for the current wave in `docs/waves/`. Say which wave and track you are on before you start changing files.

**Where things stand today:** Waves 0, 0.5 and 0.6 are complete and merged. The core works end to end — `booker new` then `booker build` writes a PDF — CI checks fmt, clippy, the test suite on macOS, Linux and Windows, the doc build, the TypeScript bindings and the dependency policy on every pull request, and `docs/tutorials/01-your-first-book.md` teaches everything Booker can currently do, with every command in it executed by `cargo test`. **Wave 1 is the next one to start**, and its brief is `docs/waves/wave-1.md`: the desktop application and the release pipeline. Rust lives in `~/.cargo/bin`, which is not on the default PATH — run `. "$HOME/.cargo/env"` first.

## 2. Which document gets the write

Put every kind of output in exactly one place:

- **A design decision changes** → edit `docs/PLAN.md` in place. Never keep a competing plan in a new file, never leave the plan describing something the code no longer does.
- **`docs/PLAN.md` holds future work only.** It says what is going to be built and why it is designed that way. Nothing retrospective goes in it: not what a wave shipped, not how it deviated, not what it measured, not what it cost. When a wave finishes, its section in §8 is replaced by a line pointing at `docs/WAVE-LOG.md` — never annotated with an account of what happened. A plan that carries its own history becomes a second, diverging one, and then two documents describe the same wave and a reader cannot tell which is true. The one exception is a decision taken in a finished wave that still constrains what comes next; that is design, and it belongs in the plan stated as a constraint, with the measurement behind it left in `docs/FINDINGS.md`.
- **Something needs the owner to decide** → append to `docs/OPEN-QUESTIONS.md` and keep going with a stated default. Do not block a wave on an unanswered question unless the question makes the work meaningless.
- **A wave finishes** → add its entry to `docs/WAVE-LOG.md` (what shipped, what deviated from the plan, the release tag and PR, what was deferred).
- **A hard-won fact** (a Typst behaviour, a Tauri quirk, a toolchain trap, a benchmark, a dead end and why) → append to `docs/FINDINGS.md`. The test: would a future session waste an hour without this?
- **A process rule changes** → edit this file, in the same change that introduces the rule.
- **A user-visible capability arrives or changes** → the tutorials must teach it, in the same change (`docs/requirements.md`). Extend the tutorial that covers the surrounding task when there is one, and add a new `docs/tutorials/NN-<name>.md` when the capability is a task of its own. Write it for someone making their first book: a real thing to make, every command or click, and what the screen actually says back. A feature nobody can be walked through is not finished, and a tutorial describing something the software no longer does is worse than none — if a change makes a tutorial wrong, fixing it is part of the change.
- **Anything a user can see or type changes** → update `README.md` in the same change. A new or renamed command, a changed flag or default, a new requirement to install, a different output path, a changed project layout, a capability that starts or stops working: the README must describe what the software does *today*, never what it will do. If a change makes a sentence in the README wrong, fixing that sentence is part of the change, not a follow-up.
- **The setup or the everyday commands change** → update `CONTRIBUTING.md` the same way.

Documentation is part of the work, not a chore after it. A pull request that changes behaviour and leaves the documents describing the old behaviour is incomplete, and so is one that documents something that does not exist yet. When in doubt, describe what you can demonstrate.

Keep every document in this table accurate at the end of every wave. A wave whose documents are stale is not finished.

**And `main` never holds a stale statement, not even briefly.** Some sentences go false on merge rather than on edit — "where things stand today", a `✅ done` marker, a wave log claiming something shipped, anything written in the future tense about work that has since landed. The change that merges carries their update with it; a pull request that would leave `main` describing a state of affairs that ended when it merged is incomplete, and the fix belongs in that pull request rather than in the next one. If staleness reaches `main` anyway, correct it in its own small pull request straight away — do not wait for unrelated work to carry it.

## 3. Branching, waves and pull requests

- **Never commit to `main`.** `main` only moves through a merged pull request.
- A wave lives on `wave-N`, which branches **off `main`**, or **off `wave-(N-1)` when the previous wave is not merged yet**. Check before branching: `git log --oneline main..wave-(N-1)` — if it has commits, branch off the wave branch.
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
5. Installers build on macOS, Windows and Linux in CI, signed where signing is configured.
6. **The previous release updates itself to the new one.** Install the previous version, run the updater, confirm it lands on the new build. Every wave ships through the updater; this is the one check that must never be skipped.
7. A project made by the previous version still opens, and a project made by this version opens in the previous version without losing data (unknown keys are preserved, §7).
8. `docs/WAVE-LOG.md`, `docs/OPEN-QUESTIONS.md` and `docs/FINDINGS.md` are updated, and the wave pull request is merged to `main` and tagged.

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
- When you deviate from `docs/PLAN.md`, update the plan in the same change and note it in the wave log. The plan must always describe the software that exists.
- Report honestly: if a test fails, say so with the output; if a step was skipped, say which.

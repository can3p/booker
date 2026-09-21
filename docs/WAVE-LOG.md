# Wave log

What each finished wave actually shipped, written at the end of the wave, newest first. The plan (`docs/PLAN.md` §8) says what is intended; this file says what happened. A wave is not finished until its entry is here.

Format:

```
## Wave N — <name>
- Shipped: YYYY-MM-DD as <tag> (PR #NN)
- Demo: what a person can now do, end to end
- Tracks: which landed, who (model tier) did them
- Deviations from the plan: what changed and why (and the plan was updated in the same change)
- Deferred: what moved to a later wave, with the wave it moved to
- Update check: previous release updated itself to this one — yes/no, on which platforms
- Agent surface (Wave 7 on): MCP conformance green in CI; eval run by hand — pass rate per scenario, and the new scenario this wave added
- Findings recorded: links into docs/FINDINGS.md
```

---

## Wave 1 — The application and how it reaches people
- Shipped: 2026-09-20, merged to `main` as PR #9. **Not yet released**: `v0.2.0` has not been tagged. The entry originally said it had been; it was written before the release step and merged before anyone ran it (see the deviation below).
- Demo: download an installer, open a folder that contains a book, read its chapters, edit one and watch the pages redraw, break `book.toml` from another editor and watch the problem appear, export a PDF. Then publish `v0.2.1` and watch the installed copy update itself.
- Tracks: contracts (lead) · B application shell · C preview and watcher · D hygiene · A release pipeline · D2 tutorial — run by one session, sequentially, in the order the brief's dependencies imply rather than in parallel worktrees. The wave was four tracks on paper and one line of work in practice: B had to exist before C could hang a preview inside it, and D2 describes the application that shipped.
- Deviations from the plan:
  - **The document-model-to-Typst translation moved out of `booker-cli` before any track started.** It was written there at Wave 0's integration, and the application cannot depend on the CLI. It is now `booker_typst::book`, behind `Engine::set_book`, where `PLAN.md` §6 always had codegen; both the window and the terminal go through it. The same treatment for the chapter list: `Project::chapter_summaries` produces what the sidebar shows *and* what `booker build` prints. Wave 2 track B now replaces one file rather than two.
  - **A minimal watcher landed here rather than in Wave 2.** Track C's done criterion — an edit on disk visible in the preview — needs one. Conflict handling (an outside edit meeting an unsaved buffer) stayed in Wave 2 track E, and the stress tests in Wave 7 track E. The plan was updated in the same change.
  - **`reload_project` was appended to the frozen contract**, in an integration commit: the event says *that* the folder changed, the command says what the book now is.
  - **The beta channel is half built.** A beta tag produces installers and a prerelease that `releases/latest` does not point at, so no installed copy is offered it. Staying on the channel needs a second manifest at a fixed URL and a setting in the application, which means guessing now at how people move between channels — including going back to stable from a beta whose version number is higher. Deferred with the reasoning in `docs/OPEN-QUESTIONS.md` Q-12.
  - **The release step never ran, and would have failed if it had.** The wave merged with its last exit criterion — tag `v0.2.0`, tag `v0.2.1`, watch one update to the other — still open, while this entry, `AGENTS.md`, `CHANGELOG.md`, `README.md` and the app tutorial already described `v0.2.0` as released. When the release was picked up the next day it turned out the step could not have succeeded: `tauri.conf.json` sets no version, so the bundles take the workspace's `0.1.0`, which nobody had bumped. Both tags would have produced builds calling themselves `0.1.0`, and the updater — which compares versions, not tag names — would never have offered one to the other. The fix sets the version to `0.2.0`, adds a job to `release.yml` that refuses a tag differing from the version before any build starts, and makes the `booker --version` line in tutorial 01 a checked one so the next bump cannot forget it. The documents were corrected to say the release is pending, in its own pull request, until it happens.
  - **The first tagged build then failed on three of four targets**, for two reasons the release workflow had never been run long enough to find: an unset Apple secret still defines `APPLE_CERTIFICATE`, which makes Tauri try to sign with nothing (both macOS targets), and the corepack workaround that works on Linux does not on Windows. Both are fixed in `release.yml` and recorded in `docs/FINDINGS.md`; the fix was proven with a manual dry-run build of all four targets before anything was tagged again.
  - **Signing is not set up.** There is no Apple Developer ID and no Windows certificate, so the first launch warns on both platforms. The release workflow already reads their secrets, so signed builds need certificates rather than another change. The *updater* is signed regardless, with a minisign key generated during the wave and stored as repository secrets.
- **What the tests found that nothing else would have.** Two bugs and three bad tests, all in the watcher:
  - One write produces several filesystem events for the same path, so asking "is this our own echo?" once per event marked the first as ours and every later one as an outside edit. Every save turned into a reload that fought the editor. Deduplicating the paths within a burst is what fixes it; echo suppression then stopped guessing altogether and now compares the file's content with what we wrote (`AGENTS.md` §7), which has no time window to get wrong and means an outside edit landing a moment after a save is reported rather than swallowed.
  - Every save reported itself anyway, through a path nothing could recognise: `write_atomic` creates its temporary file *inside* the watched folder and renames it away, so `content/.booker-XXXXXX.tmp` arrived as an outside change. The test that should have caught it wrote its chapters with `std::fs::write` instead of through the project's writer, so the hardest half of a save was never exercised. The name now belongs to `booker_project::is_write_temporary`, and the test performs a real save.
  - A test asserted that thirty files rewritten at once produce exactly one coalesced event. It passed on a laptop and failed on Linux and macOS runners, because whether thirty writes fit inside a 250 ms debounce window is a fact about the disk. The assertion was wrong, not the window; the tempting fix — lengthening the debounce until CI agrees — would have made the application slower for no reason.
  - Three tests asserted that nothing was reported, or that a particular event came first. Both are assertions about how fast the machine is, and the macOS runner collected on them twice: it hands a watch the files written just before the watch started, so a test that had written neither was shown `AGENTS.md` and the template's chapter. The tests now use a sentinel — a watch delivers in order, so a file written last and waited for proves everything before it has already arrived. Simulating the leak fails four of the five old tests and none of the new ones.
  - The tutorial harness refused a tutorial with no commands in it, which is every tutorial about a window. It now checks such a tutorial by building the files it tells the reader to create.
- Deferred: the beta channel (Q-12); conflict handling on an unsaved buffer (Wave 2 track E); `CHANGELOG.md` entries for anything before `v0.2.0`, which were never releases.
- Update check: **not yet done** — it needs the two releases above.
- Agent surface: not applicable — Wave 7.
- Findings recorded: nine entries in `docs/FINDINGS.md` — the Node and rolldown traps, what Tauri's two new licences actually require, generating an icon without an image editor, the multi-event echo bug, `tauri::test` as a way to test a custom protocol, why a debounce boundary must not be asserted, how the debouncer splits one write across batches, that an atomic write changes the folder it is written in, and that a macOS watch can report files written before it started.

---

## Wave 0.6 — The first tutorial, and the harness that keeps tutorials true
- Shipped: 2026-09-20, merged to `main` as PR #7 (no release tag: nothing installable yet).
- Demo: from a clean clone, `cargo install --path crates/booker-cli`, then follow `docs/tutorials/01-your-first-book.md` — `booker new the-moon-jar`, write two chapters, set a 5.5 × 8.5 in page, `booker build .` — and hold a two-page PDF of *The Moon in a Jar*. Then change `Problems: none` to `Problems: nothing` in `crates/booker-cli/src/lib.rs` and `cargo test` fails with `docs/tutorials/01-your-first-book.md:107: the tutorial says this line is printed: Problems: none / but `booker build .` printed: Problems: nothing`.
- Tracks: A the tutorial · B the harness · C where it is advertised — one session, sequentially, as the brief allowed. The wave was far too small for worktrees to pay for themselves, the same conclusion Wave 0.5 reached.
- Deviations from the plan:
  - **The harness spawns the binary instead of calling `booker_cli::run`.** The brief did not say which; `tests/commands.rs` does the latter and is much faster. It cannot work here: a tutorial quotes `Created a `novel` book in the-moon-jar` — a relative path, as a reader's shell prints it — and the exit code their shell reports. Running in process means rewriting the path arguments, which changes exactly the text under test. `env!("CARGO_BIN_EXE_booker")` with `current_dir` set costs a process spawn per command and the whole suite runs in under two seconds.
  - **The harness is deliberately not a shell.** It understands `booker …`, `cd`, `rm` and `echo $?`, and anything else fails the test telling the author to mark the block ```console ignore. Silently skipping an unrecognised line is precisely how a tutorial starts lying, so the brief's "runs each `$` line" became "runs each `$` line or refuses".
  - **`echo $?` became the way a tutorial declares a non-zero exit**, rather than a magic comment. It is what a reader would actually type, it teaches the exit codes the README documents, and a `booker` command that fails without the tutorial saying so now fails the test.
  - **The wildcard is `…` only, and the plan's "durations and paths" narrowed to durations.** Every path Booker prints is project-relative and stable (`display_path`, Wave 0.5), so wildcarding one would hide a regression rather than tolerate noise.
- **The harness earned its keep on its first three-platform run.** It went red on Windows only, at `docs/tutorials/01-your-first-book.md:56`: `booker new` listed `content\01-the-first-chapter.md` where `build`, three lines of output later, printed the same file with a forward slash. This is the Wave 0.5 bug class in the one command Wave 0.5 never checked — `new`'s output had no separator assertion on any platform. Fixed by routing `new` through `display_path` like everything else, with an assertion in `tests/commands.rs` that says why. The tutorials are now, incidentally, a cross-platform output contract for every command they walk through.
- Deferred: nothing from this wave. One small bug found and not fixed: `booker new x --template unknown` reports the location as `.` instead of naming anything relevant (`docs/FINDINGS.md`) — a self-contained fix for whoever next touches `Template::from_name`.
- What the harness could not catch: it passed on its first run while the tutorial still claimed the PDF's "paragraphs are indented after the first one". They are not. It was found by rendering both pages with `pdftoppm` and looking at them, which is now written down as the step a tutorial needs before a wave closes. The corrected paragraph describes what is actually on the page — justified lines, hyphenation that depends on `language`, and the `inside` margin swapping sides between page 1 and page 2.
- Cost: 11 commands replayed, 1.9 s of test time on top of the existing `test` job, no new CI job and no new minutes.
- Update check: not applicable — no installers in this wave.
- Agent surface: not applicable — Wave 7.
- Findings recorded: five entries in `docs/FINDINGS.md` — that a test proves a tutorial's commands but never its sentences, why the harness spawns the binary, how far build durations vary between runs, the TOML trap where an appended key lands in the last table, and the `--template` location bug.

---

## Wave 0.5 — Continuous integration
- Shipped: 2026-09-20, merged to `main` as PR #4 (no release tag: nothing installable yet). The planning that preceded it merged as PR #3.
- Demo: open a pull request that is unformatted, warns under clippy, fails a test or duplicates a dependency key; CI marks it red, one job per fault, each message naming the fault. Fixed, it goes green on macOS, Linux and Windows.
- Tracks: A workflow · B green off macOS · C dependency and repository guards — run by one session rather than in parallel worktrees; the wave was too small for the setup to pay for itself.
- Deviations from the plan:
  - **Track B was the whole wave.** The brief expected line endings and fonts to be the trouble; neither was (fonts are embedded with `include_bytes!`, and `.gitattributes` landed before the first Windows run). What Windows actually found was three separate faults, in three runs, because the first one hid the next: a path separator leaking into printed output, Unix-absolute path literals in tests that turn on `is_absolute`, and the same separator bug again in a font diagnostic built by a different crate.
  - **`display_path` ended up in `booker-core`, not `booker-project`.** It was written in `booker-project` and had to move one run later: `booker-typst` builds diagnostics too and must not depend on `booker-project`. What the crates that build diagnostics share belongs beside `SourceLocation`.
  - **`publish = false` on every workspace crate**, not in the brief: `cargo deny`'s wildcard check cannot tell a path dependency from a real `"*"` unless the crates say they are not published — and they are parts of an application, so saying it is true anyway.
  - **`--no-fail-fast` in the test job**, learned the expensive way: one failing test binary hid every later one, and a Windows run costs five minutes to surface one fact at a time.
  - The `cargo metadata` smoke check and the pre-push hook moved here from Wave 1 track D, as the brief said they would.
- Deferred: the frontend `pnpm lint` and `pnpm test` jobs (there is no `package.json` yet — Wave 1 track B adds one and the jobs with it); branch protection on `main`, which only the owner can enable (Q-10).
- Each check seen to fail on purpose: `fmt` (`Diff in crates/booker-doc/src/lib.rs:56`), `clippy` (`ptr_arg`, `-D clippy::ptr-arg implied by -D warnings`), `test` (`assertion left == right failed` at `crates/booker-doc/tests/parse.rs:47:5`, left 4, right 5) on a throwaway branch, closed as #5; and `deps` locally with the Wave 0 bug itself — a second `tempfile = "3"` in the workspace manifest makes `cargo metadata --locked` exit 101 with `Cargo.toml:47:1: duplicate key`.
- Cost, cold with no cache: fmt 5s, bindings 27s, deps 40s, clippy 1m15s, docs 1m20s, tests 2m10s on Linux, 3m46s on macOS, 5m06s on Windows. Warm, on the green run: 21s, 22s, 60s, 21s, 21s, 56s, 1m22s, 6m15s — Windows stays cold because `rust-cache` saves nothing from a failed job, so a platform that keeps failing keeps paying the full price.
- Update check: not applicable — no installers in this wave.
- Agent surface: not applicable — Wave 7.
- Findings recorded: four entries in `docs/FINDINGS.md` — what `cargo deny` needed before it was useful here, the two `quick-xml` advisories that cannot be updated away, forward-slashed paths in everything Booker prints, and `/…` not being an absolute path on Windows.

### What the first three-platform run was worth
The suite had never run off one macOS laptop. It found three real faults in a day-old codebase, two of them in the product rather than in the tests: `booker build` was telling Windows users about a file called `build\the-secret-garden.pdf` while the chapter list two lines above spelled the same kind of path with a forward slash, and a font warning did the same in the location an agent would read. Neither would have been noticed by anyone working on a Mac, and both are exactly the class of thing Wave 7 will have to assert about the MCP server. The cost of finding them was four runs and about forty minutes of runner time.

---

## Wave 0 — The core: engine, format, command line
- Shipped: 2026-09-19 on `main` (no release tag: there is no installer until Wave 1)
- Demo: `booker new my-book` then `booker build my-book` writes a real PDF. A 201-page novel lays out in 520–570 ms cold, 24–29 ms after a one-character edit.
- Tracks: contracts (lead) · C Typst engine · E format and CLI · bridge (lead, at integration). Tracks C and E ran in parallel in separate worktrees, each with its own owned paths.
- Deviations from the plan:
  - **The wave was split.** Tracks A (release pipeline), B (application shell) and F (hygiene) were independent of the core and became Wave 1, so the numbering of every later wave moved up by one. The plan was updated in the same change.
  - The Markdown parser decision was made on measured evidence rather than preference: `pulldown-cmark`, because its spans exclude block markers and are not optional.
  - A minimal document-model-to-Typst bridge was written at integration so the wave could end with a PDF. It is temporary and marked as such; Wave 2 track B replaces it.
- Deferred: `fixtures/novel-200p` as a checked-in fixture (the engine generates its large fixture in code instead); `#:schema` lines in generated project files (the schema itself is Wave 7).
- Update check: not applicable — no installers in this wave.
- Agent surface: not applicable — Wave 7.
- Findings recorded: 13 entries in `docs/FINDINGS.md`, including the incremental-compile numbers, Typst's global file-id cache, `toml_edit` discarding spans when a document becomes mutable, and ts-rs ignoring `#[serde(flatten)]`.

### What was learned about working in parallel
Two tracks, roughly 2,500 lines, integration in about ten minutes. Three conflicts, all anticipated by the brief: `docs/FINDINGS.md` (both appended — keep both sides), `Cargo.lock` (regenerate, never merge), and `Cargo.toml` — which **merged cleanly and then failed to parse**, because both tracks appended `tempfile = "3"`. Append-only prevents textual conflicts, not semantic ones; Wave 1 adds a `cargo metadata` check to CI for exactly this.

The frozen-contract rule held: two agents reported nine friction points between them and neither edited `booker-core`. Three of those were fixed by the wave lead at integration; the rest are open questions.

Writing the bridge at integration was what proved the two tracks fit together — and it immediately found a content-loss bug (tight list items came back with no text at all) that neither track's own tests could have caught.

# Wave 0.6 — The first tutorial, and the harness that keeps tutorials true

Branch: `wave-0.6` (off `main`, or off `wave-0.5` if that is not merged yet — check first, `AGENTS.md` §3) · Finishes as one pull request into `main` · No release tag: the first is Wave 1's `v0.2.0`

Read `AGENTS.md` first. This brief assigns the tracks, the paths each owns, and what done means.

## Why this wave exists

`docs/requirements.md` now asks for tutorials covering everything Booker can do, written for someone making their first book, and for every wave that adds a visible capability to update or add one in the same change. Booker can already do something — `booker new` then `booker build` turns a folder of Markdown into a PDF — and nothing teaches it. Starting the rule at Wave 1 would mean the application's tutorial is written on top of a gap.

The rule also has a failure mode worth designing against before there are ten tutorials rather than after: **a tutorial rots quietly.** Nothing fails when a printed line changes or a flag is renamed; a reader finds out instead. So this wave writes the first tutorial *and* the thing that checks it — which is why it comes after Wave 0.5, with CI already running.

**Demo at the end of the wave:** someone who has never seen Booker follows `docs/tutorials/01-your-first-book.md` and ends up with a PDF of their own book. Then change a line of the CLI's output on a branch, and CI goes red naming the tutorial and the line that is now a lie.

## What exists already, and is therefore what the tutorial covers

Everything below ships today. Nothing else may appear in a tutorial in this wave.

- `booker new <path>`, with `--template novel` (the only template) and `--title`; the title otherwise comes from the folder name.
- What `new` generates: `book.toml`, `content/01-the-first-chapter.md`, `assets/images/`, and an `AGENTS.md` that explains the folder to whoever opens it next.
- `book.toml`: `format`, `title`, `author`, `language`, `chapters`, `[page]` with `size` (`a4`, `a5`, `letter`, `trade`, `digest`, `square`, or a measurement like `"5.5x8.5in"`) and `facing`, and `[page.margins]` with `top`, `bottom`, `inside`, `outside`. Comments, key order and unknown keys survive being written back.
- Chapters in Markdown: headings, paragraphs, emphasis, lists, quotes, links, images, code blocks. The `chapters` list sets the order; delete it and `content/*.md` is read in file-name order.
- `booker build [project]`: the summary it prints — title, format, language, page size and margins, the per-chapter line with word, heading and image counts, `Problems:`, and `Built build/<name>.pdf — N pages in N ms`.
- Diagnostics a beginner will actually hit: `BK-FORMAT-001` (no `book.toml`), `BK-FORMAT-002` (not valid TOML), `BK-FORMAT-004` (a value that cannot be understood), `BK-FORMAT-005` (an unknown key, kept), `BK-REF-002` (a chapter listed but absent), `BK-REF-003` (no content at all). Each names the file and the line.
- The project is a git folder that other tools — an editor, an agent — can rewrite while you work.

**Not in a tutorial this wave**, because it does not exist: the application, the preview, styles, page rules, frames, anchored images, `booker check`, HTML or EPUB. Each arrives with its own wave and its own tutorial.

## Where tutorials live

`docs/tutorials/`, one Markdown file per task, numbered so the order is a reading order: `01-your-first-book.md`, and an `index.md` that says what each one teaches and what it assumes. Plain Markdown in the repository, reviewed in the same pull request as the code it describes — when there is a documentation site, it renders these files rather than replacing them.

## Tracks

Small wave; one session can run all three. Each track works in its own worktree (`../booker-wt/w06-<track>`) on branch `w06/<track>` if they are split.

### A. The tutorial — tier M
**Owns:** `docs/tutorials/01-your-first-book.md`, `docs/tutorials/index.md`.

One tutorial that takes a person from nothing to a PDF of a short book they wrote: install and build the CLI, `booker new`, look at what it made, write two chapters, set the page size and margins, build, open the PDF. Then two things that go wrong on purpose — a typo in `book.toml` and a chapter listed but not there — so the reader learns what a diagnostic looks like and that a broken book still opens.

Rules for the writing, and they are the point of the wave:

- **A real book, named and specific.** Not "your project" — a short book with two chapters that the reader actually types.
- **Every command, and what it prints back.** Output blocks are copied from a real run, not paraphrased.
- **No typesetting vocabulary without a sentence explaining it**, and no reference to a wave, a crate, a rule ID's internals or anything in `docs/PLAN.md`. The audience is an amateur making a kids book or a novel.
- **Say what to do when it goes wrong**, at each step where it plausibly does.
- Ends by pointing at what Booker cannot do yet, honestly, so nobody hunts for a feature that is two waves away.

**Done when:** a person who has not seen Booker can follow it start to finish and hold a PDF, and every command block in it is covered by track B.

### B. Tutorials that check themselves — tier M
**Owns:** `crates/booker-cli/tests/tutorials.rs` (or `xtask` if it grows past one file), and the fenced-block convention documented in `docs/tutorials/index.md`.

A test that reads every file in `docs/tutorials/`, extracts the fenced ```console blocks, runs each `$` line in a temporary project folder, and compares what came back with the lines the tutorial claims. A line may contain `…` to mean "anything here", which is how a duration, a path or a version stays out of the comparison. Blocks that are file content rather than commands are marked as such and are written to the path the tutorial names, so the reader's `book.toml` and the one the test builds are the same text.

It runs inside the existing `test` job — no new CI job, no new minutes beyond the seconds it takes.

**Done when:** changing a printed line in `booker-cli` makes `cargo test` fail naming the tutorial file and the line that no longer matches, and the tutorial passing means a reader following it sees what it says.

### C. Where tutorials are advertised — tier S
**Owns:** `README.md`, `CONTRIBUTING.md`.

`README.md` links the tutorial as the way in, above the command reference — someone arriving at the repository should reach a finished book, not a flag list. `CONTRIBUTING.md` says how to run the tutorial check locally and how to add a tutorial: the fenced-block convention, the `…` wildcard, and the rule that output is copied from a real run.

**Done when:** the README's first instruction to a new user is the tutorial, and `CONTRIBUTING.md` tells a contributor how to keep one green.

## Order of work

1. Write the tutorial first, by actually doing it — install from a clean checkout, `booker new`, write the chapters, build. Anything that is awkward to explain is a finding, and possibly a bug: record it in `docs/FINDINGS.md` rather than writing around it.
2. Then the harness, against the tutorial that already exists.
3. `README.md` and `CONTRIBUTING.md`.
4. `docs/WAVE-LOG.md` entry, `docs/OPEN-QUESTIONS.md`, pull request into `main`.

## Exit criteria

- `docs/tutorials/01-your-first-book.md` takes a beginner from nothing to their own PDF, and every command block in it is executed by `cargo test`.
- A deliberate change to the CLI's output turns the suite red, naming the tutorial and the line.
- `README.md` points at the tutorial; `CONTRIBUTING.md` says how to add and check one.
- `AGENTS.md` §4's new criterion — every capability a wave adds can be learned from a tutorial — is true of everything Booker can do today.

## Which of the `AGENTS.md` §4 wave criteria apply

Criteria 5, 6 and 7 (installers, the updater, cross-version projects) do not apply: there is still no application and no release, as in Waves 0 and 0.5. Criterion 4 (the MCP suite and evals) starts at Wave 7. Criteria 1, 2, 3 and 8 apply, and **criterion 3 is what this wave exists to make true for the first time**.

## What this wave changes for every later wave

From here on a wave brief has a tutorial line in it, owned by a named track, and the wave is not done until a reader can learn the new capability from `docs/tutorials/`. Wave 1 inherits the first instance of that: the application arrives, and with it a tutorial that opens a folder, shows the pages and exports a PDF from the window rather than the terminal.

# Wave 2 — Write a simple book

Branch: `wave-2` (off `docs/distribution-paused` while that is unmerged, rebased onto `main` once it lands — `AGENTS.md` §3) · Finishes as one pull request into `main`. **No release tag**: distribution is paused (`docs/OPEN-QUESTIONS.md` Q-13), so `AGENTS.md` §4 criteria 5 and 6 are not checked this wave.

Read `AGENTS.md` first. This brief assigns the tracks, the paths each owns, the contracts they share, and what done means.

## Goal

**Demo at the end of the wave** (`docs/PLAN.md` §8): from a source checkout, create a novel from a template, write chapters in the window — or edit them in VS Code and watch the preview follow — get a table of contents, and export a PDF that looks like a book. Click a paragraph in the preview and the editor jumps to it; move the cursor and the preview follows. Edit a chapter in another editor while it has unsaved changes in the window, and be offered both versions. Then `booker check` the book from the terminal and see the same problems the window shows.

Wave 0 made a PDF; Wave 1 put a window around it. What neither did is make the PDF look like a book. Today every book is Libertinus at 11 pt with no table of contents, no page numbers, straight quotes and hyphens where dashes belong, and chapters that simply start on the next page. This wave is where the defaults become the ones a book should have.

## What exists already

- **The document model** (`crates/booker-doc`): headings, paragraphs, emphasis, lists, quotes, code, images, links, thematic breaks, raw HTML, each node with its byte span. Attributes are read only on headings (`{#id .class}`); `Attributes` exists on every node that will carry them.
- **The one translation to Typst** (`crates/booker-typst/src/book.rs`): marked temporary since Wave 0 and replaced by this wave. It escapes `'`, `"` and `-` in text, which is why a Booker book has straight quotes and no dashes: Typst turns `"`, `'`, `--` and `---` into typographic quotes and dashes by itself, and escaping them switches that off.
- **The project** (`crates/booker-project`): `book.toml` round-trips with comments and unknown keys kept, "did you mean" on unknown keys (`BK-FORMAT-005`), atomic writes, `ConfigEditor` for targeted edits, one template (`novel`), eleven diagnostic rules (`rules.rs`).
- **The application** (`app/`): a plain `<textarea>` editor saving 400 ms after the last keystroke, a sidebar that lists chapters, a preview that renders visible pages over `booker://` with a zoom, a problems panel, and a watcher that reloads on an outside edit — **discarding any unsaved text in the editor** (`book.svelte.ts`, `reloadFromDisk`). That last one is this wave's to fix.
- **Two commands**, `booker new` and `booker build`. `xtask golden` is a stub that says "golden snapshots arrive with Wave 2 track G".
- **Fonts**: only Typst's defaults, Libertinus Serif and DejaVu Sans Mono (`crates/booker-typst/fonts/`).

## Contracts — landed on `wave-2` before the tracks, then frozen

A track that needs a change to any of this says so rather than editing it (`AGENTS.md` §3).

### 1. `book.toml`: three new tables and one key

```toml
theme = "novel"             # novel | picture-book | poetry | paper — built in; default "novel"

[toc]
enabled = true              # default: true when the book has more than one chapter
depth = 1                   # heading levels listed, 1–3; default 1
title = "Contents"          # default depends on `language`

[chapter]
start = "right-page"        # new-page | right-page | continue; default from the theme
```

- All optional, all with defaults, so **a book that sets none of them is valid and looks right** (`AGENTS.md`, the audience rule). No format bump: an older Booker keeps the keys it does not know and warns (`AGENTS.md` §7), which is `AGENTS.md` §4 criterion 7.
- **A theme is a built-in bundle of typographic defaults** — body face and size, leading, paragraph indent or spacing, heading faces and sizes, where page numbers go, whether chapters start on the right. It is the `theme` key `docs/PLAN.md` §5.2 already reserves. Wave 3 makes every value in it overridable from `styles.toml`; this wave only lets a book choose one. Themes live in `booker-typst` as data, one per template.
- Unknown values are diagnostics with "did you mean", not parse failures: `theme = "novle"` → `BK-FORMAT-006` naming the four themes; `start = "odd"` → `BK-FORMAT-004` with the three allowed values.
- `BookConfig` gains `theme: Option<String>`, `toc: TocConfig`, `chapter: ChapterConfig`, each keeping its own `extra` map, exported to TypeScript like the rest.

### 2. The Markdown Booker understands

Pandoc-compatible, as `docs/PLAN.md` §5.3 says. Added this wave:

| Written | Means | Model |
|---|---|---|
| `{#id .class key=value}` after a heading, an image, a `[bracketed span]`, or on a `:::` fence | attributes | `Attributes` on the node — no longer headings only |
| `::: {.class key=value}` … `:::` | a block with attributes around other blocks | `Block::Div` |
| `[some text]{.class}` | an inline with attributes | `Inline::Span` |
| `break-before=page` on a heading or a div | start a new page here | an attribute, read by codegen |
| `::: page-break` / `:::` (an empty div of class `page-break`) | a page break on its own | `Block::Div` with that class |
| `---` between paragraphs | a scene break, drawn as a centred `⁂`, not a rule | `Block::ThematicBreak`, unchanged |
| `~~struck~~` | strikethrough | `Inline::Strikethrough` |
| GFM tables | a plain table | `Block::Table` |

Every new node carries its byte span, and an attribute block carries its own span so a diagnostic can point at the `{…}` rather than the paragraph.

Unknown attribute keys and classes are **kept** and warned about (`BK-DOC-001`, with "did you mean" against the keys codegen reads). Classes with no meaning yet are not an error — Wave 3 gives them styles — so an unknown *class* is silent this wave; an unknown *key* is not.

### 3. From a page back to the source, and from the source to a page

Codegen returns a **span map** alongside the Typst text: for each piece of generated source, the chapter it came from and the Markdown byte span. The engine combines it with `typst-ide` 0.15.1 (`jump_from_click`, `jump_from_cursor`), which maps between positions on a page and positions in the generated source. `typst-ide` is a Typst crate and stays inside `booker-typst` (`AGENTS.md` §6).

`booker-core` gains, exported to TypeScript:

```rust
pub struct PagePoint { pub page: u32, pub x: Length, pub y: Length }      // 0-based page
```

and two IPC commands, appended in the contracts commit: `source_at(PagePoint) -> Option<SourceLocation>` (click to source) and `pages_at(SourceLocation) -> Vec<PagePoint>` (cursor to page). The source side is the `SourceLocation` diagnostics already use — file, line, column and byte span — rather than a second type saying the same thing.

**Both have CLI equivalents in this wave**, because anything the window can say about a book the terminal must be able to print (`booker-core/src/ipc.rs`, rule 2): `booker where content/01-the-garden.md:12` prints the page (or pages) that line landed on, and `booker page 3` prints the source lines on page 3. They are the first half of the `where` and `page` commands `docs/PLAN.md` §11.3 gives Wave 7; Wave 7 adds phrase lookup and JSON, and must keep these forms working.

### 4. Saving when the file changed underneath

`save_chapter` already receives the revision the text was read at. It now returns

```rust
#[serde(tag = "outcome", rename_all = "kebab-case")]   // {"outcome": "saved", "info": …}
pub enum SaveOutcome {
    Saved { info: Box<ProjectInfo> },
    /// The file on disk is not what the editor last read. Nothing was written.
    Conflict { disk: ChapterText, mine: String },
}
```

and never overwrites a file whose content differs from what the editor loaded. The decision is by content, not by revision number: the revision moves for *any* change to the folder, and a change to another chapter is not a conflict with this one. The store keeps what the editor loaded as its base.

### 5. `booker check`

`booker check <path>` loads the project, lays it out without writing anything, and prints every diagnostic in the format `booker build` already uses, then `Problems: none` or a count. Same exit codes as `build` (0 clean, 1 errors, 2 could not run). `--format json` prints the diagnostics as the JSON `booker-core::Diagnostic` already serialises to, one array, sorted by file and line, with project-relative paths — the shape Wave 7's `check` extends, not a second one.

### 6. New diagnostic rules

Appended to the rule tables in `crates/booker-project/src/rules.rs` and `crates/booker-typst/src/diagnostics.rs`. IDs are stable from here on.

| ID | Severity | Raised when | Owner |
|---|---|---|---|
| `BK-FORMAT-006` | error | `theme` names no built-in theme (did you mean …) | E |
| `BK-REF-005` | error | a link to `#id` where no heading, div or span has that id | G |
| `BK-REF-006` | error | two elements share an `id` | G |
| `BK-DOC-001` | warning | an attribute key codegen does not read (did you mean …) | A |
| `BK-DOC-002` | warning | Markdown Booker keeps but cannot lay out yet (footnotes, math, raw HTML) — today this is silent | A |
| `BK-TEXT-001` | warning | a chapter file renders nothing | G |

Each carries a source location — file, line, column — and a fix where the fix is mechanical (the "did you mean" ones).

## Tracks

**One session runs this wave, sequentially**, as Waves 0.5, 0.6 and 1 did. The tracks below are still the unit of work — each lands as its own commits on `wave-2`, and owns the paths listed, so a future session could split them into worktrees without redrawing the lines. The order under "Order of work" is the dependency order.

### A. Document model — tier L
**Owns:** `crates/booker-doc/**`.
Everything in contract 2: the attribute pass over images, bracketed spans and fenced divs (on top of `pulldown-cmark`'s events, keeping their spans — `docs/FINDINGS.md`, "Markdown parser"), `Block::Div`, `Inline::Span`, `Inline::Strikethrough`, `Block::Table`, `BK-DOC-001` and `BK-DOC-002`.
**Done when:** every row of contract 2 parses to the model with correct spans (`tests/spans.rs` asserts each one slices back to its source), and a Markdown file with every construct in it parses without losing a byte of text.

### B. Codegen and typography — tier L
**Owns:** `crates/booker-typst/src/book.rs` (replaced), `crates/booker-typst/src/themes/**` (new), the span map and `typst-ide` use in `crates/booker-typst/src/engine.rs`.
The real translation: themes, `chapter.start` (`pagebreak(to: "odd")` for `right-page`, with the blank verso left truly blank — no page number), the table of contents (`outline`, its title in the book's language), page numbers in the footer from the theme, `break-before`, page-break divs, scene breaks, tables, strikethrough. **Typographic defaults, on without asking** (`docs/PLAN.md` §6, "Good typography without asking"): justified text, hyphenation in the book's `language`, Typst's widow and orphan costs, smart quotes and dashes (stop escaping `"`, `'` and `-`; escape only what would otherwise be markup), ligatures and kerning left on, first-line indent after the first paragraph of a chapter for prose themes, spacing between paragraphs for the others. Then the span map and `source_at` / `pages_at`.
**Done when:** each theme renders the same fixture book and the pages are reviewed as images (`AGENTS.md` §6: regenerate deliberately, look before committing); a quote and a dash come out typographic; `right-page` puts every chapter on an odd page; a click at a paragraph's position on a rendered page maps back to that paragraph's line.

### C. Editor — tier M
**Owns:** `app/src/lib/panes/Editor.svelte`, `app/src/lib/editor/**` (new), `app/src/lib/panes/Sidebar.svelte`.
CodeMirror 6 in place of the `<textarea>`: Markdown highlighting with headings, emphasis and strong shown styled, and the syntax markers dimmed — not hidden: hiding them moves the text under the cursor, which is worse for an amateur than seeing a `#`. The autosave policy stays in the store, unchanged. **The chapter tree**: add, rename, reorder and remove chapters from the sidebar, each a targeted edit of `book.toml`'s `chapters` list through `ConfigEditor` plus the file operation, through new commands `add_chapter`, `rename_chapter`, `move_chapter`, `remove_chapter` (removal moves the file to the system trash rather than deleting it). These are writes, not questions about a book, so they need no CLI twin — an agent edits `book.toml` and renames a file, which is what the project being plain text is for (`AGENTS.md` §7, last rule).
**Done when:** typing in a 10,000-word chapter stays smooth, a chapter added in the sidebar appears in `book.toml` in the place it was dropped with the file's comments intact, and component tests cover the tree's operations.

### D. Preview — tier M
**Owns:** `app/src/lib/panes/Preview.svelte`, `app/src/lib/preview/**`, `app/src-tauri/src/protocol.rs`.
Click-to-source and cursor-to-page on top of contract 3: a click on a page selects that place in the editor, opening the chapter if it is not the open one; moving the cursor scrolls the preview to the page it is on, and marks the spot. Zoom controls that stay put (fit width, fit page, 100 %), and the page position kept across a re-render (it already survives a reload; it must survive a zoom).
**Done when:** the round trip works on the demo book, and a test of the store covers "click on page 3 opens chapter 2 at line 14" against a mocked command.

### E. Project I/O — tier M
**Owns:** `crates/booker-project/src/{config,edit,lib}.rs` for the new keys, `app/src/lib/book.svelte.ts`, `app/src/lib/panes/Conflict.svelte` (new), `app/src-tauri/src/{commands,session}.rs`.
Loading and validating contract 1's keys (`BK-FORMAT-006`, "did you mean" for keys inside the new tables). Contract 4, end to end: the session refuses to overwrite a file that changed, the store keeps its base text, an outside edit to the open chapter while it is unsaved **no longer discards the buffer** and shows both versions side by side with "keep mine", "take theirs" and "keep both" (the second version saved as a new chapter file next to the first). An outside edit to a chapter with no unsaved changes still simply reloads.
**Done when:** the conflict path is covered by a session test and a store test, and no path through the store can drop unsaved text without the person choosing to.

### F. Templates and fonts — tier S (content) / M (wiring)
**Owns:** `crates/booker-project/src/template.rs` and `crates/booker-project/templates/**` (new), `crates/booker-typst/fonts/**`, `THIRD-PARTY.md` (regenerated, never hand-edited).
Four templates for `booker new --template`: `novel` (5.5 × 8.5 in, chapters on the right, Literata), `picture-book` (8.5 × 8.5 in, large friendly type in Andika, no table of contents, flowing text with pictures between paragraphs — frames are Wave 5), `poetry` (A5, EB Garamond, one poem per page via `break-before`), `paper` (A4, Source Serif 4 with Inter headings, sections continue rather than break). Each has a sample text that shows off what it is for, written for a first-time author. Bundle exactly the faces the four themes use, with their licences in `fonts/NOTICE.txt` — this answers the Wave 2 half of `docs/OPEN-QUESTIONS.md` Q-02 and moves the rest (Inter as a UI face, Atkinson Hyperlegible, Cinzel Decorative) to the waves that need them. Mind the binary size: the CLI embeds fonts with `include_bytes!`, so record what the four families add.
**Done when:** `booker new x --template <each>` then `booker build x` gives a clean, good-looking PDF for all four, reviewed as images, and `booker new x --template unknown` names the four and suggests the nearest (this also fixes the `.` location bug in `docs/FINDINGS.md`).

### G. Tests and the command line — tier S (fixtures) / M (harness)
**Owns:** `crates/booker-cli/**`, `xtask/src/golden.rs` (new), `fixtures/golden/**` (new), and the rules marked G in contract 6.
**Golden tests**: a fixture project per theme under `fixtures/golden/<theme>/`, rendered to PNG at a fixed scale and compared with committed snapshots under a per-pixel tolerance; `cargo xtask golden` regenerates them for review; the comparison runs in `cargo test`. Snapshots are rendered at a modest scale so the repository does not grow by megabytes a wave, and the tolerance absorbs font rasterisation differences between the three platforms — measure that on CI before settling the number, and write it in `docs/FINDINGS.md`. `booker check` (contract 5), `booker where` and `booker page` (contract 3).
**Done when:** changing a margin in a theme fails the golden suite with an image diff that shows where, `booker check fixtures/broken` names every fault in that fixture with file and line, and all three commands appear in the tutorials.

### H. Tutorials — tier S
**Owns:** `docs/tutorials/**`, and the README's command table.
Extend `01-your-first-book.md` for the terminal: choosing a template, what the table of contents and `[chapter] start` do, `booker check`, and `booker where`. Extend `02-the-app.md` for the window: the new editor, the chapter tree, clicking between page and text, and what happens when two editors change the same chapter. **Add `03-writing-in-markdown.md`**: the Markdown a book needs — headings and chapters, emphasis, scene breaks, page breaks, attributes, what quotes and dashes turn into — written for someone who has never heard the word Markdown. Every page-level claim is checked by rendering the pages and looking at them, not only by the harness (`docs/FINDINGS.md`, "A test can prove a tutorial's commands, never its sentences").
**Done when:** `AGENTS.md` §4 criterion 3 is true of everything above.

## Order of work

1. ~~**Contracts** (lead): the `booker-core` types and IPC names in contracts 1, 3 and 4, the rule constants, `typst-ide` in the workspace, this brief.~~ Done and frozen. `app/src-tauri/src/commands.rs` lists the six commands not yet implemented, with the track that owns each.
2. **A**, then **B** — codegen reads the new model — then **F**, whose templates need B's themes.
3. **G**, once B's output is worth freezing into snapshots.
4. **E**, then **C** and **D** — the conflict handling changes the store that the editor and preview both sit on.
5. **H** throughout: each track's user-visible part lands with its tutorial paragraph, and the last pass renders every tutorial's claims and reads them.
6. Integration: the full suite on all three platforms, the demo performed from a fresh clone, `docs/WAVE-LOG.md`, `docs/FINDINGS.md`, `docs/OPEN-QUESTIONS.md`, `AGENTS.md` "where things stand", and the pull request into `main`.

## Exit criteria

- ⬜ The demo at the top, performed end to end from a fresh clone following `CONTRIBUTING.md`.
- ⬜ All four templates build clean and look right, reviewed as images; the golden suite guards them on all three platforms.
- ⬜ `booker check`, `booker where` and `booker page` print what the window shows.
- ⬜ No path through the window loses unsaved text.
- ⬜ A project made by this wave opens in the Wave 1 build with its new keys preserved (`AGENTS.md` §4 criterion 7), and a Wave 1 project opens here unchanged.
- ⬜ Tutorials 01, 02 and 03 teach all of it.
- ⬜ `docs/WAVE-LOG.md` entry written; every document in `AGENTS.md` §2 accurate.

## Not in this wave

`styles.toml` and per-style typography knobs (Wave 3 — themes are its starting values); running heads, page rules and roman-numbered front matter (Wave 4); frames, and pictures placed anywhere but between paragraphs (Waves 5 and 6); footnotes and cross-references beyond `#id` links (Wave 8); JSON Schema, `capabilities`, `explain` and recipes for the vocabulary added here (Wave 7, which must backfill it); releases (Q-13).

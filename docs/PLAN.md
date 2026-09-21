# Booker — Architecture & Development Plan

Status: proposal, revision 3 · 2026-09-19 · input: `docs/requirements.md`

Companion documents: `AGENTS.md` (how sessions work in this repository), `docs/OPEN-QUESTIONS.md` (undecided), `docs/WAVE-LOG.md` (what each finished wave shipped), `docs/FINDINGS.md` (lessons worth carrying forward), `docs/waves/wave-N.md` (per-wave briefs).

---

## 1. Summary of decisions

| Topic | Decision | One-line reason |
|---|---|---|
| Layout / typesetting engine | **Typst** (Rust crate, Apache-2.0), embedded in-process | Fast incremental compiles, same output on every OS, strong PDF output, lets us query the layout for multi-pass placement |
| Core language | **Rust** (one Cargo workspace) | Typst is Rust; the app is fast and has no runtime to ship |
| Desktop shell | **Tauri 2** | Installers of about 10–15 MB, a signed updater built in, bundles for all 3 OSes |
| UI | **Svelte 5 + TypeScript**, **CodeMirror 6** editor | Small runtime; CodeMirror handles input methods (IME), spell-check, and large documents |
| Text source | **Markdown files** (CommonMark + GFM) with a small attribute syntax | Users can keep editing in Obsidian, VS Code, iA Writer, or on GitHub |
| Page model | **Two modes in one project: flowing text and composed pages** (frames placed freely or on a grid, dragged in the preview) | A novel should flow; a kids book needs the text and the picture exactly where the author puts them |
| Styles / pages / layout | **TOML** files, written back with `toml_edit` (keeps comments and order) | Readable, produces clean diffs, and the GUI can rewrite them without mangling hand edits |
| Project | **A plain folder that works as a git repo** (not a single file) | Required for version control; each part can be edited with outside tools |
| PDF output | Typst PDF plus our own bleed and crop marks | Print-ready |
| HTML / EPUB output | **Our own renderer** from the same document model and styles | Typst's HTML export is still experimental and ignores paging |
| Updates | `tauri-plugin-updater` with signed artifacts; `latest.json` hosted on GitHub Releases; stable and beta channels | Works from the very first release |
| Headless build | `booker` CLI plus a GitHub Action template | "Push to GitHub → PDF is built" |
| License / repo | **MIT**, `github.com/can3p/booker`; app ID `com.github.can3p.booker` for now | Decided; no domain needed to ship |
| Agents as users | **The CLI and an MCP server share the app's core**; diagnostics with stable rule IDs; the project survives being rewritten from outside while the app is open | People will point Claude Code at their book folder; the app must help rather than fight (§11) |
| Version control in the app | **Out of scope for 1.0** — the format is git-friendly, users bring their own git client | Decided; keeps the app small |

---

## 2. How others do it (and what we take from them)

| Tool | Model | What we take / avoid |
|---|---|---|
| LaTeX | Source markup → batch compile | Take: plain text in git, reproducible builds. Avoid: steep learning curve, slow compiles, huge install |
| InDesign / Affinity Publisher | Frame-based DTP, master pages, text threading | Take: **master pages / page templates**, paragraph and character styles, anchored objects. Avoid: manual frame work on every page, binary files |
| Scribus | Frame-based DTP, open source | Avoid: clunky frame workflow, manual reflow |
| Vellum / Atticus | Pick a theme, write chapters, export | **Closest match to our audience.** Take: zero-config themes, chapter list, live book preview, "starts on a right-hand page" toggles. Avoid: closed formats, very limited layout control |
| Pandoc / Quarto / Bookdown | Markdown → LaTeX/HTML | Take: Markdown attribute syntax (`{#id .class}`), fenced divs (`:::`), "one source, many outputs" |
| Paged.js / Vivliostyle / WeasyPrint | HTML + CSS Paged Media → PDF | Take: page selectors (`:left`, `:first`, named pages), CSS-like style model. Avoid: output that depends on the browser engine |
| Typst | Modern markup plus an incremental compiler | Use it as the engine, but hide its syntax from users |

What sets Booker apart: Vellum-level simplicity, InDesign-style master pages and anchored images driven by **rules instead of manual work**, and a git-friendly project that stays readable without Booker.

---

## 3. Choosing the engine

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **Typst (embedded)** | Rust library; incremental compilation (memoized), so a small edit recompiles fast; same result on every OS; PDF/A and PDF/UA; variable fonts (0.15); layout can be queried (`locate` / `query`), which enables a multi-pass placement solver; package ecosystem (`meander` wraps text around images, drop-cap packages) | Pre-1.0, so there will be breaking changes; no built-in text wrapping around images; HTML export is experimental; CMYK colours exist but there is no ICC profile, output intent or PDF/X support (verified 2026-09; see §10) | **Chosen.** We pin the version and keep all Typst code behind one crate |
| CSS Paged Media in the webview (Paged.js / Vivliostyle) | HTML inserts "just work"; styles are CSS | Tauri uses a different browser engine per OS (WebKit on macOS and Linux, Chromium on Windows), so **pagination would differ per OS** unless we bundle Chromium (bloat); slow on 300-page books; weak print features | Rejected as the primary engine |
| WeasyPrint | Good CSS support | Needs a bundled Python runtime; slow; no page floats | Rejected |
| LaTeX (Tectonic) | Top-quality typesetting | Large, slow, and impossible to debug for amateurs | Rejected |
| Our own engine | Full control | Years of work | Rejected |

**Rule:** users never *have to* see Typst. Advanced users can add `extensions/*.typ` (see §5.7), which puts the extra complexity in the user's project, as the requirements ask.

## 4. Choosing the toolkit and language

| Option | Size / speed | Rich text editor | Updater and packaging | Verdict |
|---|---|---|---|---|
| **Tauri 2 + Rust + Svelte** | ~10–15 MB, native speed in the core | CodeMirror 6 (mature) | Official signed updater; builds DMG, MSI/NSIS, AppImage, deb and rpm | **Chosen** |
| Electron | 100 MB+, heavy on memory | Same editors | Good | Rejected: too bloated |
| Native Rust UI (Slint, iced, egui, GPUI) | Smallest | Would need a text editor with IME, spell-check and accessibility built from scratch | We'd build it ourselves | Rejected for now; revisit after 1.0 |
| Qt (C++ or PySide) | Medium to large | Good | Our own updater; LGPL constraints | Rejected: slower to develop, heavier |
| Flutter | Medium | Weak | OK | Rejected: no typesetting engine in Dart |

Known weak spot: WebKitGTK on Linux is slower. This only affects the UI, not the output. We reduce the impact by rendering preview pages in Rust (`typst-render` for PNG tiles, `typst-svg`) and serving them over a custom `booker://` protocol, rather than sending base64 over IPC.

---

## 5. Project format (a git repo)

### 5.1 Layout

```
my-book/
├── book.toml              # metadata, chapter order, page setup, output presets, format version
├── content/
│   ├── 01-the-garden.md   # one file per chapter (or a single book.md; both work)
│   └── 02-flowers.md
├── styles.toml            # text styles: body, headings, quote, caption, classes, drop caps
├── pages.toml             # page templates (masters) + page rules (odd/even/first-of-chapter/...)
├── layouts.toml           # reusable page layouts: frames with position and size (composed pages)
├── assets/
│   ├── images/
│   └── fonts/             # project fonts, so the build is reproducible anywhere
├── extensions/            # optional: *.typ functions, custom components (advanced)
├── .booker/               # (git-ignored) cache, generated .typ, preview tiles
├── build/                 # (git-ignored) PDF / HTML / EPUB outputs
├── .gitignore             # generated
└── .gitattributes         # generated: images/fonts via LFS (optional), *.md text eol=lf
```

Principles:
1. **Text lives only in `.md` files.** Nothing about layout is copied into another file.
2. **Layout attaches to content through stable IDs, never page numbers.** Pages reflow; anchors don't move.
3. **The GUI edits files in place, keeping formatting and comments** (`toml_edit`, and small targeted Markdown edits). When a file changes on disk, the app reloads it (`notify` crate). Opening a project must never change any file.
4. **Deterministic output.** Keys are ordered and there are no timestamps or GUIDs in files. A GUI change produces a small diff.
5. `book.toml` has `format = 1`. Newer versions of the app migrate older projects explicitly, with a backup commit or copy.
6. **A project that uses no styling is valid.** A folder with one `book.md` opens and exports with the default theme.

### 5.2 `book.toml`

```toml
format = 1
title = "The Secret Garden of Mia"
author = "Mia P."
language = "en"
theme = "picture-book"          # built-in or ./themes/<name>

chapters = [                    # order; omit to use content/*.md sorted by name
  "content/01-the-garden.md",
  "content/02-flowers.md",
]

[page]
size = "8.5x8.5in"              # presets: a4, a5, letter, 6x9in, 5x8in, ...
margins = { top = "18mm", bottom = "22mm", inside = "20mm", outside = "15mm" }
facing = true                   # recto/verso aware
bleed = "3mm"

[toc]
enabled = true
depth = 2

[chapter]
start = "new-page"              # new-page | right-page | continue

[output.print]
format = "pdf"
crop-marks = true
[output.web]
format = "html"
```

### 5.3 Markdown extensions (kept minimal, Pandoc-compatible where possible)

```markdown
# The Garden {#garden .fancy}

::: {.poem break-before=page}
Roses are red…
:::

She loved the [flowers]{#flowers} in her grandmother's garden.

![Tulips in May](assets/images/tulips.jpg){#tulips anchor=flowers page=same align=top width=full}

<span class="handwritten">a note</span>   ← a small whitelisted subset of HTML becomes styles
```

- `{#id .class key=value}` on headings, images, fenced divs (`:::`), and bracketed spans `[text]{…}`. Other editors show this as plain text, and GitHub still renders the file readably.
- **HTML inserts:** we support a whitelisted subset (`span`, `div`, `br`, `sup`, `sub`, `u`, `class=`, and a limited `style=`). The subset maps to styles, so the PDF and HTML outputs match. Raw HTML outside the subset is kept only in the HTML output and produces a warning in PDF.
- Escape hatches: ```` ```{=typst} ```` blocks for PDF only and ```` ```{=html} ```` blocks for HTML only.
- Parser: `pulldown-cmark` plus our attribute and directive pass. It is the one whose source positions exclude block markers and are never optional, which is what click-to-source needs (`docs/FINDINGS.md`).

### 5.4 `styles.toml`

```toml
[text.body]
font = "Literata"
size = "11pt"
leading = 1.4
indent-first-line = "1.2em"

[text.heading-1]
font = "Arial"
size = "18pt"
weight = "bold"
background = "#d9f2d9"
break-before = "page"
keep-with-next = true

[text.body.first-in-chapter]          # contextual variants
indent-first-line = 0
first-line = { small-caps = true }
drop-cap = { lines = 3, font = "Cinzel Decorative", color = "#2a6" }
# or: drop-cap = { image = "assets/images/letters/{letter}.png", lines = 4 }

[class.poem]                          # used by {.poem}
align = "center"
style = "italic"
break-before = "page"
```

The cascade is simple: theme → `text.*` → contextual variant → class → inline attribute. The inspector panel shows which layer set each value.

### 5.5 `pages.toml`: page templates and rules

```toml
[template.default]
header = { center = "{chapter-title}", font-size = "9pt" }
footer = { outside = "{page}" }

[template.chapter-opening]
header = "none"
background = { image = "assets/images/vines.png", fit = "cover", bleed = true }

[[rule]]                    # rules apply in order; later rules override earlier ones
when = "odd"
background = { color = "#fffaf0" }

[[rule]]
when = "first-of-chapter"
template = "chapter-opening"

[[rule]]
when = { chapter = "flowers" }        # by anchor/ID, not page number
layers = [{ image = "assets/images/petal.svg", position = "bottom-right", size = "25mm" }]

[[rule]]
when = "blank"                          # pages inserted to start chapters on the right page
template = "empty"
```

Available selectors: `odd`, `even`, `left`, `right`, `first`, `last`, `blank`, `first-of-chapter`, `{ chapter = id }`, `{ from = id, to = id }`, `{ nth = "3n+1" }`, plus `all`/`any` combinations. Images that repeat on every page are **layers** in templates or rules. Overflow modes: `fit`, `cover`, `clip`, `bleed`, `tile`.

### 5.6 Two page modes: flowing text and composed pages

A project mixes both, chapter by chapter or page by page.

**Flow mode** (novels, papers, most text): text runs from page to page by itself, images float or anchor to a phrase. This is the default.

**Composed mode** (picture books, photo books, cookbooks, poetry with art): a page or a spread is made of **frames**. A frame is a box with a position, a size and a role:

- `text` — text is placed into it
- `image` — one image, with a fit or overflow setting
- `block` — a colour panel or decoration

A frame's position can be given in three ways, and they mix freely on the same page:

| Way | Example | For |
|---|---|---|
| Absolute | `at = ["20mm", "30mm"]`, `size = ["120mm", "60mm"]` | "Exactly here", dragged in the preview |
| Relative to the page or margins | `at = ["50%", "0"]`, `width = "full"`, `bleed = true` | Layouts that follow the page size |
| Grid cell | `grid = "row 1 / col 1 .. 2"` with a grid declared in the layout | Tidy, repeatable arrangements |

Frames may overlap, have a z-order, rotation and padding, so text over a full-bleed picture is easy.

```toml
# layouts.toml
[layout.image-top]
description = "Big picture on top, text below"
[[layout.image-top.frame]]
role = "image"; slot = "hero"; at = ["0", "0"]; size = ["100%", "60%"]; bleed = ["top", "left", "right"]; fit = "cover"
[[layout.image-top.frame]]
role = "text"; at = ["15mm", "62%"]; size = ["calc(100% - 30mm)", "auto"]; style = "picture-book-body"
```

```markdown
::: page {layout="image-top"}
![Mia opens the door](assets/images/garden.jpg){slot=hero}

Mia opened the little green door.
:::

::: page                                 ← no named layout: a free page
![](assets/images/cat.png){at="18mm,120mm" width="70mm" rotate="-4deg" z=2}

::: frame {at="15mm,20mm" size="90mm,45mm" style="picture-book-body"}
The cat was already waiting.
:::
:::
```

Rules that keep this from turning into a second copy of the text:

1. **Text still lives only in the Markdown file.** A frame either names a slot (`slot=hero`), or takes the content written inside it, or takes the next piece of content in order.
2. **Geometry lives wherever the user started.** One-off pages keep it in the Markdown attributes; reusable arrangements keep it in `layouts.toml`. Dragging a frame in the preview edits the same place it came from, and when the frame belongs to a shared layout, Booker asks whether to change every page that uses it.
3. **A composed page is a fixed page.** Reflowing the book never moves it.
4. **Text flows from frame to frame by default.** Frames belong to a chain (by default, the page's text frames in order, continuing onto the next page), and text simply continues. Per frame, that can be changed to `stop` (the frame holds only what it is given), `shrink-to-fit`, or `clip`. When nothing is left to hold the text, the problems panel says so.
5. **Automatic first, manual on top.** The author sees the rendered result and adjusts in place: drag a frame, break the text here rather than there, let this paragraph run a little tighter so the page ends well (copy-fitting, as in InDesign), keep these lines together. Each override is attached to the text or the frame it belongs to, so it survives later edits, and each one is listed in an "overrides" panel so they can be found and removed.
6. Even in flow mode the main text area is a frame: a page template can set its box, so "text only on the lower half of chapter openings" needs no composed page. Flow mode and composed mode are the same machinery — one chain of frames — which is also what later gives us margin notes and pull-out boxes for free.

**What makes this easy without training:** a **layout gallery**. The user picks a thumbnail ("picture on top, text below", "full-page picture with a text box", "two pictures side by side", "picture across the spread"), the page takes that shape, and anything can still be dragged afterwards. Dragging gives snapping to margins, grid and other frames, rulers, guides, arrow-key nudging, alignment tools, and "save this page as a reusable layout".

### 5.7 Extensibility (complexity lives in the project)

- `extensions/*.typ` can define components, e.g. `#let callout(body, ..args) = …`, which are called from Markdown with `::: {.callout}`. The mapping is declared in `styles.toml`: `[class.callout] component = "callout"`.
- `themes/` in a project or in a shared git repo: a theme is simply a `styles.toml`, a `pages.toml`, fonts and images.
- The app itself has no plugin API before 1.0. Extensions are data and Typst code, not native code, so nothing unsafe runs.

---

## 6. System architecture

```
┌──────────────────────── Tauri app ────────────────────────┐
│  Svelte UI: chapter tree · CodeMirror editor · page preview │
│  (spreads) · inspector (style/page/image) · problems panel  │
└──────────────▲──────────── IPC (typed, ts-rs) ──────────────┘
               │   booker:// protocol → page tiles (PNG/SVG)
┌──────────────┴──────────── Rust core ───────────────────────┐
│ booker-project  load/save/watch, toml_edit, migrations       │
│ booker-doc      Markdown+attrs → Doc AST (with source spans) │
│ booker-style    theme/style/page-rule resolution (cascade)   │
│ booker-frames   frame geometry: absolute/relative/grid, overflow│
│ booker-typst    AST+styles → .typ, Typst World, fonts, compile│
│ booker-place    multi-pass anchor solver (query → re-emit)   │
│ booker-html     AST+styles → HTML/CSS, EPUB 3                │
│ booker-check    diagnostics: rules, severities, locations, fixes│
│ booker-print    bleed, crop marks, preflight checks          │
│ booker-cli      `booker new|build|check|render|where|fmt|mcp` │
└──────────────────────────────────────────────────────────────┘
```

Key technical choices:
- **Generate Typst source as text** (kept in memory and optionally dumped to `.booker/gen/`), rather than building Typst content objects directly. This is easy to debug, easy to test with snapshots, and gives an "eject to Typst" option. A span map links generated code to Markdown positions for click-to-source.
- **Incremental preview:** Typst's memoized compiler, debounced to about 150 ms, and rendering only the visible pages. Target: under 100 ms from keystroke to preview for small edits in a 300-page novel.
- **Anchor solver (`booker-place`):** pass 1 compiles and queries which page each anchor and each flow block lands on. Pass 2 places each float in the flow at the start of the target page (same, previous, next, or facing page). Repeat until nothing changes (at most N passes), then report "could not satisfy" in the problems panel and apply the best result found. This is the biggest technical risk in the plan, and Wave 6 track A is where it is built and proved.
- **Text wrapping around images:** `meander` (Typst package, vendored and pinned) for pages that have wrapped images. Otherwise, images float at the top or bottom, or sit between paragraphs.
- **Composed pages** become a Typst page whose body is a stack of `place(dx:, dy:)` boxes — a direct, predictable mapping, and the part of the system we have the most control over.
- **Frame chains are core, not an extra.** Text continues from frame to frame on its own, so the chain engine carries weight: `meander` (MIT, so we can vendor it and fork it if we must, ideally contributing fixes back) does exactly this today, and Typst's `measure` tells us whether content fits, which drives shrink-to-fit and the "nothing is left to hold this text" warning. Wave 5 track B decides whether we lean on the package, fork it, or write our own chain layout. Same machinery serves flow mode, composed pages, margin notes and pull-out boxes, so it is worth doing properly once.
- **Dragging must round-trip.** The preview knows each frame's identity, so a drag edits exactly the attribute or `layouts.toml` entry it came from, in millimetres rounded to one decimal, leaving the rest of the file untouched.
- **Good typography without asking.** From Wave 2 the defaults are the ones a book should have anyway: justified text with hyphenation in the book's language, protection against single lines stranded at the top or bottom of a page, proper quotation marks and dashes for the language, ligatures and kerning on. Typst gives us all of these (its `costs` control for stranded and over-short lines, `hypher` hyphenation patterns, OpenType features), so the work is picking defaults per template, not building machinery. The knobs appear in Wave 3, the fine control in Wave 10; an amateur never has to touch any of it.
- **Fonts:** a curated bundled set of open fonts (e.g. Literata, EB Garamond, Source Serif 4, Inter, Atkinson Hyperlegible, Andika for kids, Cinzel Decorative for initials), plus project fonts and system fonts. The problems panel warns when the project uses a system-only font and offers "copy into project".
- **Contracts first:** `Doc AST`, `ResolvedStyle`, and the IPC command types are defined in Rust and exported to TypeScript (`ts-rs`). These contracts are frozen at the start of each wave so the tracks can work in parallel.

---

## 7. Releases and updates (from day one)

- CI: GitHub Actions matrix (macOS arm64 and x64 or universal, Windows x64 and arm64, Linux x64 AppImage/deb/rpm) using `tauri-action`.
- Signing: macOS Developer ID plus notarization; Windows Authenticode (Azure Trusted Signing); updater artifacts signed with a minisign key kept in CI secrets. Both accounts are owned by you; Wave 1 track A wires them in, and works without them until they exist. Unsigned builds work in the meantime, but Gatekeeper and SmartScreen will warn.
- Identity: app ID `com.github.can3p.booker`, matching the repository, so no domain is needed. **This must be settled before the first public release**, because changing it later means installed copies stop receiving updates. Renaming the app is cheap; renaming the ID is not.
- Update flow: check on start and every 24 h, plus "Check for updates…" in the menu. Download in the background, then "Restart to update"; never force-restart while a file is unsaved. Channels: `stable` and `beta` (the endpoint URL depends on the channel).
- Hosting: `latest.json` / `beta.json` as GitHub Release assets (a static endpoint, no server). CrabNebula Cloud is an option later if we need download stats or staged rollouts.
- Linux: AppImage updates itself; deb and rpm are supported by the updater too. Flathub is optional later (it updates through Flatpak).
- The format version in `book.toml` is checked against the app version. If an old app opens a newer project, it offers to update the app.
- Every wave ends with a tagged release that **older installed builds pick up through the updater**, which tests the update path on every release.

---

## 8. Development waves

From Wave 1 on, each wave ends with a signed, installable build for macOS, Windows and Linux, delivered through the updater, **and with the tutorials that teach whatever it added** (`docs/requirements.md`; `AGENTS.md` §4). Each wave has a **contracts** step (done by the lead model, about 1 day), then **parallel tracks** in separate git worktrees, then an **integration and release** step.

Model tiers used below:
- **L (large: Opus):** architecture, contracts, solver, codegen core, tricky integration.
- **M (mid: Sonnet):** features with a clear spec, UI panels, exporters.
- **S (small: Haiku):** boilerplate, CI YAML, fixtures, golden tests from a template, docs, theme/template content, i18n strings, icon wiring, lint fixes.

**This section describes work that has not been done yet.** When a wave finishes, its section here is replaced by one line pointing at `docs/WAVE-LOG.md`, which is the only place that says what actually shipped — the plan must never become a second, diverging history (`AGENTS.md` §2).

### Waves 0, 0.5 and 0.6 ✅ done
The core (engine, project format, `booker new` and `booker build`), continuous integration on three platforms, and the first tutorial with the test that replays it. What each of them shipped, what it deviated from and what it cost is in [`docs/WAVE-LOG.md`](WAVE-LOG.md); what was learned building them is in [`docs/FINDINGS.md`](FINDINGS.md). Their briefs remain in `docs/waves/` for reference.

Two decisions made in those waves that still constrain everything below: the Typst engine is kept alive per open project, because the incremental recompile is what makes a live preview possible at all (`docs/FINDINGS.md`), and nothing in CI runs on a schedule — every check runs on push and pull request, with the three-platform test matrix widening only on pull requests into `main`.

### Wave 1 — merged, release outstanding
The desktop application and the release pipeline: a window that opens a book folder, edits it, draws its pages and exports a PDF, and installers for macOS, Windows and Linux that update themselves. The one step left is the first release and the check that it updates itself (`docs/waves/wave-1.md`, "What remains"); Wave 2 starts after it. What it shipped, what it deviated from and what it cost is in [`docs/WAVE-LOG.md`](WAVE-LOG.md); its brief remains in `docs/waves/wave-1.md`.

Three decisions taken there that constrain what follows: there is exactly one translation from a book to pages (`booker_typst::book`, behind `Engine::set_book`) and both the window and the CLI go through it; the application keeps one engine per open project, because the memoized compiler is what makes the preview incremental; and an outside change to the folder is a reload, never a merge, because everything the window holds is derived from files that something else may have rewritten.

### Wave 2: "Write a simple book" (MVP)
**Demo:** create a novel from a template, write chapters (or edit them in VS Code and watch the preview follow), get a table of contents, and export a good-looking PDF.

| Track | Work | Tier |
|---|---|---|
| A. Doc AST | Markdown → AST (headings, paragraphs, emphasis, lists, quotes, simple images, breaks, attributes), with source spans | L |
| B. Codegen | AST → Typst: chapters, `chapter.start`, TOC, page setup presets, margins, base font, **typographic defaults** (justification, hyphenation by language, stranded-line protection, smart quotes and dashes, ligatures) | L |
| C. Editor | CodeMirror 6: Markdown live-styling (syntax hidden like Obsidian), chapter tree (add/rename/reorder → `book.toml`), autosave | M |
| D. Preview | Incremental compile loop, visible-page rendering, zoom, click-to-source and cursor-to-page sync | M |
| E. Project I/O | Conflict handling on top of Wave 1's watcher (an outside edit meeting an unsaved buffer offers both versions), `toml_edit` writer, `.gitignore`/`.gitattributes` generation | M |
| F. Templates & fonts | 4 starter templates (novel, picture book, poetry, short paper), bundled font set + licenses | S (content) / M (wiring) |
| G. Tests & CLI | Golden tests: fixture projects → PNG snapshots with pixel diff; `booker build`, and the first `booker check` rules (missing files, unresolved references) | S (fixtures) / M (harness) |

### Wave 3: Styles
**Demo:** click a heading, change it to bold 18 pt Arial on a green background, and see every heading update. Add a drop cap from an image to each chapter.

| Track | Work | Tier |
|---|---|---|
| A. Style engine | `styles.toml` schema, cascade, contextual variants, classes, validation → problems panel | L |
| B. Codegen for styles | Property → Typst mapping, `keep-with-next`, `break-before`, first-line and drop-cap (text and image) | L/M |
| C. Inspector UI | Style panel (with "where does this value come from?"), class picker, "apply to all similar" | M |
| D. HTML subset | Whitelisted inline HTML → styles; warnings for unsupported HTML | M |
| E. Font manager | Bundled/project/system list, preview, "copy into project", fallback warnings | M |
| F. Typography controls | The everyday knobs on top of the Wave 2 defaults: hyphenation on/off plus a project word list, justification, line spacing, spacing between paragraphs, stranded-line strictness, small caps, all per style | M |
| G. Themes | 3 more themes that use the new style features, documentation pages | S |

### Wave 4: Pages, templates and rules
**Demo:** every odd page gets a cream background, chapter openings get a full-bleed vine image with no header, and running heads show the chapter title. All of this comes from rules, not manual edits.

| Track | Work | Tier |
|---|---|---|
| A. Rule engine | `pages.toml`, selectors, override order, facing pages (inside/outside margins), blank-page insertion | L |
| B. Headers/footers | Running heads, page numbers (roman numerals for front matter), per-template layers | M |
| C. Spread preview | Two-page spread view, page thumbnails strip, "why does this page look like this?" (lists the matching rules) | M |
| D. Print prep | Bleed, crop marks, PDF/A option, preflight checks (image DPI < 300, missing fonts, overflowing content) | M |
| E. Template editor UI | Visual editor for page templates (header/footer slots, background, layers) | M |
| F. Fixtures | Golden tests for rule combinations | S |

### Wave 5: Composed pages — frames, free placement, direct manipulation
**Demo:** make a kids book page by page: pick "picture on top, text below" from the gallery, then drag the text box somewhere else on the next page, put a cut-out cat at an angle over the picture, and save the arrangement as a reusable layout.

| Track | Work | Tier |
|---|---|---|
| A. Frame model | `layouts.toml` and page/frame attributes, geometry (absolute, relative, grid), z-order, rotation, padding, validation | L |
| B. Codegen | Frames → Typst placement, frame chains (text continues by itself), text fitting with `measure`, per-frame overrides (`stop`, shrink-to-fit, clip) | L |
| C. Direct manipulation | Drag, resize, nudge, snapping and guides, rulers, alignment tools, z-order, round-trip writeback, undo | L/M |
| D. Layout gallery | Preset layouts with thumbnails, "apply to page/spread/chapter", "save page as layout", "change all pages using this layout" | M (presets: S) |
| E. Picture-book template | A full sample picture book that exercises every frame feature, plus docs | S |
| F. Fixtures | Golden tests for frame geometry and overflow | S |

Exit criteria: a 24-page picture book can be laid out end to end without touching a text file by hand, and its `layouts.toml` diff stays readable.

### Wave 6: Images and anchoring in flowing text
**Demo:** anchor a flower photo to the word "flowers", choose "same page, top", and it follows the text as you edit. Wrap text around a cut-out image. Drag and drop images into the editor.

| Track | Work | Tier |
|---|---|---|
| A. Solver | Production `booker-place`: same/previous/next/facing, convergence, diagnostics | L |
| B. Placement modes | Inline, top/bottom float, full page, text wrap via `meander`, size/fit/overflow, captions | L/M |
| C. Image UX | Drag/drop into the editor (copies to `assets/`), image properties panel, anchor-link overlay in preview, drag in preview to change alignment | M |
| D. External editing | "Open in default editor", reload on change, image optimization hints | S/M |
| E. Tests | Solver stress fixtures (many anchors, conflicting constraints) | S (fixtures) / L (review) |

### Wave 7: Diagnostics and the agent surface
**Demo:** break a book on purpose — delete an image file, overflow a caption, point an anchor at a phrase that no longer exists — then tell Claude Code in that folder "the layout looks broken, analyse the errors and fix them". It runs `booker check`, sees exactly what is wrong and where in the source, fixes it, and the app, still open, reloads and shows the result.

| Track | Work | Tier |
|---|---|---|
| A. Diagnostics engine | Rule registry with stable IDs and severities, source **and** layout locations, `[check]` configuration, inline suppressions, `booker explain <rule>` | L |
| B. Rules backfill | The rules for everything Waves 1–5 shipped: references, layout, style, text, images (one rule per ticket) | M (rules: S) |
| C. CLI | `check` (human / JSON / SARIF, exit codes), `render --page`, `where`, `page`, `fmt`, `fix --safe` | M |
| D. MCP server | `booker mcp` over stdio, tools mirroring the CLI, confined to the project folder, read-mostly | M |
| E. Resilience | Hardening against outside edits: atomic writes, echo suppression, revision counter, partial load of broken projects, and the stress tests that prove Wave 1's watcher and Wave 2's conflict handling survive thirty files rewritten at once | L |
| F. Discoverability | JSON Schemas generated from the parsing structs, `booker capabilities --json`, `booker explain <topic\|rule>`, `booker recipe list/show`, "did you mean" suggestions on unknown keys | M (recipes: S) |
| G. Agent onboarding | `AGENTS.md` and a Booker skill generated into new book projects from the capability manifest, `booker explain format` | S |
| H. MCP conformance | Real-client tests over stdio: handshake, tools matching the capability manifest, schema validity, CLI/MCP parity, refusals, cancellation, deterministic output, on all three platforms | M |
| I. Eval harness | `evals/` in this repository (never shipped): fixture projects, a scenario runner with programmatic success checks, instrumentation of which discovery commands the agent used, `cargo xtask eval` and a written result for the wave log | L (scenarios: S) |
| J. CI | `booker check --format sarif` annotations in the sample GitHub workflow | S |

Exit criteria: while the app is open, an outside process rewriting every file in the project leaves the app correct and responsive; `booker check` finds every fault in the "deliberately broken book" fixture and `booker fix --safe` repairs the mechanical ones; and an agent given only the project folder and the CLI can add a drop cap, move an image to a fixed position and make chapters start on the right — without being told the format beforehand. That last one is the real test, and it is the first scenario in the eval harness.

### Wave 8: The furniture of a book
**Demo:** a novel with footnotes, an epigraph, a proper copyright page, ornaments between scenes, and a non-fiction book with margin notes, a bibliography and an index.

| Track | Work | Tier |
|---|---|---|
| A. Notes | Footnotes (`[^1]`), endnotes per chapter or book, **margin notes** (a frame in the outside margin, using the same chain machinery), note styling and numbering | L/M |
| B. References | Cross-references ("see page 42", "Chapter 3"), captions and automatic figure numbering, lists of figures and tables | M |
| C. Bibliography | `references.bib` or CSL-JSON in the project, `[@key]` citations, citation styles, per-chapter bibliographies | M |
| D. Index and glossary | Index entries marked in the text, alphabetised index with page numbers, glossary | M |
| E. Front and back matter | Half-title, title, copyright page, dedication, epigraph, colophon, appendices, roman page numbers before the main text, "chapters start on a right-hand page" | M (content: S) |
| F. Ornaments | Scene-break ornaments, chapter-opening flourishes and vignettes as style and page-rule features, a small bundled ornament set | S/M |

These are independent of each other: good wave for many parallel tracks.

### Wave 9: Outputs and publishing
**Demo:** one click exports a print PDF, a website and an EPUB. Pushing to GitHub builds the PDF in CI.

| Track | Work | Tier |
|---|---|---|
| A. HTML | AST + styles → HTML/CSS (single page and multi-page), page rules translated to screen styling where it makes sense | M |
| B. EPUB 3 | Reflowable EPUB for flowing books, **fixed-layout EPUB for composed books** (frames map to absolutely positioned boxes), navigation, fonts, epubcheck in CI | M/L |
| C. Output presets | Print / screen / KDP / IngramSpark presets (trim sizes, bleed, spine notes) | M (research: S) |
| D. CLI & CI | `booker build --preset`, published GitHub Action, sample workflow in templates | S/M |
| E. Accessibility | PDF/UA tagging, alt-text prompts for images | M |

### Wave 10: Fine typography and print production
**Demo:** send a file to a print shop that asks for CMYK with a colour profile, with facing pages whose lines sit on the same baseline, and a cover whose spine width matches the page count.

| Track | Work | Tier |
|---|---|---|
| A. Fine typography | Baseline grid across facing pages, hanging punctuation and optical margins, word and letter spacing limits, OpenType feature panel (old-style figures, alternates, swashes), per-language hyphenation exceptions | L |
| B. Copy-fitting | Per-paragraph and per-page nudges made while looking at the render: tighten or loosen, pull a line back, keep lines together, "make this page end here", all listed in an overrides panel | M |
| C. Colour management | A colour value that knows its space (already in the model from Wave 3), CMYK and spot colours, an ICC profile in the project, output intent, PDF/X export. **Typst has CMYK colours but no ICC or output intent yet**, so this runs as a post-processing step (Ghostscript or an ICC library) behind one export hook | L |
| D. Covers | A cover project (front, spine, back) with spine width computed from page count and paper, bleed, barcode/ISBN placement | M |
| E. Preflight | Expanded checks: stranded lines, overflowing frames, low-resolution images, unembedded fonts, content outside the safe area, missing alt text, colour space mismatches | M |

### Wave 11: Extensibility and polish (→ 1.0)
| Track | Work | Tier |
|---|---|---|
| A. Git-adjacent, not git | No git UI in 1.0: generated `.gitignore`/`.gitattributes`, "Open project folder / in terminal", a short guide on using GitHub with Booker, and making sure outside changes always reload cleanly | S/M |
| B. Extensions | `extensions/*.typ` components, themes installed from a git URL, "eject to Typst" | L |
| C. i18n | UI translations, hyphenation per language, spell-check languages | S |
| D. Performance | Profiling 500+ page books, memory limits, tile cache | L |
| E. Onboarding | Interactive first-run tour, sample book, help site | S/M |
| F. Reliability | Opt-in crash reporting, recovery of unsaved buffers, format migration tests | M |

### Parallelization rules
1. Contracts first (AST, style schema, frame geometry, IPC types) are owned by the L-tier lead and merged before the tracks start.
2. One worktree and one branch per track — see §14 for the mechanics. Tracks only touch their own crates or UI folders. Shared files (`Cargo.toml`, the IPC registry) change only in the integration step.
3. Each S-tier task comes as a ticket with input files, expected output, and a command to verify it (e.g. "add fixture X; `cargo test -p booker-typst golden::x` must pass").
4. Integration step: merge, run the full golden suite on 3 OSes, tag the release, and confirm the previous build updates to it.
5. **Every wave ships the diagnostic rules for what it added.** A way for a book to break that Booker cannot name and locate is unfinished work — from Wave 7 on, that means rules in the registry; before it, at least a clear message in the problems panel.

---

## 9. What books need, and when it arrives

The shape of the argument: **the things that make a book look right by itself come early; the things that need judgement come later, and are never required.**

| Feature | When | Note |
|---|---|---|
| Justified text, hyphenation, real quotation marks, ligatures, protection against stranded lines | **Wave 2, on by default** | Nearly free with Typst; this is most of what separates an amateur-looking page from a decent one |
| Chapters, table of contents, page numbers, running heads | Waves 1–3 | |
| Text styles, drop caps, first-line styling | Wave 3 | |
| Everyday typography knobs per style | Wave 3 | Hyphenation on/off, spacing, stranded-line strictness |
| Page rules, master pages, facing pages, bleed | Wave 4 | |
| Free placement, frames, text flowing between them, layout gallery | Wave 5 | |
| Anchored images, text wrapping around images, captions | Wave 6 | |
| Footnotes and endnotes | Wave 8 | |
| **Margin notes / side notes** | Wave 8 | Falls out of the frame-chain model: a frame in the outside margin |
| Cross-references, figure numbering, lists of figures | Wave 8 | |
| **Bibliography and citations** | Wave 8 | Typst has this natively; we map `[@key]` and a `references.bib` in the project |
| Index, glossary | Wave 8 | |
| Front and back matter (title, copyright, dedication, epigraph, colophon, appendix) | Wave 8 | Mostly templates and numbering rules |
| **Vignettes and ornaments** (scene breaks, chapter flourishes) | Wave 8 | Style and page-rule features plus a bundled ornament set |
| `booker check`: first rules (missing files, unresolved references) | Wave 2 | Grows one wave at a time |
| **Full diagnostics, CLI and MCP server for agents** | Wave 7 | Rule IDs, source locations, page renders, `fix --safe` |
| HTML, EPUB (reflowable and fixed-layout), CLI builds | Wave 9 | |
| Accessibility: alt text, tagged PDF | Wave 9 | The document model carries roles from Wave 2, so tagging is a rendering concern |
| Baseline grid, optical margins, OpenType feature control | Wave 10 | For people who care; invisible to everyone else |
| Copy-fitting overrides made while watching the render | Wave 10 | |
| **CMYK, spot colours, ICC profiles, PDF/X** | Wave 10 | Needs a post-processing step; see §10 |
| Cover with computed spine width, ISBN barcode | Wave 10 | |
| Tables, code blocks, mathematics | Waves 1–2 (basic), later refinement | Typst gives mathematics and tables cheaply |
| Verse and lyrics, dialogue conventions, recipes | As templates and classes, any wave | User-level, not engine-level |
| Beyond 1.0 | | Vertical CJK text, shared template gallery, collaboration, plugin API, print-shop integrations |

## 10. Choices made so that later features stay possible

None of the following costs much now, and each one is expensive to retrofit:

1. **A colour is a value with a colour space**, not a hex string: `#2a6`, `cmyk(...)`, `spot("Pantone 185 C")`. `book.toml` reserves `color-space` and `icc-profile`. Wave 3 stores and round-trips them even though only sRGB renders.
2. **Export runs through one hook** after Typst produces the PDF, so ICC conversion, PDF/X output intents or an imposition step slot in without touching codegen.
3. **Unknown keys in project files are preserved, not dropped.** An older Booker opening a project made by a newer one keeps what it doesn't understand, warns, and writes it back unchanged. This is what makes the format survive years of feature growth.
4. **The document model carries meaning, not just looks**: a note is a note, a citation is a citation, a caption is a caption. Accessibility tagging, EPUB semantics and the index all read from that later.
5. **Typography is one resolved set of properties** handed to codegen. Adding a baseline grid or optical margins later changes that set and the codegen, not the file format or the UI.
6. **Everything is a frame chain.** Margin notes, pull-out boxes, multi-column pages and composed pages are all the same mechanism, so each of them is a feature rather than a new engine.
7. **Every automatic decision can be overridden, and every override is visible.** Overrides attach to text or frames, never to page numbers, and are listed in one panel so a book can be cleaned up before printing.
8. **Everything an agent needs to know is generated from the code**: schemas from the parsing structs, the capability manifest from the property registries, rule documentation from the rule registry. Hand-written references go stale within a wave; generated ones cannot.
9. **Everything the app knows, a command can print.** The app never holds state the CLI cannot reach, which is what makes an agent surface a thin layer rather than a second implementation.
10. **Language is a document property from day one** (hyphenation patterns, quotation marks, numbering words like "Chapter"), so a second language is a settings change, not a rewrite.

## 11. Agent-facing design

Assume the owner of a book opens Claude Code in the book folder and says "fix the spelling", "make the chapter openings calmer", or "the layout looks broken — find the errors and fix them". Booker has to be a good citizen in that workflow. Half of it comes free from the format being plain text; the rest is three obligations.

### 11.1 Survive being edited from outside

- **Nothing is locked, and nothing lives only in memory.** The project on disk is the whole truth; `.booker/` is a derived cache that can be deleted at any moment without loss.
- **Writes are atomic** (temporary file, then rename) and targeted, and the app ignores the echo of its own writes by comparing content hashes.
- **The app saves eagerly** (on idle and when focus leaves), so an outside agent rarely meets an unsaved buffer. When it does, the user is offered the two versions rather than losing one.
- **Bulk changes are normal, not an error**: an agent rewriting thirty files, or a `git checkout` of another branch, arrives as one coalesced reload with one re-render, keeping the page position and, where the text still matches, the cursor.
- **A broken project still opens.** Missing image, chapter file listed but deleted, unparseable style file: load what is loadable, report the rest as diagnostics. Refusing to open is never the answer, because that is exactly when someone needs to see the errors.
- **The project carries a revision counter**, so a tool can tell whether what it read is still current.

### 11.2 Say precisely what is wrong

One diagnostics engine (`booker-check`) feeds four surfaces: the problems panel in the app, the CLI, CI annotations, and the agent tools. Rules have **stable IDs**, so a person or an agent can talk about `BK-LAYOUT-003` and configure it.

| Family | Examples |
|---|---|
| `ref.*` | Anchor referenced but not defined, image file missing, chapter listed in `book.toml` but absent, dangling cross-reference, citation with no bibliography entry, duplicate ID |
| `layout.*` | Text overflows with no frame left to continue into, frame outside the page or the safe area, a frame covering text, an image anchor that cannot be satisfied, a broken chain |
| `style.*` | Unknown style or class, property meaningless for its target, font not available, rules that contradict each other |
| `text.*` | Lines stranded at the top or bottom of a page, a chapter that renders empty, a heading with no text under it |
| `image.*` | Resolution too low for the trim size, missing alt text, colour space mismatch |
| `print.*` | Content inside the bleed, spine width inconsistent with the page count, fonts not embedded |
| `format.*` | Unknown or deprecated keys, format version newer than this build |

Each diagnostic carries: rule ID, severity, a message written for a human, **the source location in the Markdown or TOML file** (file, line, column), the layout location (page, frame, chapter), and, where the fix is mechanical, a patch. The source location matters most: "text overflows on page 12" is useless, "`content/03-flowers.md:145` overflows the `caption` frame on page 12" can be acted on. This reuses the span map that click-to-source needs anyway.

Configurable in `book.toml` under `[check]` (severity per rule, thresholds such as minimum image resolution), plus `<!-- booker-ignore BK-IMAGE-001 reason -->` in the text for one-off exceptions. `booker check` exits non-zero on errors, which is what makes it useful in CI and in an agent loop.

**Every wave ships the rules for the features it adds.** A feature that can break in a way we cannot name is not finished.

### 11.3 Give agents eyes and hands

```bash
booker check --format json          # or human (default), or sarif for CI annotations
booker build --preset print
booker render --page 12 --png out.png     # an agent with vision can look at the page
booker where "flowers"              # which page and frame a phrase landed on
booker page 12                      # which source ranges ended up on this page
booker fmt                          # canonical formatting of project files
booker fix --safe                   # apply only the mechanical fixes
booker explain BK-LAYOUT-003        # what the rule means and how to fix it
booker mcp                          # the same surface as an MCP server over stdio
```

The MCP server ships inside the CLI binary and needs no running app, so `claude mcp add booker -- booker mcp` in a book folder is the whole setup. Its tools mirror the commands: `check`, `render_page`, `locate`, `page_contents`, `project_info`, `fix_safe`.

**Deliberately read-mostly.** Agents already edit text well with their own tools, and the project is text on purpose; what they lack is truth about the rendered result. So Booker offers eyes (render, diagnostics, "where did this land") and only two writing tools (`fmt`, `fix --safe`). A larger write API would mostly be a worse way to edit a Markdown file.

Later, optionally: the running app exposes the same tools on a local socket with a token, so an agent can act on what the user is currently looking at and the user watches the changes land live. The CLI path stays the primary one, because it works headless, in CI, and without permissions questions.

### 11.4 How an agent learns what is possible

The write-API question answers itself once an agent can find out what the project understands, instead of guessing at key names. Five mechanisms, cheap to build because each is generated from the code rather than written by hand:

**1. Schemas, derived from the parser.** JSON Schema for `book.toml`, `styles.toml`, `pages.toml` and `layouts.toml`, generated from the same Rust structs that parse them (`schemars`), so they can never drift out of date. `booker schema styles` prints one; generated project files carry a `#:schema` line, so a plain text editor validates them too. This tells an agent exactly which keys exist, which values are allowed and what is required.

**2. A capability manifest.** `booker capabilities --json` prints one structured document describing the whole vocabulary: the Markdown attributes and directives we understand, the style properties valid for each kind of element, page selectors, frame properties, units, colour notations, output presets, diagnostic rules, CLI commands — each tagged with the app version and format version, so an agent knows what *this* build supports. A few kilobytes, read once.

**3. Drill-down explanations.** `booker explain drop-cap`, `booker explain page-rule`, `booker explain BK-LAYOUT-003`. An index plus targeted lookups keeps the agent's context small: it reads the one thing it needs instead of a manual.

**4. Recipes, because examples beat specifications.** `booker recipe list` and `booker recipe show chapter-starts-on-right` produce short worked examples: the three lines to add, and where. Most real requests ("give chapters a drop cap", "put this image at a fixed spot", "make odd pages cream") are recipe-shaped, and an agent working from a correct example is far more reliable than one reasoning from a schema.

**5. Errors that teach.** Guessing is fine when the feedback is good: unknown keys produce "unknown property `font-size` in `[text.body]` — did you mean `size`? see `booker explain text-style`". `booker check` is fast and precise, so an agent converges in one or two passes even when it started from a wrong assumption. This is the safety net under all of the above, and it is worth the effort on its own, because it is also what the human sees.

All five are exposed through the MCP server as well, so an agent connected to the project discovers them through the protocol rather than by reading documentation.

**Where this leaves write actions.** With the vocabulary discoverable, the agent should edit the project files directly — they are text, that is what agents are good at, and the user can read the diff. Booker offers write tools only where the operation genuinely needs the layout engine in the loop, which an agent cannot compute from the source:

- `fix --safe`: apply the patch a diagnostic already describes.
- Fit-and-iterate operations: "shrink this until the text fits", "pull this line back", "make the chapter end on this page" — each needs render, measure, adjust, repeat.
- `fmt`: canonical formatting, so an agent's edit and the app's edit produce the same file.

Anything expressible as a text edit stays a text edit.

### 11.5 Proving it works: MCP conformance and agent evals

Two kinds of test, because two different things can break. The protocol can break, which is deterministic and belongs in CI on every pull request. The *experience* can break — the vocabulary stops being discoverable, an error message stops suggesting the right key, a recipe goes stale — which no unit test catches, and which decays silently.

**MCP conformance tests** (CI, every pull request, all three platforms):

- Spawn `booker mcp` over stdio with a real client, complete the handshake, and list the tools.
- **Tools must match the capability manifest**, checked automatically. This is the invariant that stops the two from drifting: a new command without a tool, or a tool the manifest never mentions, fails the build.
- Every tool's input schema is valid JSON Schema, and every declared example validates against it.
- **CLI and MCP parity**: for each read tool, the MCP result equals the CLI's JSON output for the same question, on the same fixture. One implementation, proven.
- Failure cases behave: a path outside the project is refused, a page that does not exist returns a clean error rather than a panic, an unparseable project still answers `check`, a long render can be cancelled.
- Output is deterministic — sorted keys, project-relative paths, no timestamps — so an agent diffing two runs sees only real change. This is a requirement on the output format, not only on the tests.
- Windows gets the same run, because path handling and stdio buffering differ there.

**Agent evals** — a development harness in this repository (`evals/`, run with `cargo xtask eval`), not part of the shipped application. Scenarios and fixtures live with the source; nothing about them reaches a user's machine. Run by hand, at the end of a wave and before a release:

Each scenario is a fixture project, a prompt phrased the way a person would phrase it, and a **programmatic success check** — so the assertion is deterministic even though the agent is not.

| Scenario | Prompt | Passes when |
|---|---|---|
| Drop cap | "Give each chapter a large decorative first letter" | `styles.toml` has a valid drop-cap entry, `booker check` is clean, the golden render shows it |
| Fixed placement | "Put the cat picture in the bottom-left corner of page 3, about 70 mm wide" | The image renders within tolerance of that position on that page |
| Right-hand starts | "Chapters should start on right-hand pages" | Every chapter opens on a recto; blank versos inserted |
| Repair | "The layout looks broken, analyse the errors and fix them" | Starting from the deliberately broken fixture, `booker check` ends clean with no content lost |
| Spelling pass | "Fix the spelling throughout" | All thirty files rewritten from outside; the app, running, reloads correctly and loses no edits |
| Explain | "Why is the flower photo on the wrong page?" | The answer names the anchor and the unsatisfied constraint |

What the evals measure beyond pass or fail: how many turns it took, and **which discovery commands the agent used** — `capabilities`, `explain`, `recipe`, or none. An agent that guessed and got a "did you mean" correction tells us a recipe is missing; an agent that read the manifest and still got it wrong tells us the manifest is unclear. That instrumentation is the point of running them.

Policy: MCP conformance is an ordinary integration test suite — cheap, deterministic, part of `cargo test`, and blocking in CI. **Evals are a deliberate manual step**: `cargo xtask eval` run by a person at the end of a wave and before a release, never on a schedule, because each run costs model time and needs an API key that CI should not hold. Each scenario should pass in at least four runs out of five; the result is written into the wave log, and a drop in pass rate is a reason not to release. Keeping it manual also keeps the scenario set honest — if running it is a chore, the set is too big.

### 11.6 Teach the agent the format

`booker new` writes an `AGENTS.md` **into the book project** — what each file means, how to edit safely, that `booker check` must pass before finishing — and installs a Booker skill in the book repo, so a Claude Code session there loads the vocabulary when it becomes relevant rather than reading it up front. Both are generated from the capability manifest, so they describe the installed version and never go stale. `booker explain format` prints the same specification on demand.

## 12. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Breaking changes in Typst (pre-1.0) | Codegen breaks on upgrade | Pin the version; all Typst code in `booker-typst`; golden tests act as the upgrade gate |
| Anchor solver fails to converge | Images land on the wrong page | Built in Wave 6 track A; limit the number of passes; report the problem clearly; manual "pin to page" fallback |
| Text-wrap quality (`meander`) | Awkward layout | Use it only when asked; default to floats; upstream fixes |
| Frame chains carry the whole layout model, and today they rest on one package | Text stops continuing between frames | `meander` is MIT, so we can vendor, fork and contribute back; Wave 5 track B decides between package, fork and our own chain layout; the manual overrides (break here, shrink, stop) are the escape hatch in every case |
| Colour management (CMYK, ICC, PDF/X) is missing in Typst | A print shop rejects the file | Colours carry their space in the model from Wave 3, so nothing has to be retrofitted; the actual conversion is a post-processing step behind one export hook (§10.2, Wave 10); most print-on-demand services accept RGB today |
| Fine typography (baseline grid, optical margins) is not native | Books look slightly less refined than InDesign output | Typst already gives hyphenation, justification and stranded-line control, which is most of the visible quality; the rest is Wave 10 and does not block anything earlier |
| An outside agent and the app edit the same file at the same moment | A change is lost | Eager autosave, atomic writes, echo suppression, and both versions offered rather than one discarded; Wave 7 stress-tests exactly this |
| Diagnostics that are vague ("something overflows on page 12") | Neither person nor agent can act on them | Every rule must carry a source location; a rule without one does not ship |
| Drag-and-drop editing fights hand-edited files | Users lose their formatting or their edits | Frames carry stable IDs; writes are targeted and keep comments; every drag is one undo step and one small diff |
| PDF/X or CMYK needed by some printers | Print shop rejects the file | Most POD services (KDP, IngramSpark, Lulu) accept RGB PDF; verify Typst's roadmap; a post-processing step with Ghostscript as an optional external tool |
| Linux WebKitGTK performance and quirks | Sluggish UI on Linux | Render the preview in Rust; test AppImage on 2 distros in CI |
| Code-signing costs and setup | Scary install warnings | Set up in Wave 1 track A; budget for an Apple Developer account and Azure Trusted Signing |
| Markdown extensions look odd in other editors | User confusion | Keep attributes minimal; Pandoc-compatible syntax; the GUI never adds attributes that aren't needed |

## 13. Decisions

Settled (2026-09-19):

1. **Repository** `github.com/can3p/booker`; app ID `com.github.can3p.booker` for now — no domain needed.
2. **Signing accounts** (Apple Developer, Windows) are owned by you; set up during Wave 1.
3. **EPUB is in scope** — reflowable for flowing books, fixed-layout for composed ones (Wave 8).
4. **License: MIT.** Typst is Apache-2.0 and stays a dependency, so the build ships a third-party notices file; bundled fonts keep their own licences (OFL and similar).
5. **No git features inside the app for 1.0.** The format stays git-friendly and outside changes reload cleanly.
6. **Free placement is a first-class feature**, not an escape hatch (§5.6, Wave 5).
7. **`main` is protected by a repository ruleset** ("protect main"): no deletion, no force-push, a linear history, and every change arrives through a pull request whose `fmt`, `clippy`, `test (ubuntu-latest)`, `docs`, `bindings` and `deps` checks are green. It is a ruleset rather than the older branch-protection settings, so it is read with `gh api repos/can3p/booker/rules/branches/main`.
8. **A pull request into `main` merges by rebase**, the only method the ruleset allows, because a linear history is required. A wave therefore arrives on `main` as its own commits rather than as one squashed commit or a merge bubble, which is what lets the wave log point at them (§2).

Still open: tracked in `docs/OPEN-QUESTIONS.md`, each with the default we proceed with in the meantime. The ones outstanding today are the final app name and bundle identifier (must be settled before the first public release), the bundled font set, whether print targets need CMYK or PDF/X, spell-checking, and how shared templates are distributed.

## 14. Parallel development with git worktrees

The waves are built for it: each one is cut into tracks that barely touch each other, so several can run at once.

**Branches.** `main` ← `wave-N` (integration branch) ← `wN/<track>`. The wave's contracts (types, schemas, IPC) land on `wave-N` first and are frozen. Tracks branch off it, rebase on it daily, and merge back through a pull request with CI green. `main` only receives a wave merge at release time.

**Worktrees.**

```bash
git worktree add ../booker-wt/w0-release  -b w0/release  wave-0
git worktree add ../booker-wt/w0-engine   -b w0/engine   wave-0
git worktree list
git worktree remove ../booker-wt/w0-engine     # after the merge
```

Keep worktrees in a sibling directory (`../booker-wt/`), not inside the repo, so no tool ever scans them as project files.

**Keeping builds fast** (this is where the experiment usually hurts):

- Give each worktree its own `CARGO_TARGET_DIR` and turn on `sccache`. A single shared target directory looks tempting, but Cargo locks it, so parallel builds would queue up.
- Use `pnpm`, whose shared store makes `node_modules` per worktree cheap.
- Expect a few GB of build output per worktree; `cargo clean` between waves.

**Avoiding merge pain:**

- Each track owns a listed set of paths, written down in `docs/waves/wave-N.md` as part of the contracts commit. Two tracks never edit the same file.
- Shared files (workspace `Cargo.toml`, the IPC command registry, generated TypeScript types, CHANGELOG) are append-only lists and are changed only in the integration step.
- Golden test images are binary and merge badly: each track owns its own fixture folder, and the integration step regenerates and reviews them.
- Keep a track under roughly a day of work between merges. Long-lived branches are where parallel work falls apart.

**Running the tracks with Claude Code:** one agent per track, each in its own worktree (`isolation: "worktree"` when spawning an agent, or a dedicated session per worktree). Each brief states the goal, the owned paths, the contracts to respect and the exact command that proves the work is done. Small-model tickets run inside their track's worktree, so they don't add merges of their own.

The process rules themselves — branching, what counts as done, which document records what — live in `AGENTS.md`, so a fresh session follows them without being told.

**Measure it every wave:** how long integration took, how many conflicts appeared and where, recorded in `docs/WAVE-LOG.md`. If integration costs more than about a tenth of the wave, cut the number of parallel tracks rather than the contracts step — and a wave small enough that setting up worktrees costs more than it saves runs in one session instead.

## Sources
- [Typst 0.15 release notes](https://typst.app/blog/2026/typst-0.15/) · [Typst HTML export](https://typst.app/docs/reference/html/)
- [Tauri updater plugin changelog](https://v2.tauri.app/release/updater/all-versions/) · [tauri-plugin-updater](https://github.com/tauri-apps/tauri-plugin-updater/releases)
- [meander.typ: text wrapping around images](https://github.com/Vanille-N/meander.typ)
- [Paged.js on page floats](https://pagedjs.org/posts/en/paged-media-approaches-:-page-floats/) · [Vivliostyle supported CSS](https://docs.vivliostyle.org/en/reference/supported-css-features/)

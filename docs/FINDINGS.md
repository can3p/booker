# Findings

Things learned while building Booker that a future session would otherwise have to rediscover: how a dependency really behaves, a trap in the toolchain, a measurement, an approach that was tried and abandoned and why.

The test for including something: **would a session next month waste an hour without it?** Ordinary code context does not belong here — the code says that itself.

Format:

```
### <short title>
- Learned: YYYY-MM-DD, wave N / track X
- What: the fact, stated plainly
- Why it matters: what it changes about how we build
- Where: file paths, issue links, versions it applies to
```

---

### Typst has CMYK colours but no colour management
- Learned: 2026-09-19, planning research
- What: Typst can express CMYK colours, but has no ICC profile embedding, no PDF output intent and no PDF/X export. These are open upstream issues (typst/typst #3143, #3002).
- Why it matters: colour management has to be a post-processing step after Typst produces the PDF, behind a single export hook. Colour values must carry their colour space in our own model from the start, or retrofitting is painful.
- Where: `docs/PLAN.md` §10, Wave 9 track C. Applies to Typst 0.15.x.

### `meander` is MIT-licensed
- Learned: 2026-09-19, planning research
- What: the Typst package that wraps text around images and threads text between containers (`github.com/Vanille-N/meander.typ`, 0.4.4) is MIT.
- Why it matters: frame chains are core to Booker, so we must be able to vendor and, if needed, fork this package. MIT makes that safe, and our own licence is MIT too. Prefer contributing fixes upstream before forking.
- Where: `docs/PLAN.md` §6, Wave 0 spike D2.

### Typst 0.15 changed how blocks behave in HTML export
- Learned: 2026-09-19, planning research
- What: 0.15.0 aligned `box` and `block` between HTML and paged export — a breaking change — and added bundle export (one project, several output files) plus MathML for equations.
- Why it matters: when we pin or upgrade the Typst version, HTML-adjacent output can shift even if the PDF does not. The golden suite must cover both.
- Where: Typst 0.15 release notes; `docs/PLAN.md` §3.

### `deny_unknown_fields` and `flatten` cannot both be used, and preserving wins
- Learned: 2026-09-19, Wave 0 contracts
- What: serde rejects a struct that both denies unknown fields and captures them with `#[serde(flatten)]` into a map — the flattened map receives everything, so the deny rule fires first and parsing fails. The contract test caught it immediately.
- Why it matters: our rule is that an older Booker preserves keys a newer one wrote (`AGENTS.md` §7), so no project-file struct may use `deny_unknown_fields`. A misspelled key is therefore not a parse error; it is caught by a `format.*` diagnostic that suggests the right key, which serves the user better anyway.
- Where: `crates/booker-core/src/project.rs`, `BookConfig` and `PageConfig`.

### Toolchain on the development machine
- Learned: 2026-09-19, Wave 0
- What: rustup installed to `~/.cargo`, stable 1.98.1 (aarch64-apple-darwin). `~/.cargo/bin` is **not** on the PATH, because the installer was run with `--no-modify-path`; a shell needs `. "$HOME/.cargo/env"` or an explicit `PATH` export.
- Why it matters: a session that runs `cargo` without that will be told the command does not exist, and may wrongly conclude Rust is missing.
- Where: `CONTRIBUTING.md`.

### Incremental recompilation is the reason to keep one engine per open project
- Learned: 2026-09-19, wave 0 / track C
- What: measured on a generated 201-page A5 novel (aarch64-apple-darwin, release build): **cold compile 519–567 ms, recompile after a one-character edit 24–29 ms (about 20×), recompile with nothing changed 0 ms**. The same run in a debug build: cold 16.3 s, one-character edit 187 ms. Font loading at `Engine::open` is 0–3 ms in release because the bundled faces are `include_bytes!`. The saving comes entirely from keeping `comemo`'s memoized layout alive between compiles and from editing the parsed `Source` in place (`typst_kit::files::FileStore::reset` keeps the old tree and calls `Source::replace`, which reparses incrementally) — build a new `Engine` per keystroke and every compile is a cold one.
- Why it matters: it settles the preview design. A debounce of ~150 ms with one long-lived engine per project meets `PLAN.md` §6's "under 100 ms from keystroke to preview" for a 300-page novel, and the first render after opening a book is the only slow one. It also says the 200-page golden tests must be release-only: debug Typst is ~30× slower, which is why the measurement lives in an `#[ignore]`d test.
- Where: `crates/booker-typst/tests/incremental.rs` (`cargo test -p booker-typst --release --test incremental -- --ignored --nocapture`); `crates/booker-typst/src/engine.rs`. Typst 0.15.1.

### Typst identifies a file by its path *inside* a project, and the layout cache is global
- Learned: 2026-09-19, wave 0 / track C
- What: in Typst 0.15 a `FileId` interns a `RootedPath` = (`VirtualRoot::Project` | a package, virtual path). The real directory is **not** part of it, so `main.typ` in two different books is the same `FileId`, and `comemo`'s memoization cache is a process-wide global. Correctness is safe — comemo revalidates a cached entry by asking the current `World` for the files it depended on, so a second project's `main.typ` fails the check and is recompiled — but the two books evict each other's work. Interned ids are also leaked forever (capped at 65535 distinct paths per process, then a panic inside Typst).
- Why it matters: the app should keep one long-lived engine per open project and expect the incremental win to shrink when several books are open at once; it must not assume `FileId` identifies a project. If we ever open dozens of books in one process, the id budget is the thing that runs out.
- Where: `crates/booker-typst/src/world.rs`; test `two_projects_open_at_once_do_not_bleed_into_each_other` in `crates/booker-typst/tests/compile.rs`. Typst 0.15.1.

### `typst-kit` gives us the file-slot machinery without the system integration
- Learned: 2026-09-19, wave 0 / track C
- What: `typst-kit` 0.15.1 with `default-features = false` pulls in almost nothing (no network, no `fontdb`, no `dirs`) and still provides `files::FileStore` + the `FileLoader` trait, which is the fiddly part of a `World`: caching bytes and sources per file, and reusing a stale parsed `Source` across compiles so an edit reparses incrementally. Booker implements `FileLoader` itself — project root plus an overlay of unsaved editor buffers — which is also where reads are confined to the project and Typst packages are refused. The `datetime` feature (chrono) is worth taking for a correct local `today()`.
- Why it matters: it is a small, upstream-maintained piece of the integration we would otherwise get subtly wrong, and taking it does not drag in the CLI's font discovery or package downloading. Do not enable `system-files`, `system-packages` or `scan-fonts`: each would let a book depend on the machine it was written on.
- Where: `crates/booker-typst/src/world.rs`, workspace `Cargo.toml`. Typst/typst-kit 0.15.1.

### Bundled fonts, and why no system fonts
- Learned: 2026-09-19, wave 0 / track C
- What: Typst's defaults name "Libertinus Serif" for text and "DejaVu Sans Mono" for `raw`, so a document that configures nothing renders nothing unless those faces exist. Booker bundles them (copied from `typst-assets` 0.15.1, OFL and Bitstream Vera; licences in `crates/booker-typst/fonts/NOTICE.txt`, ~1.5 MB embedded with `include_bytes!`) and additionally loads a project's own `assets/fonts/**`, project first so a book's own cut of a family wins. System fonts are deliberately not searched.
- Why it matters: it is what makes "the same book on every machine" true, and it is the reason a missing font is a diagnostic about the *project* rather than about the user's computer. A font file that will not parse is a `BK-FONT-001` warning and the book still lays out.
- Where: `crates/booker-typst/src/fonts.rs`, `crates/booker-typst/fonts/`.

### `RenderRequest::scale` is CSS pixels, which is not Typst's unit
- Learned: 2026-09-19, wave 0 / track C
- What: `typst_render::render` takes `pixel_per_pt`, where a point is 1/72 inch, while the contract's `scale` is "pixels per CSS pixel" (1/96 inch) — a device pixel ratio, 2.0 on a retina screen. The conversion is `pixel_per_pt = scale * 96/72`. At scale 1 an A5 page comes out 559 × 794 px, which is the size the preview should lay out a page at before applying zoom.
- Why it matters: get the factor wrong and every preview is 33% off, which looks like a styling bug rather than a unit bug. SVG ignores the scale entirely: it is resolution independent, and the viewport carries the size.
- Where: `crates/booker-typst/src/engine.rs` (`POINTS_PER_CSS_PIXEL`), `crates/booker-typst/tests/render.rs`.
### Markdown parser: pulldown-cmark, decided on source positions
- Learned: 2026-09-19, Wave 0 / track E
- What: both candidates were measured on one criterion — does every node carry a byte range that slices back out of the source to the text that produced it. On a document with headings, paragraphs, emphasis, inline code, quotes, nested lists, images, a fenced block and a setext heading: `pulldown-cmark` 0.13.4 produced 70 events, **0** without a usable range; `markdown-rs` (`markdown` 1.0.0) produced 48 mdast nodes, **0** without a `Position`. Both are good; three differences decided it. (1) **Granularity.** `markdown-rs` emits one `Text` node per paragraph, and its span covers the block markers: in a blockquote the node for `A quote that spans\ntwo source lines.` spans `80..118`, which includes the `> ` prefixes, so slicing the span does not give the text. `pulldown-cmark` splits text at soft breaks with exact ranges that exclude the markers, which is what click-to-source needs. (2) **Optionality.** `markdown-rs` positions are `Option<Position>`, so every consumer must handle absence; `into_offset_iter()` gives a non-optional `Range<usize>`. (3) **Booker's own syntax.** With `ENABLE_HEADING_ATTRIBUTES`, `pulldown-cmark` already parses `# Head {#garden .fancy}` into id/classes/attrs with the heading's range intact; `markdown-rs` hands that back as the literal text `Head {#garden .fancy}` (and its `Constructs` list has no directive or attribute construct at all, so `::: {.poem}` is a plain paragraph there too), meaning we would have to strip it and recompute positions by hand in Wave 1.
- Why it matters: click-to-source and every future diagnostic that points at a line depend on these ranges, so this is the load-bearing choice in `booker-doc`. It also means Wave 1's attribute pass is a pass over an event stream with offsets rather than a tree rewrite with hand-maintained positions. Cost of the choice: `pulldown-cmark` gives byte offsets only, so line and column are ours to compute — `booker_doc::LineIndex` does it, counting columns in characters, and `book.toml` needs the same thing anyway because `toml_edit` also reports byte ranges.
- Where: `crates/booker-doc/src/parse.rs`, `crates/booker-doc/src/span.rs`, tests in `crates/booker-doc/tests/spans.rs`. Applies to `pulldown-cmark` 0.13.4 and `markdown` 1.0.0.

### `toml_edit` throws every span away when a document becomes editable
- Learned: 2026-09-19, Wave 0 / track E
- What: `ImDocument::parse` records a byte span on every key, value and table (`Item::span()`, `Value::span()`, `Key::span()`, including values inside inline tables). `ImDocument::into_mut()` — which is what `"…".parse::<DocumentMut>()` does — returns them all as `None`. Verified: `title` has `Some(8..13)` through `ImDocument` and `None` through `DocumentMut` on the same text.
- Why it matters: it is the difference between "`book.toml:17:9`: `bleed` is not a measurement" and a diagnostic with no location, and it is silent — the code compiles and the spans are simply gone. Anything that must both *report on* and *write* a TOML file has to keep two parses of the same text: the immutable one for reading and locating, the mutable one for editing. `booker-project` does exactly that (`ConfigFile { spanned: ImDocument<String>, document: DocumentMut }`) and re-parses the immutable view after an edit so later diagnostics point at the pending text. Spans inside inline tables survive on the immutable view, so `margins = { top = "…" }` locates as precisely as `[page.margins]`.
- Where: `crates/booker-project/src/config.rs`, `crates/booker-project/src/toml_tree.rs`. Applies to `toml_edit` 0.22.27.

### ts-rs silently ignores `#[serde(flatten)]`
- Learned: 2026-09-19, Wave 0 integration
- What: generating the TypeScript bindings prints `ts-rs failed to parse this attribute. It will be ignored.` for the `#[serde(flatten)]` fields on `BookConfig` and `PageConfig`. It is a warning, not an error, and the binding is still written — with `extra` as a named field, which is not how the TOML is shaped.
- Why it matters: the generated types are the app's view of the contracts. Anything relying on `extra` in the UI would be wrong in a way the compiler cannot catch. Either keep unknown keys out of the UI's model, or override the type by hand with `#[ts(type = "...")]` on the struct.
- Where: `crates/booker-core/src/project.rs`; warning appears in any `cargo test -p booker-core export_bindings` run.

### `cargo deny` needs three specific settings before it is useful on this tree
- Learned: 2026-09-19, Wave 0.5 / track C
- What: with a default policy, `cargo deny check` failed on things nobody can act on. Three settings fixed it, and each one is a decision rather than a workaround. (1) `[advisories] unmaintained = "workspace"` — the default flags every unmaintained crate anywhere in the graph, which here means `rustybuzz`, `ttf-parser`, `paste`, `bincode` and `yaml-rust`, all of them Typst's dependencies and none of them ours to fix; scoping the check to crates we depend on directly is the difference between a job people read and a job people ignore. (2) `[bans] allow-wildcard-paths = true` **plus `publish = false` on every crate in the workspace** — a path dependency has version `*`, so the wildcard check fires on our own crates, and `allow-wildcard-paths` deliberately does not apply to crates that look publishable, because crates.io forbids path dependencies. The flag alone does nothing; the crates have to say they are not published. (3) The licence allow list has to be exactly the licences present — listing one that no crate uses is a `license-not-encountered` warning on every run.
- Why it matters: a dependency policy that is red for reasons outside our control gets ignored within a week, and then it is not a policy. The same three questions will come back when the application crates arrive in Wave 1 and when Typst is next bumped.
- Where: `deny.toml`, `crates/*/Cargo.toml` (`publish = false`). Applies to `cargo-deny` 0.20.2.

### Two live quick-xml advisories come in through Typst's bibliography support and cannot be updated away
- Learned: 2026-09-19, Wave 0.5 / track C
- What: RUSTSEC-2026-0194 and RUSTSEC-2026-0195 are denial-of-service advisories against `quick-xml` 0.38.4 — quadratic time on duplicate attribute names, and unbounded namespace allocation in `NsReader`. The path is `typst-library → hayagriva → citationberg → quick-xml`, and 0.38.4 is the newest version citationberg's semver range accepts, so `cargo update` changes nothing; only a Typst upgrade can move it. In Booker the parser only ever reads CSL style files out of the user's own project, so the attack is a user feeding themselves a malicious style file.
- Why it matters: they are in `deny.toml`'s `ignore` list with that reasoning written next to them, and they must be re-checked whenever the pinned Typst version moves — `Cargo.toml` already says to bump the five Typst crates together, and this is one more thing to do at the same time.
- Where: `deny.toml` (`[advisories] ignore`). Applies to Typst 0.15.1 / `quick-xml` 0.38.4.

### Booker prints project-relative paths with forward slashes on every platform
- Learned: 2026-09-19, Wave 0.5 / track B
- What: the first Windows run of the Wave 0 suite failed on one assertion — `booker build` printed `build\the-secret-garden.pdf` where the test expected `build/the-secret-garden.pdf`. The interesting part is that the same line of output mixed conventions: the chapter list showed `content/01-the-first-chapter.md` with a forward slash, because that path comes out of `book.toml` as the user wrote it, while the built path was assembled with `Path::join` and so carried the platform separator. The fix is a single canonical form, `booker_project::display_path`, used by the CLI's chapter list and build line and by `format_diagnostic`; absolute paths are still shown the way the platform writes them, because inventing a form for those would be worse than showing the real one.
- Why it matters: `book.toml` spells chapter paths with `/` whoever wrote it, so a message saying `content\01.md` points at a file the user cannot find by that name; and `AGENTS.md` §6 asks for output an agent can compare across platforms — with the MCP server in Wave 7 checked against the CLI's own output, one file has to print as one string. Every future place that shows a path should use `display_path` rather than `Path::display`, and the Wave 7 conformance suite should assert it.
- Where: `crates/booker-core/src/path.rs` (`display_path`, with its unit tests), re-exported by `booker-project`; used in `crates/booker-project/src/content.rs`, `crates/booker-cli/src/lib.rs` and `crates/booker-typst/src/fonts.rs`. The helper started in `booker-project` and had to move to `booker-core` one run later, when Windows found the same bug in a font diagnostic built by `booker-typst` — which must not depend on `booker-project` (`AGENTS.md` §6, crate boundaries). Anything shared by the crates that *build* diagnostics belongs beside `SourceLocation`, not beside the project loader. Found by `test (windows-latest)` in https://github.com/can3p/booker/actions/runs/35451247935 and https://github.com/can3p/booker/actions/runs/35452064970.

### What the first CI run cost, per platform
- Learned: 2026-09-19, Wave 0.5 / track A
- What: cold, with no cache at all, on the pull-request matrix: `fmt` 5s, `bindings` 27s, `deps` 40s (including `cargo deny` fetching the advisory database), `clippy` 1m15s, `docs` 1m20s, and the test job 2m10s on Linux, 3m46s on macOS, 5m06s on Windows. The whole run is bounded by the slowest test job, so a pull request into `main` costs about five minutes of wall clock and roughly fifteen minutes of runner time cold.
- Why it matters: it sets what Wave 1's matrix can afford. The three-platform test job is the expensive half of CI and it only runs on pull requests into `main`; adding the Tauri bundle steps to the same trigger would multiply that, which is why the release workflow is a separate file on a tag trigger.
- Where: https://github.com/can3p/booker/actions/runs/35451247935. GitHub-hosted runners, `ubuntu-latest`, `macos-latest`, `windows-latest`, Typst 0.15.1.

### A path beginning with `/` is not absolute on Windows
- Learned: 2026-09-19, Wave 0.5 / track B
- What: `crates/booker-typst/src/world.rs` refuses paths that climb out of the project, and its tests used `/books/mia` as the root and `/books/mia/content/01.typ` as the file. On Windows, a path that starts with `/` and names no drive is *drive-relative*, not absolute: `Path::is_absolute` returns false, so the file took the relative branch of `virtual_path`, hit a `RootDir` component and was refused — the test asserted the opposite of what it was testing, and only on one platform. The tests now build the root and the absolute file with a small `absolute()` helper that spells them `C:\books\mia` on Windows.
- Why it matters: any test that turns on whether a path is absolute — and Booker has several, because refusing paths outside the project is a security boundary — is a platform test in disguise. The production code was right; only the tests were wrong. Write path fixtures with `PathBuf::join` and a platform-aware root, never as Unix string literals, and expect the same trap in Wave 7's "refusals for paths outside the project" conformance tests.
- Where: `crates/booker-typst/src/world.rs` (`virtual_path` and its tests). Found by `test (windows-latest)`, https://github.com/can3p/booker/actions/runs/35451658819.

### A test can prove a tutorial's commands, never its sentences
- Learned: 2026-09-20, Wave 0.6 / tracks A and B
- What: `crates/booker-cli/tests/tutorials.rs` replays every ```console block and passed on the first run, and the tutorial was still wrong. The paragraph after the first real build said "the paragraphs are indented after the first one". They are not — Booker's default separates paragraphs with space and indents nothing. Nothing in the harness could have caught it, because it is a claim about the PDF rather than about anything printed. It was found by rendering the two pages to PNG (`pdftoppm -png -r 110`) and looking at them, which also confirmed what *is* true and was worth writing down instead: justified lines, hyphenation that depends on `language`, and the `inside` margin swapping sides between page 1 and page 2.
- Why it matters: the harness checks the transcripts; a person has to check the prose, and the cheapest way to do that is to render the output and look. Every wave that touches a tutorial should render what the tutorial tells the reader to build and compare it with what the tutorial says they will see — this is the same instinct as the golden tests (`AGENTS.md` §6), applied to documentation. Expect it to matter much more from Wave 2 on, when styles arrive and there is far more on the page to describe.
- Where: `docs/tutorials/01-your-first-book.md` §5, `crates/booker-cli/tests/tutorials.rs`.

### The tutorial harness found the Wave 0.5 path bug in the one command Wave 0.5 did not check
- Learned: 2026-09-20, Wave 0.6 / track B
- What: the first three-platform run of the new tutorial test went red on Windows only, at `docs/tutorials/01-your-first-book.md:56` — `booker new` listed `content\01-the-first-chapter.md` where the tutorial, and `build` three lines later, say `content/01-the-first-chapter.md`. Wave 0.5 found exactly this class of bug in `build` and in a font diagnostic, wrote `display_path` for it, and concluded that "every future place that shows a path should use `display_path` rather than `Path::display`". `new` was simply not one of the places anyone looked: its output had no assertion about separators, on any platform.
- Why it matters: a rule of the form "always use X" does not enforce itself — the only thing that found this was a test that compares whole lines of real output on all three platforms. Two lessons follow. First, the tutorials are now a cross-platform output contract for every command they walk through, which is a second reason to keep them covering everything Booker can do. Second, when a bug class is found and a helper is written for it, the same change should go looking for every other caller rather than fixing the one that was red: `grep -n "\.display()" crates/*/src/*.rs` would have found this one in Wave 0.5.
- Where: `crates/booker-cli/src/lib.rs` (`new`), with the assertion that states the intent in `crates/booker-cli/tests/commands.rs`. Found by `test (windows-latest)` in https://github.com/can3p/booker/actions/runs/35477784823.

### The tutorial harness runs the binary, not `booker_cli::run`
- Learned: 2026-09-20, Wave 0.6 / track B
- What: `tests/commands.rs` calls `booker_cli::run` in process with a `tempfile` path, which is fast and right for a unit-level check. The tutorial harness cannot: a tutorial quotes `Created a `novel` book in the-moon-jar` — a *relative* path, as the reader's shell shows it — and `echo $?` reporting the process exit code. Running in process would mean rewriting the path arguments, which changes exactly the text being asserted. It spawns `env!("CARGO_BIN_EXE_booker")` with `current_dir` set instead; Cargo builds that binary for the crate's integration tests anyway, so the only cost is a process spawn per command, and the whole suite takes under two seconds.
- Why it matters: any future check of what Booker *prints* — the Wave 7 MCP conformance suite compares the server's output with the CLI's for the same question — has the same constraint. Compare the real process's bytes, or you are comparing something the user never sees.
- Where: `crates/booker-cli/tests/tutorials.rs`, contrasted with `crates/booker-cli/tests/commands.rs`.

### Build durations vary by an order of magnitude between runs
- Learned: 2026-09-20, Wave 0.6 / track A
- What: the same two-chapter book built in 30 ms and then 2 ms on consecutive runs of the same release binary; the debug binary the test suite uses took 94 ms, then 32 ms, then 32 ms. Nothing about the project changed; it is Typst's caches warming and the font load.
- Why it matters: it is why the tutorial convention has a `…` wildcard at all, and why the wildcard must stay narrowly scoped — `2 pages in … ms` is legitimate, `… pages in … ms` would hide a real regression. The same caution applies to any future benchmark assertion: a single timing is not a measurement.
- Where: `docs/tutorials/index.md` (the convention), `crates/booker-cli/tests/tutorials.rs` (`matches`).

### Appending a key to `book.toml` puts it in the last table, not at the top level
- Learned: 2026-09-20, Wave 0.6 / track A
- What: writing the tutorial's deliberate typo by appending `authorr = "Mia"` to the end of a generated `book.toml` produced `warning[BK-FORMAT-005] unknown key `page.margins.authorr`` — correct TOML (the key lands inside the last `[table]` header, which is `[page.margins]`) and thoroughly confusing to a beginner who thinks they added a top-level key. The tutorial puts its typo on line 5, next to `author`, and shows the whole file so the reader cannot get it wrong.
- Why it matters: it is a real trap for both people and agents editing `book.toml` by appending, and the diagnostic is doing nothing wrong — it names the key it actually found. If unknown-key suggestions ever get smarter (`AGENTS.md` §6, "did you mean …"), a key whose leaf name matches a known *top-level* key while sitting in a table is worth a better message than the generic one.
- Where: `crates/booker-project/` (BK-FORMAT-005), `docs/tutorials/01-your-first-book.md` §7.

### `booker new --template <unknown>` names the path as `.`
- Learned: 2026-09-20, Wave 0.6 / track A
- What: `booker new the-moon-jar --template kidsbook` reports `booker: .: unknown template `kidsbook`; this build has: novel`. The message, its suggestion and its exit code are all right; only the location is wrong — the template is not resolved against a path, so the error is built with `.` where every other message names the file it came from.
- Why it matters: `AGENTS.md` §6 says every user-visible error names the file it came from, and this one names a file that has nothing to do with the mistake. Not fixed in Wave 0.6 because the tutorial does not walk the reader into it and the wave was documentation; it is a small, self-contained fix for whoever next touches `Template::from_name`.
- Where: `crates/booker-project/src/template.rs`, reached from `crates/booker-cli/src/lib.rs` `new`.

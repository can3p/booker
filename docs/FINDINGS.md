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

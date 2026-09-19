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

# Wave 0 — The core: engine, format, command line ✅ complete

Branch: `wave-0`, merged into `main`. What actually shipped is in `docs/WAVE-LOG.md`.

**Scope changed during the wave:** the application shell, the release pipeline and repository hygiene (originally tracks A, B and F here) turned out to be independent of the core and became their own wave — see `docs/waves/wave-1.md`. What remained was the engine (C), the format and CLI (E), and a bridge between them.

Read `AGENTS.md` before starting. This brief assigns the tracks, the paths each one owns, and what "done" means for each.

## Goal

**Demo at the end of the wave:** install Booker on macOS, Windows or Linux; open a folder containing a `book.md`; see the text laid out as pages in the preview; export a PDF. Then publish `v0.1.1` and watch the installed `v0.1.0` update itself.

Nothing about styles, frames or anchors ships here. What ships is the spine: a workspace, an app that compiles and renders a document, and a release pipeline that updates itself. Plus three spikes whose answers shape Waves 1–5.

## Repository layout this wave creates

```
crates/
  booker-project/   project load/save/watch, book.toml, migrations
  booker-doc/       Markdown → document model (skeleton only this wave)
  booker-typst/     Typst World, fonts, compile, render to PNG/SVG
  booker-cli/       booker new | build
app/
  src-tauri/        Tauri 2 shell, IPC, booker:// protocol
  src/              Svelte 5 + TypeScript UI
fixtures/           shared test projects
xtask/              development tasks (golden-test regeneration, later the eval harness)
docs/               plan, log, findings, open questions, wave briefs
.github/workflows/  ci.yml, release.yml
```

## Contracts (land on `wave-0` before tracks start — owned by the wave lead)

1. `ProjectRef`, `BookConfig` (the `book.toml` shape for this wave: `format`, `title`, `author`, `language`, `chapters`, `[page]`), and the error type used across crates.
2. `CompileRequest` / `CompileResult` (pages, page sizes, diagnostics list — the diagnostics shape is a stub now but the type exists), and `RenderRequest` (page index, scale).
3. The IPC command list and its TypeScript generation (`ts-rs`), plus the `booker://` URL shape for page images.
4. `CARGO_TARGET_DIR` / `sccache` and `pnpm` setup documented in `CONTRIBUTING.md`.

## Tracks

Each track works in its own worktree (`../booker-wt/w0-<track>`) on branch `w0/<track>`, and merges into `wave-0` by pull request.

### A. Release pipeline — tier M (YAML tickets: S)
**Owns:** `.github/workflows/**`, `app/src-tauri/tauri.conf.json` (updater section), release documentation.
Build matrix for macOS (arm64 + x64), Windows x64, Linux x64 (AppImage, deb, rpm) with `tauri-action`. Updater plugin configured with `createUpdaterArtifacts`, minisign keys in repository secrets, `latest.json` published as a release asset, a `beta` channel endpoint. Signing hooks in place and used when the owner's certificates are available, skipped cleanly when not.
**Done when:** a tagged push produces installers for all three platforms and a `latest.json`, and the updater path is proven by installing `v0.1.0` and updating it to `v0.1.1`.

### B. Application shell — tier M
**Owns:** `app/src/**`, `app/src-tauri/src/main.rs` and window setup.
Tauri 2 + Svelte 5 + TypeScript scaffold. Window layout: chapter sidebar, editor pane, preview pane, status bar, problems panel (empty shell). Open-folder dialog, recent projects, menu with "Check for updates…".
**Done when:** the app opens a folder and shows its files; the layout survives resizing down to a small window; `pnpm lint` and `pnpm test` pass.

### C. Typst engine — tier L
**Owns:** `crates/booker-typst/**`.
Embed Typst: a `World` implementation over in-memory and on-disk files, font loading (bundled plus project `assets/fonts`), compile to PDF, render a page to PNG and SVG. Incremental recompile on change. Expose compile and render behind the Wave 0 contracts.
**Done when:** a fixture `book.md` compiles to a PDF and to page images; recompiling after a one-character edit in a 200-page fixture is measurably faster than a cold compile, and the number is written into `docs/FINDINGS.md`.

### D. Spike — anchored images — tier L
**Owns:** `crates/booker-typst/tests/spike_anchor.rs`, `docs/FINDINGS.md`.
Demonstrate that "place this image on the same / previous / next page as this phrase" converges, using Typst introspection across repeated compilations: compile, query where the anchor landed, re-emit with the float placed, repeat. Try it on three documents, including one deliberately unstable case.
**Done when:** a written go/no-go in `docs/FINDINGS.md` — does it converge, in how many passes, what does non-convergence look like, and what do we tell the user when it fails.

### D2. Spike — frames and chains — tier L
**Owns:** `crates/booker-typst/tests/spike_frames.rs`, `docs/FINDINGS.md`.
Absolutely placed frames on a page; measuring whether content fits (`measure`); shrink-to-fit; **text continuing from one frame into the next** using `meander` (MIT, vendor it for the spike); and a coordinate round-trip — change a number in a file, see the frame move, and confirm the file diff is one line.
**Done when:** a written recommendation in `docs/FINDINGS.md`: use the package, fork it, or write our own chain layout — with the reasoning and the limits found.

### E. Project format and CLI — tier L for the spec, M for the code
**Owns:** `crates/booker-project/**`, `crates/booker-doc/**`, `crates/booker-cli/**`, `fixtures/**`.
`book.toml` parsing and writing with `toml_edit` (comments and order preserved, unknown keys kept), the format version and the migration hook, `booker new` generating a starter project, `booker build` producing a PDF. A Markdown parser spike deciding between `markdown-rs` and `pulldown-cmark` on one criterion: **reliable source positions**, since click-to-source and every future diagnostic depends on them. Fixtures: `fixtures/minimal`, `fixtures/novel-200p`.
**Done when:** `booker new` then `booker build` produces a PDF from a fresh project; the parser decision is recorded in `docs/FINDINGS.md`; round-trip tests prove a file with comments and unknown keys survives a write untouched.

### F. Repository hygiene — tier S
**Owns:** `LICENSE`, `README.md`, `CONTRIBUTING.md`, `THIRD-PARTY.md`, `.editorconfig`, lint configuration, `.github/ISSUE_TEMPLATE/**`, app icons.
MIT licence, third-party notices (Typst is Apache-2.0; check every bundled font and Typst package), `rustfmt.toml`, `clippy.toml`, ESLint and Prettier, editor config, issue templates, icon set wired into the bundle.
**Done when:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `pnpm lint` all pass in CI on a clean checkout.

## Small-model tickets (tier S, run inside their track's worktree)

- CI YAML for fmt/clippy/test (track A)
- Issue and pull-request templates (F)
- `.editorconfig`, `rustfmt.toml`, `clippy.toml`, ESLint and Prettier config (F)
- Icon set at every required size, wired into `tauri.conf.json` (F)
- `fixtures/novel-200p` generated from public-domain text (E)
- `CONTRIBUTING.md` covering worktrees, `CARGO_TARGET_DIR` and `sccache` (F, from `AGENTS.md` §5)
- Third-party notice file assembled from `cargo license` output (F)

## Order of work

1. Wave lead: create `wave-0`, land the contracts and the empty workspace.
2. A, B, C, E, F start in parallel. D and D2 start as soon as C can compile anything.
3. Integration on `wave-0`: wire the app to the engine through the IPC contracts, run the golden suite on all three platforms.
4. Tag `v0.1.0`, publish, then tag a trivial `v0.1.1` and prove the update path.
5. Write the `docs/WAVE-LOG.md` entry, answer or re-file anything in `docs/OPEN-QUESTIONS.md`, open the pull request into `main`.

## Exit criteria for the wave

- Installers for macOS, Windows and Linux, produced by CI.
- **A `v0.1.0` installation updates itself to `v0.1.1`.**
- The app opens a folder, renders its pages, exports a PDF.
- Three spikes answered in writing in `docs/FINDINGS.md` (anchors, frames and chains, Markdown parser).
- `docs/WAVE-LOG.md` entry written; documents from `AGENTS.md` §2 accurate.

## What is deliberately not in this wave

Styles, page rules, frames, anchors, diagnostics beyond "this file is broken", HTML and EPUB output, the agent surface. Each has its own wave in `docs/PLAN.md` §8.

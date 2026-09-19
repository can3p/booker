# Wave 1 — The application and how it reaches people

Branch: `wave-1` (off `main`) · Finishes as one pull request into `main`, tagged `v0.2.0`

Read `AGENTS.md` first. This brief assigns the tracks, the paths each owns, and what done means.

## Goal

**Demo at the end of the wave:** download Booker, install it on macOS, Windows or Linux, open a folder containing a book, see its pages, export a PDF. Then publish the next version and watch the installed one update itself.

Wave 0 built the core: `booker build` already turns a folder of Markdown into a PDF, in about half a second for a 200-page novel and 25 ms after an edit. This wave puts a window around it and a way to deliver it.

## What exists already

- `crates/booker-core` — the contracts (geometry, config, diagnostics, compile and render requests, the IPC command list). **Frozen for this wave**; report problems, do not edit.
- `crates/booker-typst` — `Engine::open(project)`, `set_source`, `compile`, `render` (PNG and SVG, scale in CSS pixels).
- `crates/booker-project`, `crates/booker-doc`, `crates/booker-cli` — loading, parsing, `booker new`, `booker build`.
- `booker_core::ipc::page_image_url` — the `booker://page/<revision>/<page>@<scale>x.png` shape the preview should use.

## Tracks

Each track works in its own worktree (`../booker-wt/w1-<track>`) on branch `w1/<track>`, and merges into `wave-1` by pull request.

### A. Release pipeline — tier M (YAML tickets: S)
**Owns:** `.github/workflows/**`, `app/src-tauri/tauri.conf.json` (bundle and updater sections), release documentation.
Build matrix for macOS (arm64 and x64), Windows x64, Linux x64 (AppImage, deb, rpm) with `tauri-action`. Updater plugin with `createUpdaterArtifacts`, minisign keys in repository secrets, `latest.json` published as a release asset, a beta channel. Signing used when the owner's certificates are present, skipped cleanly when not.
**Done when:** a tagged push produces installers for all three platforms and a `latest.json`, and an installed `v0.2.0` updates itself to `v0.2.1`.

### B. Application shell — tier M
**Owns:** `app/src/**` except the preview, `app/src-tauri/src/**`.
Tauri 2 + Svelte 5 + TypeScript. Window layout: chapter sidebar, editor pane, preview pane, status bar, problems panel. Open-folder dialog, recent projects, menu including "Check for updates…". IPC commands wired to the core through the generated TypeScript bindings (`cargo test -p booker-core export_bindings`).
**Done when:** the app opens a project folder, lists its chapters, shows its diagnostics, and exports a PDF through the same core the CLI uses.

### C. Preview — tier M
**Owns:** `app/src/lib/preview/**`, and the `booker://` protocol handler in `app/src-tauri/src/protocol.rs`.
Page images served over the custom protocol rather than through IPC (base64 through a JSON channel is the slowest thing in the app). Render only visible pages, zoom, scroll, and re-render on change. The revision belongs in the URL so images cache forever and a stale one is never mistaken for current.
**Done when:** scrolling a 200-page book is smooth, and an edit to a file on disk is visible in the preview.

### D. Hygiene — tier S
**Owns:** `THIRD-PARTY.md`, `.github/ISSUE_TEMPLATE/**`, `.githooks/**`, icons, CI lint steps.
Third-party notices (Typst is Apache-2.0; the bundled font licences are in `crates/booker-typst/fonts/NOTICE.txt`). **A `cargo metadata` smoke check in CI** — Wave 0's integration merged two tracks' dependency lines cleanly and produced a duplicate key, which no textual merge could catch. A pre-push hook refusing direct pushes to `main`. Icons at every required size.
**Done when:** CI fails on a duplicate dependency, and a direct push to `main` is refused locally.

## Order of work

1. Wave lead: create `wave-1`, add the Tauri scaffold and the IPC command wiring, freeze it.
2. A, B, D start in parallel; C starts once B's window exists.
3. Integration on `wave-1`: full suite on all three platforms, then tag `v0.2.0`, then a trivial `v0.2.1` to prove the update path.
4. Write the `docs/WAVE-LOG.md` entry, update `docs/OPEN-QUESTIONS.md`, open the pull request into `main`.

## Exit criteria

- Installers for macOS, Windows and Linux, produced by CI.
- **A `v0.2.0` installation updates itself to `v0.2.1`.**
- The app opens a project, shows its pages, exports a PDF.
- `docs/WAVE-LOG.md` entry written; the documents in `AGENTS.md` §2 accurate.

## Not in this wave

The editor is a plain text pane here; live Markdown styling, the chapter tree that edits `book.toml`, and templates are Wave 2. Styles, page rules and frames come later still — see `docs/PLAN.md` §8.

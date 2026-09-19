# Wave 1 — The application and how it reaches people

Branch: `wave-1` (off `main`, or off `wave-0.5` if that is not merged yet — check first, `AGENTS.md` §3) · Finishes as one pull request into `main`, tagged `v0.2.0`

Read `AGENTS.md` first. This brief assigns the tracks, the paths each owns, and what done means.

## Goal

**Demo at the end of the wave:** download Booker, install it on macOS, Windows or Linux, open a folder containing a book, see its pages, export a PDF. Then publish the next version and watch the installed one update itself.

Wave 0 built the core: `booker build` already turns a folder of Markdown into a PDF, in about half a second for a 200-page novel and 25 ms after an edit. Wave 0.5 put CI around it. This wave puts a window around it and a way to deliver it.

## What exists already

- `crates/booker-core` — the contracts (geometry, config, diagnostics, compile and render requests, the IPC command list). **Frozen for this wave**; report problems, do not edit.
- `crates/booker-typst` — `Engine::open(project)`, `set_source`, `compile`, `render` (PNG and SVG, scale in CSS pixels).
- `crates/booker-project`, `crates/booker-doc`, `crates/booker-cli` — loading, parsing, `booker new`, `booker build`.
- `booker_core::ipc::page_image_url` — the `booker://page/<revision>/<page>@<scale>x.png` shape the preview should use.
- `.github/workflows/ci.yml` from Wave 0.5 — fmt, clippy, tests on three platforms, doc build, bindings, `cargo metadata` and `cargo deny`. **Add jobs to it, do not start a second workflow**; the release workflow (track A) is the one new file. The suite is known to pass on macOS, Linux and Windows, so a failure on one platform only is this wave's bug, not a mystery.
- `.githooks/pre-push`, refusing direct pushes to `main` once `git config core.hooksPath .githooks` has been run (`CONTRIBUTING.md`).

## Tracks

Each track works in its own worktree (`../booker-wt/w1-<track>`) on branch `w1/<track>`, and merges into `wave-1` by pull request.

### A. Release pipeline — tier M (YAML tickets: S)
**Owns:** `.github/workflows/release.yml` (a new file — `ci.yml` belongs to Wave 0.5 and track D touches it for the frontend jobs), `app/src-tauri/tauri.conf.json` (bundle and updater sections), release documentation.
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

### D2. The application's tutorial — tier S
**Owns:** `docs/tutorials/02-the-app.md`, `docs/tutorials/index.md`.
Everything Wave 0.6's tutorial does from the terminal, done from the window instead: install Booker, open a book folder, read the pages in the preview, export a PDF, and update to the next version. Same rules as Wave 0.6 track A — a real book, every step, what the screen actually says — and the same check: whatever can be executed is executed by the tutorial harness, and the rest is walked through by hand before the wave closes.
**Done when:** someone who installed Booker from an installer can learn the app from `docs/tutorials/`, and `AGENTS.md` §4 criterion 3 is true of everything this wave added.

### D. Hygiene — tier S
**Owns:** `THIRD-PARTY.md`, icons, the frontend jobs in `.github/workflows/ci.yml`.
Third-party notices (Typst is Apache-2.0; the bundled font licences are in `crates/booker-typst/fonts/NOTICE.txt`), assembled from the licence data `cargo deny` already produces rather than by hand. Icons at every required size, wired into `tauri.conf.json`. `pnpm lint` and `pnpm test` added as jobs in the Wave 0.5 workflow, gated on the app directory so a Rust-only change does not pay for a Node install.
**Done when:** `THIRD-PARTY.md` covers every bundled dependency and font, the icons appear in the built bundle, and a pull request that breaks `pnpm lint` is red.

*(The `cargo metadata` smoke check and the pre-push hook that were listed here moved to Wave 0.5: they guard a four-track merge, so they have to exist before this wave rather than inside it.)*

## Order of work

1. Wave lead: create `wave-1`, add the Tauri scaffold and the IPC command wiring, freeze it.
2. A, B, D start in parallel; C starts once B's window exists; D2 is written last, against the application that actually shipped.
3. Integration on `wave-1`: full suite on all three platforms, then tag `v0.2.0`, then a trivial `v0.2.1` to prove the update path.
4. Write the `docs/WAVE-LOG.md` entry, update `docs/OPEN-QUESTIONS.md`, open the pull request into `main`.

## Exit criteria

- Installers for macOS, Windows and Linux, produced by CI.
- **A `v0.2.0` installation updates itself to `v0.2.1`.**
- The app opens a project, shows its pages, exports a PDF.
- The application can be learned from `docs/tutorials/` (`AGENTS.md` §4 criterion 3).
- `docs/WAVE-LOG.md` entry written; the documents in `AGENTS.md` §2 accurate.

## Not in this wave

The editor is a plain text pane here; live Markdown styling, the chapter tree that edits `book.toml`, and templates are Wave 2. Styles, page rules and frames come later still — see `docs/PLAN.md` §8.

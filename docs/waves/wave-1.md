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

## What the contracts step already did

Done on `wave-1` before any track starts, and **frozen**: a track that needs a change to any
of it says so rather than editing it (`AGENTS.md` §3).

- **`booker_core::ipc` is the contract.** Ten commands, one event (`project-changed`), and
  the payload types — `ProjectInfo`, `ChapterSummary`, `ChapterText`, `ExportRequest`,
  `ProjectChanged` — exported to TypeScript. `app/src-tauri/src/commands.rs` carries a test
  listing the commands still unimplemented, so the gap between the contract and the code is
  visible rather than remembered.
- **There is one translation from a book to pages.** It was `bridge.rs` inside `booker-cli`,
  which the app must not depend on; it is now `booker_typst::book`, reached through
  `Engine::set_book(config, chapters)`. Both the window and the CLI call it. Likewise
  `Project::chapter_summaries` and `Project::info` produce the sidebar's contents *and* what
  `booker build` prints.
- **The scaffold exists and runs**: Tauri 2, Svelte 5, TypeScript, Vite 8, `pnpm tauri dev`
  opens a window that opens a folder and lists what is in it. `app/src-tauri/src/session.rs`
  holds one engine per open project, which is what makes the preview incremental.
- **CI still passes.** `app/src-tauri` is a workspace member, so every Rust job that compiles
  the workspace now installs Linux desktop libraries first, from
  `.github/actions/tauri-system-deps`. Two licences arrived with Tauri (MPL-2.0 and ISC) and
  are allowed in `deny.toml` with the reasoning written next to them.
- **The updater keypair exists.** The public half is in `tauri.conf.json`; the private half
  and its password are repository secrets (`TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). Track A does not need to generate anything.

Two decisions the tracks inherit:

- **Signing is off.** There is no Apple Developer ID and no Windows certificate yet, so the
  release workflow wires the hooks and skips them cleanly. Gatekeeper and SmartScreen will
  warn on install; the updater is unaffected, because it uses the minisign key above.
- **A minimal watcher is in scope, in track C.** Track C's done criterion needs an edit made
  outside the app to reach the preview, so one coalesced reload per burst of changes lands
  here. Conflict handling — an outside edit meeting an unsaved buffer — stays in Wave 2
  track E, and the stress tests stay in Wave 7 track E. `docs/PLAN.md` §8 was updated to
  match.

## Tracks

Each track works in its own worktree (`../booker-wt/w1-<track>`) on branch `w1/<track>`, and merges into `wave-1` by pull request.

### A. Release pipeline — tier M (YAML tickets: S)
**Owns:** `.github/workflows/release.yml` (a new file — `ci.yml` belongs to Wave 0.5, track D touches it for the frontend jobs, and the contracts step added the Linux system-dependency action to it), `app/src-tauri/tauri.conf.json` (bundle and updater sections), release documentation.
Build matrix for macOS (arm64 and x64), Windows x64, Linux x64 (AppImage, deb, rpm) with `tauri-action`. Updater plugin with `createUpdaterArtifacts` (already on) and `latest.json` published as a release asset, plus a beta channel. The minisign keys are already in repository secrets and the public key is already in `tauri.conf.json`. Signing is skipped: there is no Apple Developer ID and no Windows certificate yet, so the workflow must build correctly without them and pick them up when they appear.
**Done when:** a tagged push produces installers for all three platforms and a `latest.json`, and an installed `v0.2.0` updates itself to `v0.2.1`.

### B. Application shell — tier M
**Owns:** `app/src/**` except the preview, `app/src-tauri/src/**`. The contracts step left a session, three commands and a window that lists a book; the rest of the ten commands are unimplemented and named in `commands.rs`.
Tauri 2 + Svelte 5 + TypeScript. Window layout: chapter sidebar, editor pane, preview pane, status bar, problems panel. Open-folder dialog, recent projects, menu including "Check for updates…". IPC commands wired to the core through the generated TypeScript bindings (`cargo test -p booker-core export_bindings`).
**Done when:** the app opens a project folder, lists its chapters, shows its diagnostics, and exports a PDF through the same core the CLI uses.

### C. Preview — tier M
**Owns:** `app/src/lib/preview/**`, the `booker://` protocol handler in `app/src-tauri/src/protocol.rs`, and the watcher in `app/src-tauri/src/watch.rs`.
Page images served over the custom protocol rather than through IPC (base64 through a JSON channel is the slowest thing in the app). Render only visible pages, zoom, scroll, and re-render on change. The revision belongs in the URL so images cache forever and a stale one is never mistaken for current.
The watcher is the minimal one: `notify` with a debouncer, a burst of changes coalesced into one `project-changed` event and one re-render, the page position kept. It must ignore the echo of our own writes, and it must survive thirty files changing at once — an agent rewriting a book while the window is open is the normal case, not the exotic one (`AGENTS.md` §7).
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

1. ~~Wave lead: create `wave-1`, add the Tauri scaffold and the IPC command wiring, freeze it.~~ Done — see "What the contracts step already did" above.
2. **B first, then the rest in parallel.** Track C hangs its preview inside track B's window
   and cannot start before it exists, and the tutorial (D2) describes an application that
   has shipped. So the lead builds the shell, and A, C, D and D2 start once there is a
   window: A and D never touch `app/src/`, and C owns a subtree of it.
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

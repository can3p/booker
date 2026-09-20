# Booker

[![CI](https://github.com/can3p/booker/actions/workflows/ci.yml/badge.svg)](https://github.com/can3p/booker/actions/workflows/ci.yml)

Book authoring for people who want a good-looking book without learning typesetting.

A Booker project is a plain folder: Markdown for the text, a few readable TOML files for how it should look. It lives in git, opens in any editor, and builds into a print-ready PDF. The layout engine is [Typst](https://typst.app), embedded in the application, so a 200-page novel lays out in about half a second.

**Status: early.** The core works end to end on the command line — a folder of Markdown becomes a PDF. Next come the desktop application and the installers, so today Booker is built from source and run as a command. See [`docs/WAVE-LOG.md`](docs/WAVE-LOG.md) for what is finished and [`docs/PLAN.md`](docs/PLAN.md) §8 for where it is going.

## Start here

**[Your first book](docs/tutorials/01-your-first-book.md)** takes you from an empty folder to
a printable PDF of a short book you wrote yourself, in about twenty minutes. It assumes
nothing — it starts at installing Booker — and it is the quickest way to find out what this
is. Everything below is the short version and the reference.

[`docs/tutorials/`](docs/tutorials/index.md) is the full list; between them the tutorials
cover everything Booker can currently do.

## What you need

- **Rust**, stable. Install it with [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  The exact version is pinned by `rust-toolchain.toml`; rustup picks it up on its own. If your shell cannot find `cargo` afterwards, add it:
  ```bash
  . "$HOME/.cargo/env"
  ```

Nothing else. Fonts are bundled, and no LaTeX or Typst installation is needed.

## Run it

```bash
git clone https://github.com/can3p/booker.git
cd booker
cargo build --release
```

The first build takes a few minutes because it compiles the layout engine. After that:

```bash
# Create a book
cargo run --release -p booker-cli -- new ~/books/my-book --title "The Secret Garden of Mia"

# Write in it with any editor — the text is plain Markdown
$EDITOR ~/books/my-book/content/01-the-first-chapter.md

# Build the PDF
cargo run --release -p booker-cli -- build ~/books/my-book
# → Built build/the-secret-garden-of-mia.pdf — 1 pages in 98 ms
```

To type `booker` instead of `cargo run -p booker-cli --`, install it onto your PATH:

```bash
cargo install --path crates/booker-cli
booker new ~/books/my-book
booker build ~/books/my-book
```

### Commands today

| Command | What it does |
|---|---|
| `booker new <path> [--title <title>] [--template novel]` | Create a book project. `novel` is the only template so far. |
| `booker build <path>` | Load the project, report any problems, and write `build/<title>.pdf`. |

`booker build` exits `0` when the book is clean, `1` when the project has errors, and `2` when the command itself could not run.

## What a book looks like on disk

```
my-book/
├── book.toml      # title, author, language, chapter order, page size and margins
├── content/       # the text, one Markdown file per chapter
│   └── 01-the-first-chapter.md
├── assets/images/ # pictures that belong to this book
├── AGENTS.md      # what this folder is, for whoever (or whatever) opens it next
├── .gitignore     # written for you: build output and cache stay out of git
├── .gitattributes
├── build/         # generated PDFs — git-ignored
└── .booker/       # cache — git-ignored, safe to delete at any time
```

Two rules the project format keeps, and will keep:

- **Your text lives only in the Markdown files.** Nothing about layout is copied into a second place, so editing a chapter in VS Code, Obsidian or on GitHub is always safe.
- **Booker never rewrites what you wrote.** Saving preserves comments, key order, and even settings a newer version of Booker wrote that this one does not understand. Opening a project changes nothing on disk.

## Working on Booker itself

```bash
cargo test --workspace                      # the whole suite
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p booker-core export_bindings   # regenerate the TypeScript types
cargo xtask --help                          # development tasks
```

[`CONTRIBUTING.md`](CONTRIBUTING.md) covers the setup, including working on several branches at once with git worktrees. [`AGENTS.md`](AGENTS.md) is the standing instruction for anyone — person or agent — working in this repository: branching, what counts as done, and which document records what.

| Document | What it holds |
|---|---|
| [`docs/PLAN.md`](docs/PLAN.md) | Architecture, and the plan for the work still to come |
| [`docs/WAVE-LOG.md`](docs/WAVE-LOG.md) | What each finished milestone actually shipped |
| [`docs/OPEN-QUESTIONS.md`](docs/OPEN-QUESTIONS.md) | What is undecided, and what we are doing meanwhile |
| [`docs/FINDINGS.md`](docs/FINDINGS.md) | Things learned the hard way, so they are learned once |
| [`docs/tutorials/`](docs/tutorials/index.md) | Tutorials: a real task walked through end to end |
| [`docs/requirements.md`](docs/requirements.md) | What the product must do |

## Licence

MIT — see [`LICENSE`](LICENSE). Booker embeds [Typst](https://github.com/typst/typst) (Apache-2.0) and bundles open fonts; their licences are in `crates/booker-typst/fonts/NOTICE.txt`.

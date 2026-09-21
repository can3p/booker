# Changelog

What changed in each release, for the people who install Booker. The
development view — what each wave shipped, what it deviated from, what it
cost — is [`docs/WAVE-LOG.md`](docs/WAVE-LOG.md).

This file is append-only at the top, and edited on the wave branch at
integration rather than by each track, so that parallel work never conflicts
over it (`AGENTS.md` §3).

## Unreleased

Nothing here has been published: Booker is developed locally for now.

### Added
- **Four templates**: `booker new --template novel | picture-book | poetry | paper`,
  each with sample text that explains itself, and each typeset by its own built-in
  theme — chosen with `theme` in `book.toml`.
- **Books that look like books, with no settings**: a table of contents, chapters
  starting on right-hand pages, page numbers, justified and hyphenated text, curly
  quotes, real dashes, scene breaks. `[toc]` and `[chapter] start` in `book.toml`
  change what you want changed.
- **More Markdown**: pictures with a width, tables, strikethrough, page breaks,
  links between chapters, and `{#name}` to name a heading or a passage.
- **`booker check`** reports every problem without building; `booker where` says
  which page a line is on, and `booker page` which lines a page shows.
- **In the window**: a Markdown editor that styles as you type; click a page to jump
  to its text, and the pages follow the cursor; add, rename, move and remove chapters;
  when a chapter changes on disk while you are typing, both versions are offered.

### Changed
- `booker build` also reports problems found while laying the book out.
- Removing a chapter never deletes its file.


## v0.2.0 — the application (not published: distribution is paused)

The first installable Booker.

### Added
- **A desktop application** for macOS, Windows and Linux. Open a folder that
  contains a book, read its chapters, edit them, watch the pages redraw, see
  what is wrong with the book, and export a PDF.
- **Updates.** The application checks for a newer version and installs it,
  with every download signed. "Check for updates…" is in the menu.
- **The preview follows the folder.** An edit made in another editor — or by
  an agent rewriting thirty chapters at once — is picked up and re-rendered,
  as one reload rather than thirty.
- **Recent books**, so the folders you work on are one click away.

### Known limitations
- The builds are not signed by Apple or Microsoft yet, so the first launch
  shows a warning. Updates are signed and verified regardless.
- The editor is a plain text pane. Live Markdown styling, the chapter tree
  and templates are the next release.
- Beta builds are published as GitHub prereleases and installed by hand;
  there is no channel switch in the application yet.

## v0.1.0 — the core (not released)

`booker new` and `booker build` on the command line, with no installer: the
layout engine, the project format and the diagnostics that the application
is built on. Available by building from source.

# Changelog

What changed in each release, for the people who install Booker. The
development view — what each wave shipped, what it deviated from, what it
cost — is [`docs/WAVE-LOG.md`](docs/WAVE-LOG.md).

This file is append-only at the top, and edited on the wave branch at
integration rather than by each track, so that parallel work never conflicts
over it (`AGENTS.md` §3).

## Unreleased

## v0.2.0 — the application (not yet published)

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

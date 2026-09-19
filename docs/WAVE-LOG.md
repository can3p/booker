# Wave log

What each finished wave actually shipped, written at the end of the wave, newest first. The plan (`docs/PLAN.md` §8) says what is intended; this file says what happened. A wave is not finished until its entry is here.

Format:

```
## Wave N — <name>
- Shipped: YYYY-MM-DD as <tag> (PR #NN)
- Demo: what a person can now do, end to end
- Tracks: which landed, who (model tier) did them
- Deviations from the plan: what changed and why (and the plan was updated in the same change)
- Deferred: what moved to a later wave, with the wave it moved to
- Update check: previous release updated itself to this one — yes/no, on which platforms
- Agent surface (Wave 7 on): MCP conformance green in CI; eval run by hand — pass rate per scenario, and the new scenario this wave added
- Findings recorded: links into docs/FINDINGS.md
```

---

## Wave 0 — The core: engine, format, command line
- Shipped: 2026-09-19 on `main` (no release tag: there is no installer until Wave 1)
- Demo: `booker new my-book` then `booker build my-book` writes a real PDF. A 201-page novel lays out in 520–570 ms cold, 24–29 ms after a one-character edit.
- Tracks: contracts (lead) · C Typst engine · E format and CLI · bridge (lead, at integration). Tracks C and E ran in parallel in separate worktrees, each with its own owned paths.
- Deviations from the plan:
  - **The wave was split.** Tracks A (release pipeline), B (application shell) and F (hygiene) were independent of the core and became Wave 1, so the numbering of every later wave moved up by one. The plan was updated in the same change.
  - The Markdown parser decision was made on measured evidence rather than preference: `pulldown-cmark`, because its spans exclude block markers and are not optional.
  - A minimal document-model-to-Typst bridge was written at integration so the wave could end with a PDF. It is temporary and marked as such; Wave 2 track B replaces it.
- Deferred: `fixtures/novel-200p` as a checked-in fixture (the engine generates its large fixture in code instead); `#:schema` lines in generated project files (the schema itself is Wave 7).
- Update check: not applicable — no installers in this wave.
- Agent surface: not applicable — Wave 7.
- Findings recorded: 13 entries in `docs/FINDINGS.md`, including the incremental-compile numbers, Typst's global file-id cache, `toml_edit` discarding spans when a document becomes mutable, and ts-rs ignoring `#[serde(flatten)]`.

### What was learned about working in parallel
Two tracks, roughly 2,500 lines, integration in about ten minutes. Three conflicts, all anticipated by the brief: `docs/FINDINGS.md` (both appended — keep both sides), `Cargo.lock` (regenerate, never merge), and `Cargo.toml` — which **merged cleanly and then failed to parse**, because both tracks appended `tempfile = "3"`. Append-only prevents textual conflicts, not semantic ones; Wave 1 adds a `cargo metadata` check to CI for exactly this.

The frozen-contract rule held: two agents reported nine friction points between them and neither edited `booker-core`. Three of those were fixed by the wave lead at integration; the rest are open questions.

Writing the bridge at integration was what proved the two tracks fit together — and it immediately found a content-loss bug (tight list items came back with no text at all) that neither track's own tests could have caught.

# Open questions

Everything not yet decided. Agents append here rather than guessing silently; the owner answers. When a question is answered, move the answer into `docs/PLAN.md` (§12 Decisions) and mark the entry **Answered** with the date, keeping it here for one wave so the history is visible.

Format:

```
### Q-NN — short question
- Raised: YYYY-MM-DD, wave N / track X
- Needed by: (what it blocks, or "before first release")
- Options: ...
- Default we are proceeding with: ...
- Status: Open | Answered YYYY-MM-DD — <answer>
```

---

### Q-01 — Final application name and bundle identifier
- Raised: 2026-09-19, planning
- Needed by: **before the first public release**. After that, changing the identifier stops updates reaching everyone who installed it.
- Options: keep `com.github.can3p.booker`; or a domain-based identifier if a domain is acquired; the product name may change independently of the identifier.
- Default we are proceeding with: `com.github.can3p.booker`, product name "Booker".
- Status: Open

### Q-02 — Which fonts to bundle
- Raised: 2026-09-19, planning
- Needed by: Wave 1 (starter templates pick their fonts)
- Options: a small set covering a serif for fiction, a text face for non-fiction, a friendly face for kids books, a sans for UI and captions, and a decorative face for initials. Licences must permit redistribution and embedding (OFL and similar).
- Default we are proceeding with: Literata, EB Garamond, Source Serif 4, Inter, Atkinson Hyperlegible, Andika (kids), Cinzel Decorative (initials) — reviewed for licence in Wave 1.
- Status: Open

### Q-03 — Do print-on-demand targets need CMYK, ICC or PDF/X?
- Raised: 2026-09-19, planning
- Needed by: Wave 9 (colour management), but affects which presets we advertise earlier
- Context: Typst has CMYK colours but no ICC profiles, output intent or PDF/X. Amazon KDP, IngramSpark and Lulu accept RGB PDFs today; traditional print shops often do not.
- Default we are proceeding with: RGB output, colour values that carry their colour space from Wave 2 so conversion can be added as a post-processing step later.
- Status: Open

### Q-04 — Spell-checking
- Raised: 2026-09-19, planning
- Needed by: Wave 10
- Options: rely on the system webview's spell-checker (free, but behaves differently on each platform, and Linux support varies), or bundle a dictionary-based checker in the core (consistent, more work, larger download).
- Default we are proceeding with: system spell-checker, revisited if Linux behaviour is poor.
- Status: Open

### Q-05 — Where do shared templates and themes come from?
- Raised: 2026-09-19, planning
- Needed by: Wave 10
- Options: install from any git URL only; or a curated list in a repository we control; or a gallery site.
- Default we are proceeding with: install from a git URL, plus the bundled templates.
- Status: Open

### Q-06 — How far should the agent surface be allowed to write?
- Raised: 2026-09-19, planning
- Needed by: Wave 6
- Context: agents already edit Markdown and TOML well with their own tools. What they lack is truth about the rendered result. A large write API would mostly be a worse way to edit a text file, but "fix it for me" is a natural request.
- Options: read-mostly (`fmt` and `fix --safe` only); or a full write API (set a style, move a frame, apply a layout); or read-mostly by default with writes behind an explicit flag.
- Refined 2026-09-19: the question largely dissolves if agents can *discover* the vocabulary instead of guessing at it (`docs/PLAN.md` §11.4: generated schemas, a capability manifest, `explain`, recipes, and errors that suggest the right key). Then the useful write tools are only the ones needing the layout engine in the loop — applying a diagnostic's patch, and fit-and-iterate adjustments such as "shrink until the text fits".
- Default we are proceeding with: read-mostly plus engine-in-the-loop writes; everything expressible as a text edit stays a text edit.
- Status: Open

### Q-07 — Should a running app expose the agent tools too?
- Raised: 2026-09-19, planning
- Needed by: after Wave 6
- Context: `booker mcp` works headless on a folder, which covers most cases. Letting an agent talk to the *running* app would mean it can act on what the user is currently looking at, and the user sees changes land live — at the cost of a local socket, a token, and a permissions story.
- Default we are proceeding with: ship the headless CLI/MCP path first, decide on the app-attached variant once people have used it.
- Status: Open

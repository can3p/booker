# Open questions

Everything not yet decided. Agents append here rather than guessing silently; the owner answers. When a question is answered, move the answer into `docs/PLAN.md` (§13 Decisions) and mark the entry **Answered** with the date, keeping it here for one wave so the history is visible.

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
- Needed by: Wave 2 (starter templates pick their fonts)
- Options: a small set covering a serif for fiction, a text face for non-fiction, a friendly face for kids books, a sans for UI and captions, and a decorative face for initials. Licences must permit redistribution and embedding (OFL and similar).
- Default we are proceeding with: Literata, EB Garamond, Source Serif 4, Inter, Atkinson Hyperlegible, Andika (kids), Cinzel Decorative (initials). The licence review was planned for Wave 1 and did not happen; today Booker bundles only Typst's own defaults, Libertinus Serif and DejaVu Sans Mono (`crates/booker-typst/fonts/NOTICE.txt`). It moves to Wave 2 track F, which is where the fonts are first needed.
- Status: Open

### Q-03 — Do print-on-demand targets need CMYK, ICC or PDF/X?
- Raised: 2026-09-19, planning
- Needed by: Wave 10 (colour management), but affects which presets we advertise earlier
- Context: Typst has CMYK colours but no ICC profiles, output intent or PDF/X. Amazon KDP, IngramSpark and Lulu accept RGB PDFs today; traditional print shops often do not.
- Default we are proceeding with: RGB output, colour values that carry their colour space from Wave 2 so conversion can be added as a post-processing step later.
- Status: Open

### Q-04 — Spell-checking
- Raised: 2026-09-19, planning
- Needed by: Wave 11
- Options: rely on the system webview's spell-checker (free, but behaves differently on each platform, and Linux support varies), or bundle a dictionary-based checker in the core (consistent, more work, larger download).
- Default we are proceeding with: system spell-checker, revisited if Linux behaviour is poor.
- Status: Open

### Q-05 — Where do shared templates and themes come from?
- Raised: 2026-09-19, planning
- Needed by: Wave 11
- Options: install from any git URL only; or a curated list in a repository we control; or a gallery site.
- Default we are proceeding with: install from a git URL, plus the bundled templates.
- Status: Open

### Q-06 — How far should the agent surface be allowed to write?
- Raised: 2026-09-19, planning
- Needed by: Wave 7
- Context: agents already edit Markdown and TOML well with their own tools. What they lack is truth about the rendered result. A large write API would mostly be a worse way to edit a text file, but "fix it for me" is a natural request.
- Options: read-mostly (`fmt` and `fix --safe` only); or a full write API (set a style, move a frame, apply a layout); or read-mostly by default with writes behind an explicit flag.
- Refined 2026-09-19: the question largely dissolves if agents can *discover* the vocabulary instead of guessing at it (`docs/PLAN.md` §11.4: generated schemas, a capability manifest, `explain`, recipes, and errors that suggest the right key). Then the useful write tools are only the ones needing the layout engine in the loop — applying a diagnostic's patch, and fit-and-iterate adjustments such as "shrink until the text fits".
- Default we are proceeding with: read-mostly plus engine-in-the-loop writes; everything expressible as a text edit stays a text edit.
- Status: Open

### Q-07 — Should a running app expose the agent tools too?
- Raised: 2026-09-19, planning
- Needed by: after Wave 7
- Context: `booker mcp` works headless on a folder, which covers most cases. Letting an agent talk to the *running* app would mean it can act on what the user is currently looking at, and the user sees changes land live — at the cost of a local socket, a token, and a permissions story.
- Default we are proceeding with: ship the headless CLI/MCP path first, decide on the app-attached variant once people have used it.
- Status: Open

### Q-08 — How does an author write a page size?
- Raised: 2026-09-19, Wave 0 track E
- Needed by: Wave 2 (templates write `book.toml`)
- Context: `PLAN.md` §5.2 shows `size = "8.5x8.5in"`, but the `PageSize` type is either a named preset or `{ width, height }`. The loader currently accepts all three, which means three ways to write one thing.
- Options: make the string form canonical and the table form the escape hatch; or the reverse; or keep all three and say so in the schema.
- Default we are proceeding with: all three accepted, presets preferred in anything Booker generates.
- Since 2026-09-20 (Wave 0.6) `docs/tutorials/01-your-first-book.md` teaches the preset first and then the string form, `size = "5.5x8.5in"`, so whichever way this is answered, answering it now means editing a tutorial as well as the loader — and the tutorial test will say so.
- Status: Open

### Q-09 — Must a book have a title?
- Raised: 2026-09-19, Wave 0 track E
- Needed by: Wave 2
- Context: `BookConfig::title` is not optional, so a `book.toml` without one loads as an empty string and the CLI prints "(untitled)". Nothing treats that as an error.
- Options: make it `Option<String>`; or keep it required and add a `format.*` diagnostic saying a book needs a title; or leave it.
- Default we are proceeding with: leave it, and revisit when the diagnostics engine exists (Wave 7).
- Status: Open

### Q-10 — How much CI do we buy, and does `main` get branch protection?
- Raised: 2026-09-19, wave 0.5
- Needed by: Wave 0.5, and again at Wave 1 when the release matrix starts building installers
- Context: the Typst dependency tree is expensive to compile, and only the owner can enable branch protection on GitHub — an agent can add a pre-push hook, which is advice, not a gate.
- Options: (a) Linux-only checks on every push, three platforms only on pull requests into `main`; (b) three platforms on every pull request; (c) three platforms on everything.
- Default we are proceeding with: (a), with no scheduled jobs of any kind, and a request to the owner to turn on branch protection for `main` with `fmt`, `clippy` and `test` as required checks once they are green.
- Status: **Answered 2026-09-20.** Both halves are closed.

  The spend question was settled by measurement and is now what CI does: a pull request into a wave branch costs about a minute of Linux time, and a pull request into `main` about six minutes of wall clock and fifteen of runner time, because Windows is five of those (see the wave log). No job runs on a schedule.

  Branch protection is on, as a repository **ruleset** named "protect main" rather than the older branch-protection settings — which is why `gh api repos/can3p/booker/branches/main/protection` answers "Branch not protected" and `gh api repos/can3p/booker/rules/branches/main` is the query that shows it. It forbids deletion and force-pushes and requires a pull request (zero approvals), so `AGENTS.md` §3 is now enforced rather than merely asked for, and `.githooks/pre-push` is the early warning rather than the only one.

  The one gap left when this was answered — the ruleset listed no required status checks, so a red pull request could still be merged — was tracked separately as Q-11, and closed on 2026-09-21.

### Q-11 — Should the "protect main" ruleset require the CI checks to pass?
- Raised: 2026-09-20, wave 1 ramp-up
- Needed by: before the Wave 1 release, which is the first merge into `main` that produces something people install
- Context: the ruleset that closed Q-10 requires a pull request but lists no required status checks, so nothing stops a red one from being merged. The six job names are stable and have been green on every pull request since Wave 0.5.
- Options: add all six as required checks; or require only `fmt`, `clippy` and `test (ubuntu-latest)` and let the cheaper jobs advise; or leave CI advisory.
- Default we are proceeding with: ask before the Wave 1 merge, and treat a red check as blocking by convention until then.
- Status: **Answered 2026-09-21.** The six checks the question named are required: `fmt`, `clippy`, `test (ubuntu-latest)`, `docs`, `bindings` and `deps`. CI is a gate rather than advice, and a red pull request into `main` cannot be merged.

  Two things were settled alongside it. The ruleset also requires a **linear history**, and `rebase` is now the only merge method it allows — so a wave arrives on `main` as its own commits, and `AGENTS.md` §3 says so. Both are in `docs/PLAN.md` §13.

  What the gate deliberately does not cover: `test (macos-latest)`, `test (windows-latest)` and `app` run on every pull request and are read, but are not required, so a platform-specific failure does not block a merge on its own. This wave had one — a race in the watcher that only a loaded macOS runner reproduced (`docs/FINDINGS.md`) — so it is worth knowing that the gate would not have caught it. Nor is "branches must be up to date before merging" switched on, so a green run against a base that has since moved still counts.

### Q-12 — How does somebody choose the beta channel?
- Raised: 2026-09-20, wave 1 / track A
- Needed by: the first beta anyone is meant to install — not before
- Context: a beta tag builds the same installers and marks its GitHub release as a prerelease, so it is not what `releases/latest` points at and no installed copy is ever offered it. That much works today, and a beta can be installed by hand from its release page. What does not exist is a way to *stay* on the beta channel: that needs a second manifest at a fixed URL (`beta.json`) and a setting in the application that points the updater at it.
- Options: build it now; build it when somebody asks for a beta; or decide that betas are always installed by hand and drop the channel idea.
- Default we are proceeding with: wait. The machinery is a day's work whenever it is wanted, and building a channel nobody has used yet means guessing at how people will want to move between them — including the awkward part, which is going back to stable from a beta whose version number is higher.
- Status: Open

### Q-13 — When does Booker start being distributed?
- Raised: 2026-09-21, Wave 1 / release
- Needed by: the first person who is meant to install Booker rather than build it
- Context: Wave 1 built the release pipeline and the signed updater. Its first tagged build failed on macOS and Windows; once fixed, a dry run built all four targets. Apple and Windows signing secrets do not exist.
- Decision so far: **2026-09-21, the owner: keep the updater, but focus on local development, not distribution, for the time being.** No release is made, `AGENTS.md` §4 criteria 5 and 6 are not checked, and waves are not tagged.
- Open: when distribution resumes, and whether signing certificates come first. The first release after the pause also carries the update check Wave 1 never ran (`CONTRIBUTING.md`, "Making a release").
- Status: Open

# Findings

Things learned while building Booker that a future session would otherwise have to rediscover: how a dependency really behaves, a trap in the toolchain, a measurement, an approach that was tried and abandoned and why.

The test for including something: **would a session next month waste an hour without it?** Ordinary code context does not belong here — the code says that itself.

Format:

```
### <short title>
- Learned: YYYY-MM-DD, wave N / track X
- What: the fact, stated plainly
- Why it matters: what it changes about how we build
- Where: file paths, issue links, versions it applies to
```

---

### Typst has CMYK colours but no colour management
- Learned: 2026-09-19, planning research
- What: Typst can express CMYK colours, but has no ICC profile embedding, no PDF output intent and no PDF/X export. These are open upstream issues (typst/typst #3143, #3002).
- Why it matters: colour management has to be a post-processing step after Typst produces the PDF, behind a single export hook. Colour values must carry their colour space in our own model from the start, or retrofitting is painful.
- Where: `docs/PLAN.md` §10, Wave 9 track C. Applies to Typst 0.15.x.

### `meander` is MIT-licensed
- Learned: 2026-09-19, planning research
- What: the Typst package that wraps text around images and threads text between containers (`github.com/Vanille-N/meander.typ`, 0.4.4) is MIT.
- Why it matters: frame chains are core to Booker, so we must be able to vendor and, if needed, fork this package. MIT makes that safe, and our own licence is MIT too. Prefer contributing fixes upstream before forking.
- Where: `docs/PLAN.md` §6, Wave 0 spike D2.

### Typst 0.15 changed how blocks behave in HTML export
- Learned: 2026-09-19, planning research
- What: 0.15.0 aligned `box` and `block` between HTML and paged export — a breaking change — and added bundle export (one project, several output files) plus MathML for equations.
- Why it matters: when we pin or upgrade the Typst version, HTML-adjacent output can shift even if the PDF does not. The golden suite must cover both.
- Where: Typst 0.15 release notes; `docs/PLAN.md` §3.

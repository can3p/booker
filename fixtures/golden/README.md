# Golden books

One small book per built-in theme, rendered to PNG and compared with the
snapshots beside it by `crates/booker-cli/tests/golden.rs`. Between them they
use everything Wave 2's codegen lays out: chapters and the table of contents,
chapters opening on the right, scene breaks, lists, quotations, tables,
pictures (and a missing one, drawn as a placeholder), page breaks, kept line
breaks, typographic quotes and dashes.

A change to how books look fails the suite with the page that moved. If the
change was meant, regenerate and **look at every image before committing**
(`AGENTS.md` §6):

```bash
cargo xtask golden
```

The snapshots are `<book>/snapshots/page-NN.png`, at half a CSS pixel per
point-ish — small enough that the repository does not grow by megabytes a
wave, large enough that a margin moving by a millimetre shows.

# Tutorials

A tutorial is one real task, walked through from start to finish, for someone making
their first book rather than someone who already knows typesetting. It says every
command to type and shows what the screen actually says back.

Read them in order the first time; after that, go to the one that covers what you want
to do.

| # | Tutorial | What you will have made | What it assumes |
|---|---|---|---|
| 01 | [Your first book](01-your-first-book.md) | A two-chapter book, written by you, built into a PDF | Nothing. It starts at installing `booker`. |

That is the whole list today, and it covers everything Booker can currently do. Booker
is a command-line tool at this point in its life: there is no application window, no
preview, and no styling beyond the page size and margins. Each of those arrives with a
tutorial of its own — see [`../PLAN.md`](../PLAN.md) §8 for the order, and
[`../WAVE-LOG.md`](../WAVE-LOG.md) for what has actually shipped.

---

## For anyone writing or changing a tutorial

**A tutorial that is wrong is worse than no tutorial at all**, and a tutorial goes wrong
quietly: nobody notices when a printed line changes, except the next reader. So the
commands in these files are not decoration — they are executed. `cargo test -p
booker-cli --test tutorials` replays every tutorial in a temporary folder and compares
what Booker actually printed with what the tutorial claims it prints. It runs as part
of `cargo test`, on every pull request, on all three platforms.

That only works if the blocks follow a convention. There are three kinds:

### ` ```console ` — run this, and check the output

```` markdown
```console
$ booker build .
Problems: none
Built build/the-moon-in-a-jar.pdf — 2 pages in … ms
```
````

Every line starting with `$ ` is a command. The lines after it, up to the next `$ ` or
the end of the block, are what that command must print. Standard error counts as
output too, and comes after standard output.

The test understands four commands, which is deliberately not a shell:

| Command | What the test does |
|---|---|
| `booker …` | Runs the real `booker` binary in the temporary book folder |
| `cd <path>` | Moves the folder the following commands run in |
| `rm <path>` | Deletes a file, as the reader would |
| `echo $?` | Prints the exit code of the previous command — write it whenever the exit code is part of the lesson |

Anything else fails the test and tells you to mark the block `ignore`. Silently
skipping a line is how a tutorial rots.

A command is expected to succeed. If it is supposed to fail, follow it with `echo $?`
and the code you expect; the test then requires exactly that code.

**`…` (a single U+2026 ellipsis) means "anything here"** and is how a duration, a version
or a machine-specific path stays out of the comparison. `2 pages in … ms` matches
whatever the clock said. Use it only for things that genuinely vary — if you find
yourself wildcarding a word count, the tutorial and the software disagree and one of
them needs fixing.

### ` ```console ignore ` — show this, do not run it

For the steps the test cannot or should not perform: cloning the repository, `cargo
install`, `git commit`, opening a PDF. The block is still shown to the reader exactly
as written, so it must still be true — it is just not checked.

### ` ```<language> file=<path> ` — this is a file, write it

```` markdown
```toml file=book.toml
title = "The Moon in a Jar"
```
````

The block is written to that path, relative to the current folder, before the next
command runs. This is what keeps the reader's `book.toml` and the one the test builds
identical — and it means the reader can copy the block wholesale and be certain of what
they get. Show the **whole file**, not a fragment.

A fenced block with no `file=` and no `console` is shown and nothing more, which is
right for illustrating something the reader does not need to type.

### The rules that are not mechanical

- **Output is copied from a real run.** Never typed from memory, never tidied up. If
  the alignment looks odd, the alignment is what Booker prints.
- **A real, specific thing to make**, with a name. Not "your project".
- **No typesetting vocabulary without a sentence explaining it**, and no reference to
  the plan, a wave, a crate or a rule's internals. The reader is making a kids book or
  a novel and does not care how Booker is built.
- **Say what to do when it goes wrong**, at each step where it plausibly does.
- **End by saying what the software cannot do yet**, so nobody hunts for a feature that
  has not been built.

### When you change something a reader can see

Update the tutorial in the same change (`AGENTS.md` §2). If the change is a new
capability, either extend the tutorial that covers the surrounding task or add
`NN-<name>.md` and a row in the table above. The test will tell you if you forgot a
printed line; it cannot tell you if you forgot a paragraph.

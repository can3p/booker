# Your first book

By the end of this you will have written a short book — two chapters, a title page
size of your choosing — and turned it into a PDF you can print or send to someone.
It takes about twenty minutes, and you do not need to know anything about
typesetting.

The book we are going to make is called *The Moon in a Jar*. You will type its two
chapters yourself, so that the words on your screen are your own and not a sample
file. If you would rather write something else, do — just keep the file names, and
the numbers Booker prints back will be different from the ones shown here.

**What you need:** Booker, and a text editor. Any editor: VS Code, Obsidian, TextEdit,
Notepad, `vim`. A Booker book is plain text in a plain folder, so nothing is locked to
one program.

---

## 1. Get the `booker` command

Booker is not yet something you download and install — that arrives later — so for now
you build it from source. You need [Rust](https://rustup.rs) once; after that this is
two commands.

```console ignore
$ git clone https://github.com/can3p/booker.git
$ cd booker
$ cargo install --path crates/booker-cli
```

The first build takes a few minutes, because it compiles the layout engine that turns
your words into pages. When it finishes, check that your shell can find the command:

```console
$ booker --version
booker 0.2.0
```

**If your shell says `command not found: booker`**, Rust put it somewhere your shell is
not looking. Run `. "$HOME/.cargo/env"` and try again; to make that permanent, add the
same line to your `~/.zshrc` or `~/.bashrc`.

**If you would rather not install it onto your system**, you can run it out of the
Booker folder instead, writing `cargo run --release -p booker-cli --` everywhere this
tutorial writes `booker`. It is longer to type and does exactly the same thing.

Now move somewhere you keep your own work — your home folder, `~/Documents`, wherever
you like. The next command creates a new folder there.

## 2. Make the book

```console
$ booker new the-moon-jar --title "The Moon in a Jar"
Created a `novel` book in the-moon-jar
  book.toml
  content/01-the-first-chapter.md
  assets/images/.gitkeep
  .gitignore
  .gitattributes
  AGENTS.md

  book.toml — the book's settings
  content/ — the text, one file per chapter
  AGENTS.md — what this folder is, for whoever opens it next

Next:
  booker build the-moon-jar
```

`the-moon-jar` is the folder name; `--title` is what will be printed on the book. If you
leave `--title` out, Booker makes a title out of the folder name — `the-moon-jar` would
become *The Moon Jar* — which is usually close enough to start with.

The book starts from a *template*. `novel` is the one you get without asking, and the
right one for this book: chapters, flowing text, a table of contents. There are three
more — `picture-book` for a square book with a picture and a few big lines on each
page, `poetry` for poems that keep their line breaks, and `paper` for an essay or a
report — and you choose one with `--template`. Get the name slightly wrong and Booker
says what it has rather than guessing:

```console
$ booker new my-poems --template poem
booker: there is no template called `poem`; the templates are: novel, picture-book, poetry, paper — did you mean `poetry`?
$ echo $?
2
```

Nothing was created: the exit code `2` means the command could not run at all.

That folder is now your book, and it is the whole book. There is no database, no
hidden file somewhere else, nothing you can lose by moving it. Step inside it:

```console
$ cd the-moon-jar
```

Here is what is in it:

| File | What it is |
|---|---|
| `book.toml` | The settings: title, author, language, which chapters and in what order, how big the page is |
| `content/` | Your text, one file per chapter, written in Markdown |
| `assets/images/` | Pictures that belong to this book |
| `AGENTS.md` | A note explaining the folder, for whoever opens it next — a collaborator, or an AI assistant you point at it |
| `.gitignore`, `.gitattributes` | So the folder is ready to put in git, if you use git |

Two more folders appear once you build: `build/`, which holds the PDF, and `.booker/`,
a cache. Both are generated, both are safe to delete at any moment, and both are
already excluded from git for you.

## 3. Build it before you change anything

It is worth seeing the whole thing work once before you touch it. Booker put a sample
chapter in `content/`, so the book already has words in it.

```console
$ booker build .
The Moon in a Jar — format 1, language en
Page 148mm × 210mm, facing, margins 18mm/20mm/20mm/15mm
Chapters: 1
  content/01-the-first-chapter.md  74 words, 1 heading, 0 images   “The First Chapter”
  74 words and 0 images in all
Problems: none
Built build/the-moon-in-a-jar.pdf — 1 pages in … ms
```

The `.` means "the book in this folder". You can also stand outside the folder and
write `booker build the-moon-jar`.

Read that summary top to bottom, because you will see it every time:

- **The first line** is your book: title, the version of the project format, and the
  language the text is in. The language is not decoration — it tells the layout engine
  how to break words at the end of a line.
- **`Page 148mm × 210mm, facing`** is the paper. 148 × 210 mm is A5, a common size for
  a novel. *Facing* means Booker treats the book as left-hand and right-hand pages
  like a real printed book, so the inner margin — the one that disappears into the
  spine — is on the right of a left-hand page and on the left of a right-hand page.
- **The chapter lines** are a count of what is actually in each file. If a chapter says
  `0 words`, you are looking at an empty file, and that is usually the answer to "why
  is my book short".
- **`Problems: none`** means Booker found nothing wrong. When it does find something,
  it lists it here — we will make that happen on purpose in a minute.
- **The last line** is the PDF. Open it now: `open build/the-moon-in-a-jar.pdf` on a
  Mac, `xdg-open` on Linux, `start` on Windows, or just double-click it.

That is a real, finished, printable PDF, and you have not configured anything.

**If nothing at all happened and you got `booker: .: there is no folder here`**, you are
not inside the book folder. `cd the-moon-jar` first.

## 4. Write your chapters

Delete the sample chapter and write two of your own:

```console
$ rm content/01-the-first-chapter.md
```

Create `content/01-the-jar.md` in your editor and type this:

```markdown file=content/01-the-jar.md
# The Jar

Mia found the jar on the back step, where the milk used to be left, and the
moon was already inside it.

It was not a small moon. It filled the glass the way milk fills a glass, and
it hummed — very quietly, the way the fridge hums at night when you are the
only one awake.

She sat down on the step beside it. "You are supposed to be up there," she
said.

The moon did not answer. Moons rarely do.
```

Then `content/02-the-walk-home.md`:

```markdown file=content/02-the-walk-home.md
# The Walk Home

She carried the jar home with both hands, the way you carry a full cup.

Three things happened on the way:

- a cat stopped to look, and then pretended it had not
- every puddle turned silver as she passed
- the street lamps switched themselves off, one by one, embarrassed

> "Keep the lid loose," her grandmother had told her once, about something
> else entirely. "Anything worth keeping needs to breathe."

By the time Mia reached her gate, the jar was warm.
```

That is Markdown, and you have now used most of it. A line starting with `#` is a
heading — the chapter title. A blank line starts a new paragraph. Lines starting with
`-` are a list, a line starting with `>` is a quotation, `*word*` is italic and
`**word**` is bold. That is genuinely the lot; you do not need to learn anything else
to write a novel.

The file names matter in one small way: `01-` and `02-` keep them in order in your
editor's file list, which is a kindness to yourself later. Booker takes the order from
`book.toml`, which we are about to write.

## 5. Make `book.toml` yours

Open `book.toml`. Booker wrote it with explanatory comments in it; read them once, then
replace the whole file with this:

```toml file=book.toml
format = 1
title = "The Moon in a Jar"
author = "Mia Ferris"
language = "en"

chapters = [
    "content/01-the-jar.md",
    "content/02-the-walk-home.md",
]

[page]
size = "a5"
facing = true

[page.margins]
top = "18mm"
bottom = "20mm"
inside = "20mm"
outside = "15mm"
```

Put your own name in `author` if you like — the rest of this tutorial shows Mia Ferris,
so if you change it, that one line of the output will differ from what is printed here.

Line by line:

- **`format = 1`** says which version of the project format this book uses. Leave it
  alone; Booker updates it for you when it needs to.
- **`title`** and **`author`** are the book's. The title is also where the PDF's file
  name comes from — *The Moon in a Jar* became `the-moon-in-a-jar.pdf`.
- **`language = "en"`** is the language of the text, and controls hyphenation.
- **`chapters`** is the order the book is read in. Every entry is a path from the book
  folder. If you delete the whole list, Booker reads `content/*.md` in file-name order
  instead, which is why naming files `01-`, `02-` is a good habit.
- **`[page]`** is the paper. `size` takes a preset — `a4`, `a5`, `letter`, `trade`,
  `digest`, `square` — or a measurement of your own, which we will use in a moment.
- **`[page.margins]`** is the white space around the text, in millimetres or inches.
  `inside` is the margin next to the spine and `outside` is the one at the open edge of
  the book; they swap sides on every page because `facing` is `true`. If you set
  `facing = false`, every page is laid out the same way, which is what you want for
  something that will be read on a screen rather than bound.

Now build it:

```console
$ booker build .
The Moon in a Jar — format 1, language en
by Mia Ferris
Page 148mm × 210mm, facing, margins 18mm/20mm/20mm/15mm
Chapters: 2
  content/01-the-jar.md            85 words, 1 heading, 0 images   “The Jar”
  content/02-the-walk-home.md      83 words, 1 heading, 0 images   “The Walk Home”
  168 words and 0 images in all
Problems: none
Built build/the-moon-in-a-jar.pdf — 5 pages in … ms
```

Two chapters, five pages. Open the PDF again and look at it, because it now looks like
a book rather than a printout:

- **Page 1 is a table of contents**, listing both chapters with the page each starts on.
  A book with more than one chapter gets one without being asked.
- **Each chapter starts on a right-hand page**, the way printed novels do. When the page
  before a chapter would otherwise be a right-hand one, Booker leaves a blank left-hand
  page in between — that is what pages 2 and 4 are. A blank page carries no page number.
- **Chapter titles are dropped** a little way down the page, centred and larger than
  the text.
- **The text is set as prose.** The first paragraph of a chapter starts flush; every
  paragraph after it has its first line indented, with no gap between paragraphs. The
  lines are justified — straight down both edges — and long words are hyphenated to make
  that work, which is why `language` matters.
- **Your punctuation is typeset.** The straight `"` you typed became curly “ and ”, an
  apostrophe becomes ’, and if you type two hyphens `--` you get an en dash (–) and three
  `---` an em dash (—).
- The list and the quotation you wrote came out as a list and a quotation, and the page
  numbers sit on the outer corner of each page.

Now look at pages 3 and 5, the chapter openings. Both are right-hand pages, and the white
space is wider on their *left*, against the spine: that is `facing = true` doing its job,
putting the generous `inside` margin where the book is bound. Nobody set any of that up.
It all comes from the book's *theme* — `novel`, unless you choose another — and the
defaults are meant to be good enough that you never think about them.

## 6. Change the shape of the page

A5 is a fine size, but say you want something closer to a paperback you would buy. Put
your own measurement in `size`, and switch the margins to inches to match:

```toml file=book.toml
format = 1
title = "The Moon in a Jar"
author = "Mia Ferris"
language = "en"

chapters = [
    "content/01-the-jar.md",
    "content/02-the-walk-home.md",
]

[page]
size = "5.5x8.5in"
facing = true

[page.margins]
top = "0.7in"
bottom = "0.8in"
inside = "0.8in"
outside = "0.6in"
```

```console
$ booker build .
The Moon in a Jar — format 1, language en
by Mia Ferris
Page 5.5in × 8.5in, facing, margins 0.7in/0.8in/0.8in/0.6in
Chapters: 2
  content/01-the-jar.md            85 words, 1 heading, 0 images   “The Jar”
  content/02-the-walk-home.md      83 words, 1 heading, 0 images   “The Walk Home”
  168 words and 0 images in all
Problems: none
Built build/the-moon-in-a-jar.pdf — 5 pages in … ms
```

The page line changed and the book re-flowed to fit. You can mix units freely — `mm`
here and `in` there — and you can write the size either way round: `"5.5x8.5in"` is the
same as `"140x216mm"` to within a rounding error.

This is the loop you will spend most of your time in: change a line, build, look. It
takes milliseconds, so change things freely.

## 7. Two things that go wrong, on purpose

Sooner or later you will mistype something, and it is much nicer to find out now what
that looks like. Booker has one rule about broken books: **it always builds what it
can, and tells you the rest.** It will not refuse to open your book because there is a
mistake in it — that is exactly the moment you need to see your pages.

### A key it does not recognise

Add a line with a typo in it — `auther` instead of `author`:

```toml file=book.toml
format = 1
title = "The Moon in a Jar"
author = "Mia Ferris"
language = "en"
auther = "Mia Ferris"

chapters = [
    "content/01-the-jar.md",
    "content/02-the-walk-home.md",
]

[page]
size = "5.5x8.5in"
facing = true

[page.margins]
top = "0.7in"
bottom = "0.8in"
inside = "0.8in"
outside = "0.6in"
```

```console
$ booker build .
The Moon in a Jar — format 1, language en
by Mia Ferris
Page 5.5in × 8.5in, facing, margins 0.7in/0.8in/0.8in/0.6in
Chapters: 2
  content/01-the-jar.md            85 words, 1 heading, 0 images   “The Jar”
  content/02-the-walk-home.md      83 words, 1 heading, 0 images   “The Walk Home”
  168 words and 0 images in all
Problems: 0 errors, 1 warning
  book.toml:5:1: warning[BK-FORMAT-005] unknown key `auther`; it is being kept unchanged — did you mean `author`?
Built build/the-moon-in-a-jar.pdf — 5 pages in … ms
```

Everything worth noticing is in that one line:

- **`book.toml:5:1`** — the file, the line and the column. Most editors will jump
  straight there if you paste that in.
- **`warning`** — a warning, not an error. The book still built.
- **`[BK-FORMAT-005]`** — a name for this kind of problem that never changes, so you can
  search for it, and so a script or an assistant can recognise it.
- **`it is being kept unchanged`** — Booker did not delete your line. It keeps settings
  it does not understand, in case a newer version of Booker wrote them, or you meant
  something by them.
- **`did you mean `author`?`** — which is the actual answer.

### A chapter that is not there

Now a real error. Add a third chapter to the list — but do not write the file:

```toml file=book.toml
format = 1
title = "The Moon in a Jar"
author = "Mia Ferris"
language = "en"
auther = "Mia Ferris"

chapters = [
    "content/01-the-jar.md",
    "content/02-the-walk-home.md",
    "content/03-the-lid.md",
]

[page]
size = "5.5x8.5in"
facing = true

[page.margins]
top = "0.7in"
bottom = "0.8in"
inside = "0.8in"
outside = "0.6in"
```

```console
$ booker build .
The Moon in a Jar — format 1, language en
by Mia Ferris
Page 5.5in × 8.5in, facing, margins 0.7in/0.8in/0.8in/0.6in
Chapters: 2
  content/01-the-jar.md            85 words, 1 heading, 0 images   “The Jar”
  content/02-the-walk-home.md      83 words, 1 heading, 0 images   “The Walk Home”
  168 words and 0 images in all
Problems: 1 error, 1 warning
  book.toml:5:1: warning[BK-FORMAT-005] unknown key `auther`; it is being kept unchanged — did you mean `author`?
  book.toml:10:5: error[BK-REF-002] chapter `content/03-the-lid.md` is listed but the file is not there
Built build/the-moon-in-a-jar.pdf — 5 pages in … ms
$ echo $?
1
```

The PDF was still written, with the two chapters that exist. But the last line is new:
`echo $?` asks the shell what the previous command reported, and this time it is `1`
rather than `0`. That is how Booker says "there is an error in this book" to anything
that is not a person reading the screen — a script, a build server, an assistant. A
clean book gives you `0`, a book with errors gives you `1`.

Worth keeping straight: a **warning** is something Booker thinks you should look at, and
an **error** is something that will make the book come out wrong. Both are listed, both
name a file and a line, and neither stops you from getting a PDF.

### Put it back

Delete both mistakes — the `auther` line and the `content/03-the-lid.md` line:

```toml file=book.toml
format = 1
title = "The Moon in a Jar"
author = "Mia Ferris"
language = "en"

chapters = [
    "content/01-the-jar.md",
    "content/02-the-walk-home.md",
]

[page]
size = "5.5x8.5in"
facing = true

[page.margins]
top = "0.7in"
bottom = "0.8in"
inside = "0.8in"
outside = "0.6in"
```

```console
$ booker build .
The Moon in a Jar — format 1, language en
by Mia Ferris
Page 5.5in × 8.5in, facing, margins 0.7in/0.8in/0.8in/0.6in
Chapters: 2
  content/01-the-jar.md            85 words, 1 heading, 0 images   “The Jar”
  content/02-the-walk-home.md      83 words, 1 heading, 0 images   “The Walk Home”
  168 words and 0 images in all
Problems: none
Built build/the-moon-in-a-jar.pdf — 5 pages in … ms
$ echo $?
0
```

`Problems: none`, and `0`. You have written a book.

## When something else goes wrong

A few more messages you may meet, and what they mean:

| What it says | What happened |
|---|---|
| `booker: the-moon-jar: there is no folder here` | The path is wrong, or you are in the wrong directory. `ls` to see where you are. |
| ``booker: the-moon-jar: there is already a book here; `booker new` will not write over it`` | You ran `booker new` on a folder that already has a book in it. Booker will never overwrite your work. Pick another name. |
| ``warning[BK-FORMAT-001] no `book.toml` in this folder`` | You built a folder that is not a book. Booker used its defaults so you would still get something. |
| ``error[BK-FORMAT-002] `book.toml` is not valid TOML`` | A quote or a bracket is unclosed. The line and column in the message is where it gave up. |
| ``error[BK-FORMAT-004] `page.size`: `a55` is not a page size`` | A value Booker could not understand. The message lists what it will accept. |
| ``warning[BK-REF-003] this project has no text in it yet`` | `content/` is empty, or nothing in `chapters` exists. |

Every one of these names a file and a line, and none of them is a crash.

## Where to keep it

The book folder is plain text, which means git works on it properly — you can see
exactly which sentences changed between two drafts. Booker already wrote a `.gitignore`
that keeps the PDF and the cache out, so this is all it takes:

```console ignore
$ git init
$ git add .
$ git commit -m "The Moon in a Jar, first draft"
```

The same property is why you can point an AI assistant at the folder and ask it to fix
your spelling or restructure a chapter: it is reading and writing the same Markdown you
are. The `AGENTS.md` file Booker wrote is a note to exactly that sort of visitor,
explaining what each file is and what not to touch.

## What Booker cannot do yet

Honesty is more useful here than enthusiasm. This tutorial covers essentially
everything the `booker` command does; [the next one](02-the-app.md) does the same in
Booker's window, with a live preview. **There is no styling beyond the built-in theme,
page size and margins, no choice of fonts, no cover, no HTML or EPUB output, and no way
to place an image at a particular spot on a page.** Images in `assets/images/` can be
referenced from Markdown and will appear in the flow of the text, but nothing more.

All of that is being built, and each piece arrives with a tutorial of its own — see
[`index.md`](index.md) for the list as it grows, and
[`../PLAN.md`](../PLAN.md) if you want to know the order.

What exists already is the part everything else stands on: your words in a folder you
own, and a PDF that looks like a book.

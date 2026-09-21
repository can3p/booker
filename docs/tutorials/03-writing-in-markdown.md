# Writing a book in Markdown

The first two tutorials had you type a few paragraphs. This one is about everything else
a book is made of — chapter titles, *italics*, quotation marks and dashes, a break
between scenes, a picture, a table, a page that starts in a particular place, a
reference from one chapter to another — and how to write each of them so that Booker
sets it properly.

The book is a short story in two chapters, *The Lighthouse Keeper's Cat*. It takes about
fifteen minutes, and you need the `booker` command from [the first tutorial](01-your-first-book.md).

**Markdown** is the way Booker books are written: plain text, where a few symbols mean
something — a `#` at the start of a line makes a heading, `*stars*` around a word make it
italic. It is readable as it is, in any editor, which is the point: your book is never
locked inside a program.

---

## 1. Make the book

```console
$ booker new the-lighthouse --title "The Lighthouse Keeper's Cat"
Created a `novel` book in the-lighthouse
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
  booker build the-lighthouse
$ cd the-lighthouse
$ rm content/01-the-first-chapter.md
```

The sample chapter is gone; we will write two of our own. Replace `book.toml` with this:

```toml file=book.toml
title = "The Lighthouse Keeper's Cat"
author = "Sam Reed"
language = "en"

chapters = [
    "content/01-the-storm.md",
    "content/02-the-morning.md",
]

[page]
size = "5.5x8.5in"
```

## 2. The first chapter: words, and the space between them

Create `content/01-the-storm.md`:

```markdown file=content/01-the-storm.md
# The Storm {#storm}

The cat's name was *Biscuit*, and she did not like storms. "It's only wind," the keeper told her -- but she had heard that before.

By midnight the lamp was the only light for forty miles. The keeper climbed the stairs twice an hour --- *two hundred and twelve steps* --- and Biscuit climbed them with him.

![The lamp, turning](assets/images/lamp.svg){width=60%}

---

When the wind finally dropped, the sea was the colour of old spoons.
```

Line by line, this is what each part means:

- **`# The Storm`** — a line starting with `#` is a heading, and the first one in a file
  is the chapter's title. The `{#storm}` after it gives the chapter a name other parts
  of the book can point at; we will use it in the next chapter. It does not appear on
  the page.
- **A blank line starts a new paragraph.** Lines that simply follow each other belong to
  the same paragraph — Markdown joins them — so you can break your lines wherever you
  like while you write.
- **`*Biscuit*`** is italic. Two stars, `**like this**`, would be bold.
- **Quotation marks and dashes are typed plainly.** Type `"` and `'` and Booker prints
  curly quotes — “ ” and ’ — the right way round. Two hyphens, `--`, become an en dash
  (–); three, `---`, an em dash (—). You never need to find those characters on your
  keyboard.
- **`![The lamp, turning](assets/images/lamp.svg){width=60%}`** is a picture. The words
  in the square brackets become its caption, the part in round brackets is where the file
  is, and `{width=60%}` says how wide to draw it — here, 60% of the width of the text.
  Leave it off and the picture is drawn at 80%.
- **`---` on a line of its own** is a break between scenes. In a novel it is printed as a
  small ornament, ⁂, with space around it — the signal that time has passed.

The picture needs a file. Normally you would copy a photograph or a scan into
`assets/images/`; for this tutorial, here is a small drawing written as text (an SVG
file), so you can create it the same way as everything else. Create
`assets/images/lamp.svg`:

```svg file=assets/images/lamp.svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 300">
  <rect width="400" height="300" rx="24" fill="#1f2a4d"/>
  <circle cx="70" cy="60" r="3" fill="#fff"/>
  <circle cx="330" cy="50" r="2.5" fill="#fff"/>
  <circle cx="300" cy="120" r="2" fill="#fff"/>
  <circle cx="110" cy="130" r="2" fill="#fff"/>
  <path d="M160 90 h80 v20 q25 10 25 40 v90 q0 25 -25 25 h-80 q-25 0 -25 -25 v-90 q0 -30 25 -40 z"
        fill="#bfe3d0" fill-opacity="0.55" stroke="#e8f6ee" stroke-width="4"/>
  <rect x="155" y="78" width="90" height="16" rx="4" fill="#c9a86a"/>
  <circle cx="200" cy="185" r="42" fill="#fff4c2"/>
  <circle cx="186" cy="172" r="7" fill="#efe0a0"/>
  <circle cx="214" cy="198" r="5" fill="#efe0a0"/>
</svg>
```

## 3. The second chapter: pointing, tables, and new pages

Create `content/02-the-morning.md`:

```markdown file=content/02-the-morning.md
# The Morning

In the morning there was a ship on the rocks, and three sailors on the shore. They had followed the light, just as the keeper had said they would ([see the storm](#storm)).

| Who             | What they ate |
|:----------------|--------------:|
| The sailors     | Porridge      |
| Biscuit         | Kippers       |

::: page-break
:::

## A letter {break-before=page}

The captain wrote to say thank you. He spelled Biscuit's name ~~Biskit~~ correctly, which she appreciated.
```

- **`[see the storm](#storm)`** is a link to the chapter we named `{#storm}`. In the PDF
  it can be clicked, and it takes the reader there. Any heading can be named this way,
  and so can a passage (below).
- **The table** is drawn with `|` between the columns and a line of dashes under the
  first row. The colons in that line say which way each column lines up: `:---` on the
  left, `---:` on the right. Booker sets it the way a book sets a table — a rule above,
  a rule under the header, a rule below, and no boxes.
- **`::: page-break`** followed by a line with just `:::` starts a new page there.
- **`{break-before=page}`** after a heading does the same thing for that heading — here
  it is a second way of asking for the page break just above it. Booker notices that the
  two ask for the same page, and does not leave a blank one.
- **`~~Biskit~~`** is struck through.

## 4. Build it

```console
$ booker build .
The Lighthouse Keeper's Cat — format 1, language en
by Sam Reed
Page 5.5in × 8.5in, facing, margins 18mm/20mm/20mm/15mm
Chapters: 2
  content/01-the-storm.md          75 words, 1 heading, 1 image   “The Storm”
  content/02-the-morning.md        61 words, 2 headings, 0 images   “The Morning”
  136 words and 1 image in all
Problems: none
Built build/the-lighthouse-keeper-s-cat.pdf — 6 pages in … ms
```

Open `build/the-lighthouse-keeper-s-cat.pdf`. Page 1 is the table of contents. The first
chapter starts on page 3, a right-hand page, with the blank page 2 before it; its
paragraphs are indented, its quotation marks curly, its dashes long, and the picture sits
centred with its caption under it, followed by the ⁂ and the last sentence. The second
chapter starts on page 5, and page 6 begins with *A letter*.

## 5. Two small mistakes

Everyone mistypes. Change the first chapter so the picture's `width` is misspelled:

```markdown file=content/01-the-storm.md
# The Storm {#storm}

The cat's name was *Biscuit*, and she did not like storms. "It's only wind," the keeper told her -- but she had heard that before.

By midnight the lamp was the only light for forty miles. The keeper climbed the stairs twice an hour --- *two hundred and twelve steps* --- and Biscuit climbed them with him.

![The lamp, turning](assets/images/lamp.svg){widht=60%}

---

When the wind finally dropped, the sea was the colour of old spoons.
```

and the second so its link points at a name that does not exist:

```markdown file=content/02-the-morning.md
# The Morning

In the morning there was a ship on the rocks, and three sailors on the shore. They had followed the light, just as the keeper had said they would ([see the storm](#strom)).

| Who             | What they ate |
|:----------------|--------------:|
| The sailors     | Porridge      |
| Biscuit         | Kippers       |

::: page-break
:::

## A letter {break-before=page}

The captain wrote to say thank you. He spelled Biscuit's name ~~Biskit~~ correctly, which she appreciated.
```

`booker check` lays the book out and reports without writing the PDF:

```console
$ booker check .
Problems: 1 error, 1 warning
  content/01-the-storm.md:7:45: warning[BK-DOC-001] nothing reads the attribute `widht`; it is kept as written. Did you mean `width`? Booker reads: break-before, width
  content/02-the-morning.md:3:148: error[BK-REF-005] this links to `#strom`, but nothing in the book has that id. Did you mean `#storm`? An id is written after a heading, like `# The Garden {#garden}`.
$ echo $?
1
```

Each one names the file, the line and the column, and suggests what you probably meant.
The misspelled width is a *warning*: the picture is still drawn, just at the default
size. The broken link is an *error*: the words "see the storm" are still printed, but
they lead nowhere, which is not what you wrote them for.

Put both back:

```markdown file=content/01-the-storm.md
# The Storm {#storm}

The cat's name was *Biscuit*, and she did not like storms. "It's only wind," the keeper told her -- but she had heard that before.

By midnight the lamp was the only light for forty miles. The keeper climbed the stairs twice an hour --- *two hundred and twelve steps* --- and Biscuit climbed them with him.

![The lamp, turning](assets/images/lamp.svg){width=60%}

---

When the wind finally dropped, the sea was the colour of old spoons.
```

```markdown file=content/02-the-morning.md
# The Morning

In the morning there was a ship on the rocks, and three sailors on the shore. They had followed the light, just as the keeper had said they would ([see the storm](#storm)).

| Who             | What they ate |
|:----------------|--------------:|
| The sailors     | Porridge      |
| Biscuit         | Kippers       |

::: page-break
:::

## A letter {break-before=page}

The captain wrote to say thank you. He spelled Biscuit's name ~~Biskit~~ correctly, which she appreciated.
```

```console
$ booker check .
Problems: none
$ echo $?
0
```

## What else you can write

- **A heading inside a chapter** starts with `##`, and a smaller one with `###`. The
  table of contents lists chapter titles; to list sections too, add this to `book.toml`:

  ```toml
  [toc]
  depth = 2
  ```

- **Lists** are lines starting with `-` (bullets) or `1.` (numbers). **A quotation** is a
  paragraph whose lines start with `>`.
- **A name for any passage**, so a link can point at it: put it in square brackets and
  name it — `[the night of the storm]{#night}` — or wrap whole paragraphs:

  ```markdown
  ::: {#poem}
  The sea was grey,
  the sky was grey.
  :::
  ```

- **A line starting with a number and a full stop** — `1984. A good year.` — would be
  read as a numbered list. Put a backslash before the full stop, `1984\. A good year.`,
  and it stays a sentence.

## What Markdown in Booker cannot do yet

- **Footnotes** (`[^1]`) and **mathematics** are kept in your file but not printed yet;
  `booker check` says so, with the line, so nothing disappears silently.
- **Pictures go between paragraphs.** Wrapping text around a picture, or pinning one to
  a place on a page, comes later.
- **Classes** — `{.poem}` — are accepted and kept, but do not change how anything looks
  yet. Styling them is the next thing Booker learns.

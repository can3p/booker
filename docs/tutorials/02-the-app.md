# The Booker window

[Your first book](01-your-first-book.md) made a book from the command line. This one
makes the same kind of book in the Booker window: you install the application, open a
folder, watch the pages redraw as you type, export a PDF, and update to a new version.

It takes about fifteen minutes. If you have done the first tutorial you already have a
book to open; if you have not, this one makes its own and you do not need the `booker`
command at all.

**What you need:** the Booker installer for your computer, and a text editor if you want
to use one. A Booker book is plain text in a plain folder, so the window and your editor
can both have it open.

---

## 1. Install Booker

> **Not yet.** Booker has not been released, and the releases page below is still empty. Until it appears, run the window from a copy of the source instead —
> [`CONTRIBUTING.md`](../../CONTRIBUTING.md) says how — and pick this tutorial up at step 2.

Go to <https://github.com/can3p/booker/releases>, open the newest release, and download
the file for your computer:

| Your computer | The file |
|---|---|
| Mac, Apple Silicon (M1 and later) | `Booker_…_aarch64.dmg` |
| Mac, Intel | `Booker_…_x64.dmg` |
| Windows | `Booker_…_x64-setup.exe` |
| Linux | `Booker_…_amd64.AppImage`, `.deb` or `.rpm` |

Then install it the way you install anything else: open the `.dmg` and drag Booker to
Applications, run the `.exe`, or make the AppImage executable and run it.

**Your computer will warn you the first time.** Booker is not yet signed with the
certificates Apple and Microsoft sell, so:

- **macOS** says Booker "cannot be opened because it is from an unidentified developer".
  Right-click the application and choose **Open**, then **Open** again in the dialog.
  You only do this once.
- **Windows** shows a blue "Windows protected your PC" screen. Click **More info**, then
  **Run anyway**.

This has nothing to do with whether updates are safe: every update Booker downloads is
signed with our own key and refused if the signature does not match. It is only about
the first install.

## 2. Open a book

Start Booker. The window is empty, with one button: **Open a book…**

If you did the first tutorial, open `the-moon-jar`. If you did not, make a folder now —
anywhere you keep your work — with a `book.toml` in it:

```toml file=book.toml
title = "The Moon in a Jar"
language = "en"

[page]
size = "5.5x8.5in"
```

and a `content` folder containing one file:

```markdown file=content/01-the-jar.md
# The Jar

Mira kept the moon in a jam jar on the windowsill, where it turned slowly and
made a small square of light on the floor.

Her brother said it was only a firefly. Mira knew better, because the moon had
craters, and fireflies do not.
```

Now click **Open a book…** and choose the folder — the folder itself, not `book.toml`.

Three things appear:

- **On the left, the chapters.** One row per file, with its title and its word count.
  The title is the first heading in the file, so `The Jar` comes from the `# The Jar`
  line rather than from the file name.
- **In the middle, the text**, which you can edit. It is Markdown, shown the way it
  reads: `# The Jar` is set large, `*a word*` in italics, and the `#` and `*` themselves
  stay in the text, faded, so you can always see what you typed.
- **On the right, the pages**, drawn exactly as they will print, fitted to the width of
  the pane. The buttons above them switch between **Width**, a whole **Page**, and a
  size of your choosing with **−** and **+**; the number in between takes you back to
  actual size.

Along the bottom: the book's title, how many chapters and words it has, how many pages
that came to, and whether everything is saved.

**If the window says the folder could not be read**, you chose a folder with no
`book.toml` in it. Booker will open a book that has mistakes *in* it — that is when you
most need to see them — but a folder that is not a book at all is not a book.

## 3. Write, and watch the pages

Click into the text and add a paragraph to the end of the chapter:

```markdown
The jar had a lid with holes punched in it, which Mira had done herself with a
nail, for the moon to breathe.
```

Two things happen without you asking. The word count on the left goes up, and the page
on the right redraws with your sentence set into it — hyphenated and justified, the way
a printed book is.

You never pressed save. Booker writes the chapter to disk about half a second after you
stop typing, and the bottom right corner says `saved` when it has. This matters more
than it sounds: it means the file on disk is almost always the file you are looking at,
so anything else that reads your book — git, your editor, a spell-checker — sees your
current words.

### Between the text and the pages

**Click a line on a page, and the text jumps to it**: the chapter it belongs to opens if
it was not already open, and the cursor lands on that line. It is the quickest way to
get from "that sentence looks wrong" to the sentence.

It works the other way too. **Move the cursor in the text, and the pages follow**: when
you rest on a line, the page it is printed on scrolls into view, and a small blue mark
beside the text shows where on the page that line landed.

## 4. Add a chapter

Below the chapter list is **+ Add a chapter**. Click it, type `The Garden`, and press
Enter.

A new chapter appears in the list, right after the one you had open, and opens in the
editor with its title already in it: `# The Garden`. Write a sentence or two under it.

Look at the pages now. With two chapters, the book has grown a **table of contents** on
its first page, and each chapter **starts on a right-hand page**, the way printed novels
do — so there are blank pages in between, left without a page number, exactly as a
printer would leave them. Nobody asked for either; they are what a book looks like.

When a chapter is selected, four small buttons appear under it:

- **↑** and **↓** move it earlier or later in the book.
- **Rename** changes its title — the `#` heading at the top of the file. (You can also
  double-click a chapter to rename it.)
- **Remove** takes it out of the book. Booker asks first, and even then **nothing is
  deleted**: the file stays in `content/`, only the book stops including it.

Every one of these is a small change to `book.toml`, which now lists the chapters in
order. Open it in your text editor and you will find a `chapters = [ … ]` list — with
your own comments, if you wrote any, exactly where you left them.

## 5. Edit it somewhere else at the same time

Leave Booker open. Open `content/01-the-jar.md` in your own text editor, change a word,
and save.

The Booker window follows. The text, the word count and the pages all update, and you
did not have to close anything or press reload.

This is deliberate, and it is the reason Booker keeps your book as plain files rather
than in a format of its own. Your book is a folder: you can edit it in VS Code, commit
it with git, switch branches, or point an AI assistant at it and ask it to fix the
spelling in every chapter. Booker notices and catches up. Thirty files changing at once
costs one redraw, not thirty.

### When both of you change the same chapter

Booker saves half a second after you stop typing, so this is rare — but try it. Type a
few words into the chapter in Booker, and *immediately*, before it saves, change the
same file in your text editor and save it there.

Booker does not pick one and throw the other away. The editor is replaced by both
versions side by side — **Yours, not saved yet** and **On disk now** — and nothing is
saved until you choose:

- **Keep mine** saves your version over the one on disk.
- **Take the one on disk** puts their version in the editor and lets yours go.
- **Keep both** leaves the file as it is on disk and adds your version as a new chapter
  right after it, in a file of its own — `content/the-jar-my-version.md` — so you can
  merge the two by hand. Your text is kept exactly as you wrote it, so the list shows two
  chapters called *The Jar*; the second one is yours. Remove it when you have merged.

This is also what happens if an assistant working in your book folder rewrites a
chapter while you are in the middle of it.

## 6. See what is wrong

Along the bottom is a line that says **Problems: none**. Click it to open the panel.

Now break something on purpose. In your text editor, change `book.toml` so the page
size is nonsense:

```toml
[page]
size = "big"
```

Save, and look at the Booker window. The problems line turns red, and the panel names
the file, the line and what is wrong with it.

Put `size = "5.5x8.5in"` back and the problem disappears.

Every problem Booker can find is reported this way — with the file and the line, never
as a message you have to guess about: a picture that is not there, a link to a heading
that does not exist, a chapter file that prints nothing. The same list, word for word,
is what `booker check` prints in a terminal.

## 7. Export the PDF

**File → Export PDF…**, or the **Export PDF…** button, then choose where to put it.

You now have a PDF you can print, email, or send to a printer. It is the same PDF
`booker build` makes from the command line — the window and the terminal use the same
layout engine, so a book cannot look different depending on which one you used.

## 8. Update to a new version

Booker updates itself. Choose **Check for Updates…** — on a Mac it is under the
**Booker** menu, elsewhere under **Help**.

If there is a newer version, Booker says so, downloads it, and installs it when you
restart. If there is not, it says Booker is up to date. To see which version you are
running, choose **About Booker** from the same menu.

Every update is signed, and an update whose signature does not match is refused rather
than installed. That is true even though the installer you downloaded in step 1 was not
signed by Apple or Microsoft: the two are separate things.

---

## What the window cannot do yet

So that you do not go hunting:

- **Nothing about how the book looks can be changed from the window.** The look comes
  from the book's theme — `novel`, `picture-book`, `poetry` or `paper`, set with `theme`
  in `book.toml` — and the page size and margins there. Choosing fonts, styling headings
  and page rules all come later.
- **Pictures go between paragraphs.** Placing one at a particular spot on a page comes
  later; [the Markdown tutorial](03-writing-in-markdown.md) shows what you can do now.
- **There is one window.** Opening a second book replaces the first.

What you *can* do, today, is write a book in it and get a good-looking PDF out — which
is the part that has to work before any of the rest is worth having.

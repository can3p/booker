import { beforeEach, describe, expect, it, vi } from "vitest";

/**
 * The store's one promise: text somebody typed is never thrown away
 * without them choosing to (`AGENTS.md` §7). These run the store against a
 * fake core, so each case is exactly the race it describes — an agent
 * rewriting the open chapter between a keystroke and its save.
 */

vi.mock("./ipc", () => ({
  readChapter: vi.fn(),
  saveChapter: vi.fn(),
  reloadProject: vi.fn(),
  compile: vi.fn(),
  addChapter: vi.fn(),
  moveChapter: vi.fn(),
  renameChapter: vi.fn(),
  removeChapter: vi.fn(),
  sourceAt: vi.fn(),
  pagesAt: vi.fn(),
}));

import { Book } from "./book.svelte";
import * as ipc from "./ipc";

const PATH = "content/01-the-jar.md";

function info(chapters: string[] = [PATH]) {
  return {
    project: { root: "/books/moon" },
    config: {},
    chapters: chapters.map((path) => ({ path, title: "The Jar", words: 3, headings: 1, images: 0 })),
    diagnostics: [],
    revision: 1n,
  } as never;
}

const mocked = vi.mocked(ipc);

/** A book open on `PATH`, whose file says `onDisk`. */
function openBook(onDisk: string): Book {
  const book = new Book();
  book.info = info();
  book.selected = PATH;
  book.text = onDisk;
  book.base = onDisk;
  return book;
}

beforeEach(() => {
  vi.resetAllMocks();
  mocked.compile.mockResolvedValue({ revision: 1n, pages: [], diagnostics: [], duration_ms: 1n } as never);
  mocked.reloadProject.mockResolvedValue(info());
});

describe("an edit made outside while the window has unsaved typing", () => {
  it("is offered side by side with the typing, and nothing is saved", async () => {
    const book = openBook("# The Jar\n");
    book.edited("# The Jar\n\nMy new paragraph.\n");
    mocked.readChapter.mockResolvedValue({ text: "# The Jar\n\nTheir fix.\n" } as never);

    await book.reloadFromDisk();

    expect(book.conflict).toEqual({
      path: PATH,
      disk: "# The Jar\n\nTheir fix.\n",
      mine: "# The Jar\n\nMy new paragraph.\n",
    });
    expect(book.text, "the typing is still in the editor").toBe("# The Jar\n\nMy new paragraph.\n");
    await vi.waitFor(() => Promise.resolve()); // let a stray autosave run, if there were one
    expect(mocked.saveChapter).not.toHaveBeenCalled();
  });

  it("is simply taken when there was no unsaved typing", async () => {
    const book = openBook("# The Jar\n");
    mocked.readChapter.mockResolvedValue({ text: "# The Jar\n\nTheir fix.\n" } as never);

    await book.reloadFromDisk();

    expect(book.conflict).toBeNull();
    expect(book.text).toBe("# The Jar\n\nTheir fix.\n");
  });

  it("is no conflict when the file already says what the window holds", async () => {
    const book = openBook("# Old\n");
    book.edited("# Same\n");
    mocked.readChapter.mockResolvedValue({ text: "# Same\n" } as never);

    await book.reloadFromDisk();

    expect(book.conflict).toBeNull();
    expect(book.unsaved).toBe(false);
  });
});

describe("a save that finds the file changed", () => {
  it("becomes a conflict instead of a write", async () => {
    const book = openBook("# The Jar\n");
    book.edited("# Mine\n");
    mocked.saveChapter.mockResolvedValue({
      outcome: "conflict",
      disk: { text: "# Theirs\n" },
      mine: "# Mine\n",
    } as never);

    // Switching chapters saves the open one first.
    await book.select("content/02-the-garden.md");

    expect(mocked.saveChapter).toHaveBeenCalledWith(PATH, "# Mine\n", "# The Jar\n");
    expect(book.conflict).toEqual({ path: PATH, disk: "# Theirs\n", mine: "# Mine\n" });
    expect(book.selected, "a chapter with a conflict stays open until it is resolved").toBe(PATH);
    expect(mocked.readChapter).not.toHaveBeenCalled();
  });
});

describe("the three ways out of a conflict", () => {
  function conflicted(): Book {
    const book = openBook("# The Jar\n");
    book.unsaved = true;
    book.text = "# Mine\n";
    book.conflict = { path: PATH, disk: "# Theirs\n", mine: "# Mine\n" };
    return book;
  }

  it("keep mine saves over what is on disk now", async () => {
    const book = conflicted();
    mocked.saveChapter.mockResolvedValue({ outcome: "saved", info: info() } as never);

    await book.keepMine();

    expect(mocked.saveChapter).toHaveBeenCalledWith(PATH, "# Mine\n", "# Theirs\n");
    expect(book.conflict).toBeNull();
    expect(book.unsaved).toBe(false);
  });

  it("take theirs puts their text in the editor and writes nothing", async () => {
    const book = conflicted();

    await book.takeTheirs();

    expect(book.text).toBe("# Theirs\n");
    expect(book.unsaved).toBe(false);
    expect(mocked.saveChapter).not.toHaveBeenCalled();
  });

  it("keep both adds mine as a chapter after theirs and opens it", async () => {
    const book = conflicted();
    const added = "content/the-jar-my-version.md";
    mocked.addChapter.mockResolvedValue([added, info([PATH, added])] as never);
    mocked.readChapter.mockResolvedValue({ path: added, text: "# Mine\n" } as never);

    await book.keepBoth();

    expect(mocked.addChapter).toHaveBeenCalledWith(PATH, "The Jar (my version)", "# Mine\n");
    expect(book.conflict).toBeNull();
    expect(book.selected).toBe(added);
    expect(book.text).toBe("# Mine\n");
    expect(mocked.saveChapter).not.toHaveBeenCalled();
  });
});

describe("the chapter tree", () => {
  it("opens a chapter added after the open one", async () => {
    const book = openBook("# The Jar\n");
    const added = "content/the-garden.md";
    mocked.addChapter.mockResolvedValue([added, info([PATH, added])] as never);
    mocked.readChapter.mockResolvedValue({ path: added, text: "# The Garden\n\n" } as never);

    await book.addChapter("The Garden");

    expect(mocked.addChapter).toHaveBeenCalledWith(PATH, "The Garden");
    expect(book.selected).toBe(added);
  });

  it("lets go of a chapter taken out of the book", async () => {
    const book = openBook("# The Jar\n");
    mocked.removeChapter.mockResolvedValue(info([]));

    await book.removeChapter(PATH);

    expect(book.selected).toBeNull();
    expect(book.info?.chapters).toEqual([]);
  });
});

describe("between the pages and the text", () => {
  it("a click on page 3 opens chapter 2 at line 14", async () => {
    const book = openBook("# The Jar\n");
    const second = "content/02-the-garden.md";
    book.info = info([PATH, second]);
    const reveal = vi.fn();
    book.attachEditor({ setText() {}, cursor: () => ({ line: 1, column: 1 }), reveal, destroy() {} });
    mocked.sourceAt.mockResolvedValue({ file: second, line: 14, column: 3, span: null });
    mocked.readChapter.mockResolvedValue({ path: second, text: "# The Garden\n" } as never);

    await book.showSource(2, 40.5, 120.25);

    expect(mocked.sourceAt).toHaveBeenCalledWith({ page: 2, x: "40.5mm", y: "120.25mm" });
    expect(book.selected).toBe(second);
    expect(reveal).toHaveBeenCalledWith(14, 3);
  });

  it("a click on a margin changes nothing", async () => {
    const book = openBook("# The Jar\n");
    mocked.sourceAt.mockResolvedValue(null);

    await book.showSource(0, 1, 1);

    expect(book.selected).toBe(PATH);
    expect(mocked.readChapter).not.toHaveBeenCalled();
  });

  it("the cursor, once it rests, is found on its page", async () => {
    vi.useFakeTimers();
    try {
      const book = openBook("# The Jar\n");
      book.layout = { revision: 1n, pages: [], diagnostics: [], duration_ms: 1n } as never;
      mocked.pagesAt.mockResolvedValue([{ page: 4, x: "20mm", y: "1in" }]);

      book.cursorMoved(7, 1);
      book.cursorMoved(8, 2);
      await vi.runAllTimersAsync();

      expect(mocked.pagesAt).toHaveBeenCalledTimes(1);
      expect(mocked.pagesAt).toHaveBeenCalledWith({ file: PATH, line: 8, column: 2, span: null });
      expect(book.cursorSpot?.page).toBe(4);
      expect(book.cursorSpot?.y).toBeCloseTo(25.4);
    } finally {
      vi.useRealTimers();
    }
  });
});

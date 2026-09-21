/**
 * What the window knows about the book it is showing.
 *
 * One store, because every pane shows a view of the same project and they
 * must never disagree: the sidebar's word count, the status bar's page
 * count and the problems panel are all this object.
 *
 * Nothing here is authoritative. The project on disk is, and every field
 * below is replaced wholesale by what the core last said — which is what
 * makes an outside edit a reload rather than a merge (`AGENTS.md` §7).
 *
 * The one exception is text typed and not yet saved. That exists only
 * here, for up to `AUTOSAVE_MS`, and it is never thrown away without the
 * person choosing to: when the file changed underneath it, the store holds
 * both versions in `conflict` and saves nothing until they choose.
 */

import type { ChapterText } from "./bindings/ChapterText";
import type { ChapterEditor } from "./editor/markdown";
import type { CompileResult } from "./bindings/CompileResult";
import type { ProjectInfo } from "./bindings/ProjectInfo";
import type { SaveOutcome } from "./bindings/SaveOutcome";

import * as ipc from "./ipc";
import { mm, toMm } from "./units";

/** How long after the last keystroke a chapter is written to disk.
 *
 *  Short, because saving eagerly is what stops an outside edit from meeting
 *  an unsaved buffer — the case we would otherwise have to resolve by
 *  asking somebody to choose between two versions (`AGENTS.md` §7). */
const AUTOSAVE_MS = 400;

/** How long the cursor rests before the preview is asked to follow it. */
const FOLLOW_MS = 150;

/** A place on a page, in millimetres from its top-left corner. */
export interface PageSpot {
  /** 0-based. */
  page: number;
  x: number;
  y: number;
}

/** The open chapter changed on disk while it had unsaved typing. */
export interface Conflict {
  path: string;
  /** What is on disk now: another editor's, or an agent's. */
  disk: string;
  /** What this window was holding. */
  mine: string;
}

export class Book {
  /** The open project, or null when none is. */
  info = $state<ProjectInfo | null>(null);
  /** The chapter the editor is showing, as a project-relative path. */
  selected = $state<string | null>(null);
  /** That chapter's text, as edited. */
  text = $state("");
  /** That chapter's text as the window last read or wrote it — what a save
   *  expects to find on disk. When the disk says something else, somebody
   *  else changed the file. */
  base = $state("");
  /** Both versions, when the open chapter changed underneath unsaved
   *  typing. While this is set nothing is saved: the person chooses. */
  conflict = $state<Conflict | null>(null);
  /** The most recent layout. */
  layout = $state<CompileResult | null>(null);
  /** Folders opened before, newest first. */
  recent = $state<string[]>([]);

  /** Something went wrong that is not about the book's content. */
  failure = $state<string | null>(null);
  /** A long operation is running: opening, exporting. */
  busy = $state(false);
  /** There are keystrokes not yet written to disk. */
  unsaved = $state(false);

  /** Where the cursor is in the open chapter, 1-based. */
  cursor = $state<{ line: number; column: number } | null>(null);
  /** Where the cursor's line landed on the pages, for the preview to show. */
  cursorSpot = $state<PageSpot | null>(null);

  #saveTimer: ReturnType<typeof setTimeout> | null = null;
  #followTimer: ReturnType<typeof setTimeout> | null = null;
  /** The editor on screen, so a click on a page can move its cursor. */
  #editor: ChapterEditor | null = null;

  /** Everything wrong with the book: what loading found, plus what laying
   *  it out found. Sorted by severity so the worst is read first. */
  problems = $derived.by(() => {
    const fromProject = this.info?.diagnostics ?? [];
    const fromLayout = this.layout?.diagnostics ?? [];
    const order = { error: 0, warning: 1, info: 2 } as const;
    return [...fromProject, ...fromLayout].sort(
      (a, b) => (order[a.severity] ?? 3) - (order[b.severity] ?? 3),
    );
  });

  get pageCount(): number {
    return this.layout?.pages.length ?? 0;
  }

  get wordCount(): number {
    return (this.info?.chapters ?? []).reduce((total, c) => total + c.words, 0);
  }

  async loadRecent(): Promise<void> {
    try {
      this.recent = await ipc.recentProjects();
    } catch {
      // A missing list of recent folders is not worth telling anyone about.
      this.recent = [];
    }
  }

  /** Ask the core what is open, on startup. */
  async restore(): Promise<void> {
    try {
      const current = await ipc.projectInfo();
      if (current) await this.#adopt(current);
    } catch (error: unknown) {
      this.failure = String(error);
    }
    await this.loadRecent();
  }

  async open(root: string): Promise<void> {
    this.busy = true;
    this.failure = null;
    try {
      await this.#adopt(await ipc.openProject(root));
      await this.loadRecent();
    } catch (error: unknown) {
      // The folder could not be read at all. A book with *faults* in it
      // opens normally and reports them as problems.
      this.failure = String(error);
      this.info = null;
    } finally {
      this.busy = false;
    }
  }

  async close(): Promise<void> {
    await this.#flush();
    await ipc.closeProject();
    this.info = null;
    this.selected = null;
    this.text = "";
    this.base = "";
    this.conflict = null;
    this.layout = null;
    this.failure = null;
  }

  /** Show a chapter in the editor, writing away whatever was there. */
  async select(path: string): Promise<void> {
    if (this.selected === path) return;
    await this.#flush();
    // A chapter with an unresolved conflict stays open until it is resolved.
    if (this.conflict) return;
    try {
      const chapter: ChapterText = await ipc.readChapter(path);
      this.selected = chapter.path;
      this.text = chapter.text;
      this.base = chapter.text;
      this.unsaved = false;
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** The editor changed. Save shortly, unless more typing follows. */
  edited(text: string): void {
    this.text = text;
    this.unsaved = true;
    if (this.#saveTimer) clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(() => void this.#flush(), AUTOSAVE_MS);
  }

  /** The editor pane mounted (or went away). */
  attachEditor(editor: ChapterEditor | null): void {
    this.#editor = editor;
  }

  /** The cursor moved in the editor. Once it rests, find its page. */
  cursorMoved(line: number, column: number): void {
    this.cursor = { line, column };
    if (this.#followTimer) clearTimeout(this.#followTimer);
    this.#followTimer = setTimeout(() => void this.#follow(), FOLLOW_MS);
  }

  /** Click-to-source: somebody clicked a page at `x`, `y` millimetres from
   *  its corner. Open the chapter that produced what is drawn there, and put
   *  the cursor on it. A click on a margin or a page number does nothing. */
  async showSource(page: number, x: number, y: number): Promise<void> {
    try {
      const location = await ipc.sourceAt({ page, x: mm(x), y: mm(y) });
      if (!location) return;
      const file = String(location.file);
      if (file !== this.selected) await this.select(file);
      if (this.selected !== file) return; // a conflict kept the old chapter
      this.#editor?.reveal(location.line, location.column);
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** Cursor-to-page: where the cursor's line landed, for the preview. */
  async #follow(): Promise<void> {
    if (!this.selected || !this.cursor || !this.layout) return;
    try {
      const points = await ipc.pagesAt({
        file: this.selected,
        line: this.cursor.line,
        column: this.cursor.column,
        span: null,
      });
      const first = points[0];
      this.cursorSpot = first
        ? { page: first.page, x: toMm(String(first.x)), y: toMm(String(first.y)) }
        : null;
    } catch {
      // Not knowing where the cursor is on the page is not worth a message.
      this.cursorSpot = null;
    }
  }

  // ---- a conflict, and the three ways out of it -------------------------

  /** Keep what this window had, over what is on disk. */
  async keepMine(): Promise<void> {
    const conflict = this.conflict;
    if (!conflict) return;
    this.conflict = null;
    this.text = conflict.mine;
    this.base = conflict.disk;
    this.unsaved = true;
    await this.#flush();
  }

  /** Take what is on disk, and let this window's version go. */
  async takeTheirs(): Promise<void> {
    const conflict = this.conflict;
    if (!conflict) return;
    this.conflict = null;
    this.text = conflict.disk;
    this.base = conflict.disk;
    this.unsaved = false;
    await this.compile();
  }

  /** Keep both: the file keeps what is on disk, and this window's version
   *  becomes a new chapter right after it. */
  async keepBoth(): Promise<void> {
    const conflict = this.conflict;
    if (!conflict || !this.info) return;
    const title = this.info.chapters.find((c) => c.path === conflict.path)?.title;
    try {
      const [added, info] = await ipc.addChapter(
        conflict.path,
        `${title ?? "Chapter"} (my version)`,
        conflict.mine,
      );
      this.info = info;
      this.conflict = null;
      this.text = conflict.disk;
      this.base = conflict.disk;
      this.unsaved = false;
      await this.select(added);
      await this.compile();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  // ---- the chapter tree -------------------------------------------------

  /** Add a chapter after the open one (or at the end) and open it. */
  async addChapter(title: string): Promise<void> {
    await this.#flush();
    try {
      const [added, info] = await ipc.addChapter(this.selected, title);
      this.info = info;
      await this.select(added);
      await this.compile();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  async moveChapter(path: string, index: number): Promise<void> {
    await this.#flush();
    await this.#treeEdit(() => ipc.moveChapter(path, index));
  }

  async renameChapter(path: string, title: string): Promise<void> {
    await this.#flush();
    await this.#treeEdit(() => ipc.renameChapter(path, title));
    // The heading is in the chapter's text, so the open copy must follow.
    if (path === this.selected && !this.unsaved) {
      const chapter = await ipc.readChapter(path);
      this.text = chapter.text;
      this.base = chapter.text;
    }
  }

  /** Take a chapter out of the book. The file stays in the folder. */
  async removeChapter(path: string): Promise<void> {
    await this.#flush();
    await this.#treeEdit(() => ipc.removeChapter(path));
    if (path === this.selected) {
      this.selected = null;
      this.text = "";
      this.base = "";
    }
  }

  async #treeEdit(edit: () => Promise<ProjectInfo>): Promise<void> {
    try {
      this.info = await edit();
      await this.compile();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** Lay the book out. Errors in the book arrive as problems, not failures. */
  async compile(): Promise<void> {
    if (!this.info) return;
    try {
      this.layout = await ipc.compile();
      // The text reflowed; the cursor's line may be somewhere else now.
      if (this.cursor) void this.#follow();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** Write a PDF to `destination`. */
  async exportPdf(destination: string): Promise<void> {
    if (!this.info) return;
    this.busy = true;
    try {
      await this.#flush();
      this.layout = await ipc.exportPdf(this.info, destination);
    } catch (error: unknown) {
      this.failure = String(error);
    } finally {
      this.busy = false;
    }
  }

  /** The folder changed underneath us. Read it again and lay it out again. */
  async reloadFromDisk(): Promise<void> {
    if (!this.info) return;
    try {
      // Ask the core to read the folder again — `projectInfo` would hand
      // back what it already had, which is exactly what is now out of date.
      const current = await ipc.reloadProject();
      if (!current) return;
      this.info = current;
      // Keep the chapter open if it still exists; otherwise let go of it —
      // unless it holds unsaved typing, which is kept, and saved as soon as
      // the person says where.
      const stillThere = current.chapters.some((c) => c.path === this.selected);
      if (this.selected && stillThere) {
        const chapter = await ipc.readChapter(this.selected);
        this.#arrived(this.selected, chapter.text);
      } else if (this.selected && this.unsaved && !this.conflict) {
        this.conflict = { path: this.selected, disk: "", mine: this.text };
      } else if (!this.unsaved) {
        this.selected = null;
        this.text = "";
        this.base = "";
      }
      await this.compile();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** The open chapter's file says `disk` now. Take it, unless that would
   *  drop typing the person has not saved yet. */
  #arrived(path: string, disk: string): void {
    if (disk === this.base) return; // nothing changed — our own echo, say
    if (!this.unsaved) {
      this.text = disk;
      this.base = disk;
      return;
    }
    if (disk === this.text) {
      // The same edit, made in both places: nothing to choose between.
      this.base = disk;
      this.unsaved = false;
      return;
    }
    if (this.#saveTimer) clearTimeout(this.#saveTimer);
    this.conflict = { path, disk, mine: this.text };
  }

  /** Take a fresh description of the project and lay it out. */
  async #adopt(info: ProjectInfo): Promise<void> {
    this.info = info;
    this.selected = null;
    this.text = "";
    this.base = "";
    this.conflict = null;
    this.layout = null;
    if (info.chapters.length > 0) {
      await this.select(info.chapters[0].path);
    }
    await this.compile();
  }

  /** Write the open chapter now, if it has unsaved changes. */
  async #flush(): Promise<void> {
    if (this.#saveTimer) {
      clearTimeout(this.#saveTimer);
      this.#saveTimer = null;
    }
    if (!this.unsaved || !this.selected || this.conflict) return;
    const path = this.selected;
    const text = this.text;
    try {
      const outcome: SaveOutcome = await ipc.saveChapter(path, text, this.base);
      if (outcome.outcome === "conflict") {
        this.conflict = { path, disk: outcome.disk.text, mine: outcome.mine };
        return;
      }
      this.info = outcome.info;
      this.base = text;
      // Typing may have continued while the save was in flight; only what
      // was written is saved.
      this.unsaved = this.text !== text;
      await this.compile();
    } catch (error: unknown) {
      // Leave `unsaved` set: the text is still only in the window, and the
      // next keystroke will try again.
      this.failure = String(error);
    }
  }
}

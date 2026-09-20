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
 */

import type { ChapterText } from "./bindings/ChapterText";
import type { CompileResult } from "./bindings/CompileResult";
import type { ProjectInfo } from "./bindings/ProjectInfo";

import * as ipc from "./ipc";

/** How long after the last keystroke a chapter is written to disk.
 *
 *  Short, because saving eagerly is what stops an outside edit from meeting
 *  an unsaved buffer — the case we would otherwise have to resolve by
 *  asking somebody to choose between two versions (`AGENTS.md` §7). */
const AUTOSAVE_MS = 400;

export class Book {
  /** The open project, or null when none is. */
  info = $state<ProjectInfo | null>(null);
  /** The chapter the editor is showing, as a project-relative path. */
  selected = $state<string | null>(null);
  /** That chapter's text, as edited. */
  text = $state("");
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

  #saveTimer: ReturnType<typeof setTimeout> | null = null;

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
    this.layout = null;
    this.failure = null;
  }

  /** Show a chapter in the editor, writing away whatever was there. */
  async select(path: string): Promise<void> {
    if (this.selected === path) return;
    await this.#flush();
    try {
      const chapter: ChapterText = await ipc.readChapter(path);
      this.selected = chapter.path;
      this.text = chapter.text;
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

  /** Lay the book out. Errors in the book arrive as problems, not failures. */
  async compile(): Promise<void> {
    if (!this.info) return;
    try {
      this.layout = await ipc.compile();
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
      const current = await ipc.projectInfo();
      if (!current) return;
      this.info = current;
      // Keep the chapter open if it still exists; otherwise let go of it.
      const stillThere = current.chapters.some((c) => c.path === this.selected);
      if (this.selected && stillThere) {
        const chapter = await ipc.readChapter(this.selected);
        this.text = chapter.text;
        this.unsaved = false;
      } else {
        this.selected = null;
        this.text = "";
      }
      await this.compile();
    } catch (error: unknown) {
      this.failure = String(error);
    }
  }

  /** Take a fresh description of the project and lay it out. */
  async #adopt(info: ProjectInfo): Promise<void> {
    this.info = info;
    this.selected = null;
    this.text = "";
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
    if (!this.unsaved || !this.selected) return;
    const path = this.selected;
    const text = this.text;
    try {
      this.info = await ipc.saveChapter(path, text);
      this.unsaved = false;
      await this.compile();
    } catch (error: unknown) {
      // Leave `unsaved` set: the text is still only in the window, and the
      // next keystroke will try again.
      this.failure = String(error);
    }
  }
}

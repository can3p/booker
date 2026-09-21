/**
 * The chapter editor: CodeMirror 6 with Markdown styled as you type.
 *
 * Headings are larger, emphasis is italic, strong is bold — and the marks
 * that make them (`#`, `*`, `_`, `>`) stay visible, dimmed. Hiding them, as
 * some editors do, moves the text under the cursor as it enters and leaves
 * a line, which confuses exactly the people this is for; seeing a faint
 * `#` costs nothing.
 *
 * The editor owns no state that matters. What the book says lives in the
 * store and on disk; this reports keystrokes out (`onChange`) and accepts
 * text in (`setText`), and setting text never reports it back — otherwise
 * a reload from disk would look like typing and be saved straight back.
 */

import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { Annotation, EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { tags } from "@lezer/highlight";

/** Marks a change that came from outside — a reload, a chapter switch —
 *  rather than from somebody typing. */
const external = Annotation.define<boolean>();

const styles = HighlightStyle.define([
  { tag: tags.heading1, fontSize: "1.6em", fontWeight: "600" },
  { tag: tags.heading2, fontSize: "1.3em", fontWeight: "600" },
  { tag: tags.heading3, fontSize: "1.1em", fontWeight: "600" },
  { tag: [tags.heading4, tags.heading5, tags.heading6], fontWeight: "600" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strong, fontWeight: "700" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  { tag: tags.quote, fontStyle: "italic", color: "var(--muted)" },
  { tag: tags.monospace, fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace" },
  { tag: [tags.link, tags.url], color: "var(--accent, #3b6ea5)" },
  // The marks themselves: present, but quiet.
  { tag: [tags.processingInstruction, tags.meta, tags.contentSeparator], color: "var(--muted)", opacity: "0.6" },
]);

const theme = EditorView.theme({
  "&": { height: "100%", fontSize: "15px" },
  ".cm-scroller": {
    fontFamily: '"Iowan Old Style", "Palatino Linotype", Georgia, serif',
    lineHeight: "1.65",
    padding: "1rem 0",
  },
  ".cm-content": { maxWidth: "40rem", margin: "0 auto", padding: "0 1.25rem" },
  "&.cm-focused": { outline: "none" },
  ".cm-line": { padding: "0" },
});

export interface ChapterEditor {
  /** Replace the text without reporting it as typing. Keeps the cursor
   *  where it was, as far as the new text allows. */
  setText(text: string): void;
  /** The cursor's line and column, 1-based as editors count. */
  cursor(): { line: number; column: number };
  /** Put the cursor at a line and column and scroll it into view. */
  reveal(line: number, column: number): void;
  destroy(): void;
}

export function createEditor(
  parent: HTMLElement,
  text: string,
  onChange: (text: string) => void,
  onCursor: (line: number, column: number) => void = () => {},
): ChapterEditor {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: text,
      extensions: [
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown({ base: markdownLanguage }),
        syntaxHighlighting(styles),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ spellcheck: "true", autocorrect: "on" }),
        theme,
        EditorView.updateListener.of((update) => {
          const typed = update.transactions.some((t) => !t.annotation(external));
          if (update.docChanged && typed) onChange(update.state.doc.toString());
          if (update.selectionSet) {
            const head = update.state.selection.main.head;
            const line = update.state.doc.lineAt(head);
            onCursor(line.number, head - line.from + 1);
          }
        }),
      ],
    }),
  });

  return {
    setText(next: string) {
      const current = view.state.doc.toString();
      if (next === current) return;
      const head = Math.min(view.state.selection.main.head, next.length);
      view.dispatch({
        changes: { from: 0, to: current.length, insert: next },
        selection: { anchor: head },
        annotations: external.of(true),
      });
    },
    cursor() {
      const head = view.state.selection.main.head;
      const line = view.state.doc.lineAt(head);
      return { line: line.number, column: head - line.from + 1 };
    },
    reveal(line: number, column: number) {
      const doc = view.state.doc;
      const target = doc.line(Math.min(Math.max(line, 1), doc.lines));
      const anchor = Math.min(target.from + Math.max(column, 1) - 1, target.to);
      view.dispatch({
        selection: { anchor },
        effects: EditorView.scrollIntoView(anchor, { y: "center" }),
        annotations: external.of(true),
      });
      view.focus();
    },
    destroy() {
      view.destroy();
    },
  };
}

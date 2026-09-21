<script lang="ts">
  /**
   * The editor pane: the open chapter in CodeMirror (`lib/editor`), or,
   * when the chapter changed on disk under unsaved typing, both versions
   * side by side until the person chooses (`Conflict.svelte`).
   *
   * The saving policy — eagerly, on a short timer — lives in the store,
   * not here. The editor reports keystrokes and the cursor, and shows
   * whatever text the store holds; a reload from disk arrives as new text
   * and is never mistaken for typing.
   */
  import { onDestroy } from "svelte";

  import type { Book } from "../book.svelte";
  import { createEditor, type ChapterEditor } from "../editor/markdown";
  import Conflict from "./Conflict.svelte";

  let { book }: { book: Book } = $props();

  let host = $state<HTMLElement | null>(null);
  let editor: ChapterEditor | null = null;

  // Stand the editor up once there is somewhere to put it; the text it
  // starts with is whatever the store holds at that moment.
  $effect(() => {
    if (!host || editor) return;
    editor = createEditor(
      host,
      book.text,
      (text) => book.edited(text),
      (line, column) => book.cursorMoved(line, column),
    );
    book.attachEditor(editor);
  });

  // The store's text changed from outside the editor: a chapter was opened,
  // or the file was reloaded. Typing also lands here, as the same text the
  // editor already has, and `setText` ignores that.
  $effect(() => {
    const text = book.text;
    editor?.setText(text);
  });

  onDestroy(() => {
    book.attachEditor(null);
    editor?.destroy();
    editor = null;
  });
</script>

<!-- One element, because the window's grid gives this pane one cell. -->
<div class="pane">
  {#if book.conflict}
    <Conflict {book} conflict={book.conflict} />
  {/if}
  <section class:hidden={!!book.conflict || !book.selected}>
    <div class="host" bind:this={host}></div>
  </section>
  {#if !book.selected && !book.conflict}
    <p class="empty">Choose a chapter.</p>
  {/if}
</div>

<style>
  .pane {
    display: grid;
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--border);
  }

  section {
    display: flex;
    min-width: 0;
    min-height: 0;
  }

  .hidden {
    display: none;
  }

  .host {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .empty {
    color: var(--muted);
    padding: 1rem 1.25rem;
    margin: 0;
  }
</style>

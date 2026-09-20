<script lang="ts">
  /**
   * The editor pane: a plain text area in Wave 1.
   *
   * Live Markdown styling, click-to-source and the chapter tree are Wave 2
   * track C, where CodeMirror 6 replaces this. What must survive that
   * change is the saving policy — eagerly, on a short timer — which lives
   * in the store rather than here.
   */
  import type { Book } from "../book.svelte";

  let { book }: { book: Book } = $props();
</script>

<section>
  {#if book.selected}
    <textarea
      spellcheck="true"
      value={book.text}
      oninput={(event) => book.edited(event.currentTarget.value)}
    ></textarea>
  {:else}
    <p class="empty">Choose a chapter.</p>
  {/if}
</section>

<style>
  section {
    display: flex;
    min-width: 0;
    border-right: 1px solid var(--border);
  }

  textarea {
    flex: 1;
    border: 0;
    resize: none;
    padding: 1rem 1.25rem;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 13px;
    line-height: 1.7;
    background: transparent;
    color: inherit;
  }

  textarea:focus {
    outline: none;
  }

  .empty {
    color: var(--muted);
    padding: 1rem 1.25rem;
  }
</style>

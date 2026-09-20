<script lang="ts">
  /** What the book is, in one line: where it lives, how big it is, and
   *  whether anything is still only in the window. */
  import type { Book } from "../book.svelte";

  let { book }: { book: Book } = $props();
</script>

<footer>
  {#if book.info}
    <span class="title">{book.info.config.title || "(untitled)"}</span>
    <span class="counts">
      {book.info.chapters.length}
      {book.info.chapters.length === 1 ? "chapter" : "chapters"} ·
      {book.wordCount} words ·
      {book.pageCount}
      {book.pageCount === 1 ? "page" : "pages"}
    </span>
    <span class="saved">
      {#if book.busy}
        working…
      {:else if book.unsaved}
        unsaved
      {:else}
        saved
      {/if}
    </span>
  {:else}
    <span class="counts">No book open</span>
  {/if}
</footer>

<style>
  footer {
    display: flex;
    gap: 1rem;
    align-items: baseline;
    border-top: 1px solid var(--border);
    padding: 0.3rem 0.75rem;
    font-size: 0.85em;
  }

  .title {
    font-weight: 600;
  }

  .counts {
    color: var(--muted);
  }

  .saved {
    margin-left: auto;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
</style>

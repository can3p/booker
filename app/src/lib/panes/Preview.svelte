<script lang="ts">
  /**
   * The preview pane.
   *
   * Wave 1 track C fills this in: page images over the `booker://`
   * protocol, only the visible pages rendered, zoom and scroll. Until then
   * it shows what the layout says about the book, which is enough to prove
   * the compile loop runs on every edit.
   */
  import type { Book } from "../book.svelte";

  let { book }: { book: Book } = $props();
</script>

<section>
  {#if book.layout}
    <ul class="pages">
      {#each book.layout.pages as page (page.index)}
        <li class="page">
          <span class="number">{page.label ?? page.index + 1}</span>
          <span class="size">{page.width} × {page.height}</span>
          {#if page.chapter}<span class="chapter">{page.chapter}</span>{/if}
        </li>
      {/each}
    </ul>
  {:else}
    <p class="empty">Nothing laid out yet.</p>
  {/if}
</section>

<style>
  section {
    overflow-y: auto;
    padding: 1rem;
    background: var(--sunken);
  }

  .pages {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.5rem;
  }

  .page {
    display: grid;
    gap: 0.15rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--background);
    padding: 0.5rem 0.6rem;
  }

  .number {
    font-variant-numeric: tabular-nums;
  }

  .size,
  .chapter {
    color: var(--muted);
    font-size: 0.85em;
  }

  .empty {
    color: var(--muted);
  }
</style>

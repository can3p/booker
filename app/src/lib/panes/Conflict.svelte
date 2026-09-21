<script lang="ts">
  /**
   * Two versions of one chapter, side by side: what this window was holding
   * and what is on disk now, because something else — another editor, an
   * agent, a branch checkout — changed the file before the typing was
   * saved. Nothing is saved while this is showing; the person chooses
   * (`AGENTS.md` §7: offer both versions rather than discard one).
   */
  import type { Book, Conflict } from "../book.svelte";

  let { book, conflict }: { book: Book; conflict: Conflict } = $props();

  let gone = $derived(conflict.disk === "");
</script>

<section class="conflict" aria-labelledby="conflict-title">
  <header>
    <h2 id="conflict-title">This chapter changed while you were writing</h2>
    <p>
      {#if gone}
        <code>{conflict.path}</code> was deleted or moved outside Booker. Your text is still here.
      {:else}
        Something outside Booker changed <code>{conflict.path}</code> before your latest typing
        was saved. Nothing has been saved or thrown away — choose what to keep.
      {/if}
    </p>
  </header>

  <div class="versions">
    <figure>
      <figcaption>Yours, not saved yet</figcaption>
      <pre>{conflict.mine}</pre>
    </figure>
    <figure>
      <figcaption>{gone ? "On disk now: nothing" : "On disk now"}</figcaption>
      <pre>{conflict.disk}</pre>
    </figure>
  </div>

  <footer>
    <button class="primary" onclick={() => book.keepMine()}>Keep mine</button>
    {#if !gone}
      <button onclick={() => book.takeTheirs()}>Take the one on disk</button>
      <button onclick={() => book.keepBoth()}>Keep both</button>
      <span class="hint">Keep both adds yours as a new chapter right after this one.</span>
    {/if}
  </footer>
</section>

<style>
  .conflict {
    display: grid;
    grid-template-rows: auto 1fr auto;
    min-height: 0;
  }

  header {
    padding: 0.75rem 1rem 0.25rem;
    border-left: 3px solid var(--error);
  }

  h2 {
    font-size: 0.95rem;
    margin: 0 0 0.3rem;
  }

  header p {
    margin: 0;
    color: var(--muted);
  }

  .versions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
    min-height: 0;
    padding: 0.5rem 1rem;
  }

  figure {
    display: grid;
    grid-template-rows: auto 1fr;
    margin: 0;
    min-height: 0;
  }

  figcaption {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    padding-bottom: 0.3rem;
  }

  pre {
    margin: 0;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.6rem;
    white-space: pre-wrap;
    font-size: 12px;
    line-height: 1.6;
  }

  footer {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.5rem 1rem 0.75rem;
  }

  .primary {
    font-weight: 600;
  }

  .hint {
    color: var(--muted);
    font-size: 0.85em;
  }
</style>

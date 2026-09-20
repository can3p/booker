<script lang="ts">
  /** The chapter list, or — with no book open — the folders opened before. */
  import type { Book } from "../book.svelte";

  let { book, onopen }: { book: Book; onopen: (root: string) => void } = $props();

  /** The last segment of a path, which is what a person calls the book. */
  function folderName(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }
</script>

<nav>
  {#if book.info}
    <h2>Chapters</h2>
    <ul>
      {#each book.info.chapters as chapter (chapter.path)}
        <li>
          <button
            class="chapter"
            class:selected={chapter.path === book.selected}
            onclick={() => book.select(chapter.path)}
          >
            <span class="name">{chapter.title ?? folderName(chapter.path)}</span>
            <span class="words">{chapter.words}w</span>
          </button>
        </li>
      {/each}
    </ul>
    {#if book.info.chapters.length === 0}
      <p class="empty">
        No chapters. Booker reads <code>content/*.md</code>, or whatever
        <code>book.toml</code> lists.
      </p>
    {/if}
  {:else if book.recent.length > 0}
    <h2>Recent</h2>
    <ul>
      {#each book.recent as path (path)}
        <li>
          <button class="chapter" onclick={() => onopen(path)} title={path}>
            <span class="name">{folderName(path)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</nav>

<style>
  nav {
    border-right: 1px solid var(--border);
    overflow-y: auto;
    padding: 0.75rem;
  }

  h2 {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin: 0 0 0.5rem;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .chapter {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    width: 100%;
    text-align: left;
    border: 0;
    border-radius: 4px;
    padding: 0.3rem 0.45rem;
  }

  .chapter:hover {
    background: var(--hover);
    color: inherit;
  }

  .chapter.selected {
    background: var(--selected);
    color: inherit;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .words {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .empty {
    color: var(--muted);
  }
</style>

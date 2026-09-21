<script lang="ts">
  /**
   * The chapter list — which is also where chapters are added, renamed,
   * moved and taken out of the book — or, with no book open, the folders
   * opened before.
   *
   * Every change here is a targeted edit of `book.toml`'s `chapters` list
   * (and, for a new chapter or a new title, of one chapter file), made by
   * the core. Nothing is ever deleted: taking a chapter out of the book
   * leaves its file in the folder.
   */
  import type { Book } from "../book.svelte";

  let { book, onopen }: { book: Book; onopen: (root: string) => void } = $props();

  /** What is being typed into, if anything: a new chapter's title, or a
   *  new title for the chapter at `path`. */
  let naming = $state<{ path: string | null; title: string } | null>(null);
  /** The chapter whose removal is waiting to be confirmed. */
  let removing = $state<string | null>(null);

  /** The last segment of a path, which is what a person calls the book. */
  function folderName(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }

  async function finishNaming() {
    const current = naming;
    naming = null;
    const title = current?.title.trim();
    if (!current || !title) return;
    if (current.path === null) await book.addChapter(title);
    else await book.renameChapter(current.path, title);
  }

  function keys(event: KeyboardEvent) {
    if (event.key === "Enter") void finishNaming();
    if (event.key === "Escape") naming = null;
  }

  /** Give the title field the focus as soon as it appears. */
  function focus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<nav>
  {#if book.info}
    <h2>Chapters</h2>
    <ul>
      {#each book.info.chapters as chapter, index (chapter.path)}
        <li>
          {#if naming?.path === chapter.path}
            <input
              class="title"
              bind:value={naming.title}
              onkeydown={keys}
              onblur={finishNaming}
              use:focus
              aria-label="New title for this chapter"
            />
          {:else}
            <button
              class="chapter"
              class:selected={chapter.path === book.selected}
              onclick={() => book.select(chapter.path)}
              ondblclick={() => (naming = { path: chapter.path, title: chapter.title ?? "" })}
              title={chapter.path}
            >
              <span class="name">{chapter.title ?? folderName(chapter.path)}</span>
              <span class="words">{chapter.words}w</span>
            </button>
          {/if}
          {#if chapter.path === book.selected && naming?.path !== chapter.path}
            <div class="tools">
              <button
                onclick={() => book.moveChapter(chapter.path, index - 1)}
                disabled={index === 0}
                title="Move up">↑</button
              >
              <button
                onclick={() => book.moveChapter(chapter.path, index + 1)}
                disabled={index === book.info.chapters.length - 1}
                title="Move down">↓</button
              >
              <button onclick={() => (naming = { path: chapter.path, title: chapter.title ?? "" })}
                >Rename</button
              >
              <button onclick={() => (removing = chapter.path)}>Remove</button>
            </div>
          {/if}
          {#if removing === chapter.path}
            <div class="confirm" role="alert">
              <p>
                Take this chapter out of the book? Its file stays in the folder, so nothing is
                lost.
              </p>
              <button
                onclick={() => {
                  removing = null;
                  void book.removeChapter(chapter.path);
                }}>Take it out</button
              >
              <button onclick={() => (removing = null)}>Keep it</button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
    {#if naming?.path === null}
      <input
        class="title"
        bind:value={naming.title}
        onkeydown={keys}
        onblur={finishNaming}
        use:focus
        placeholder="The new chapter's title"
        aria-label="The new chapter's title"
      />
    {:else}
      <button class="add" onclick={() => (naming = { path: null, title: "" })}>+ Add a chapter</button>
    {/if}
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

  .tools,
  .confirm {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    padding: 0.2rem 0.45rem 0.4rem;
  }

  .tools button,
  .confirm button,
  .add {
    font-size: 0.8em;
    padding: 0.05rem 0.45rem;
  }

  .confirm p {
    margin: 0 0 0.2rem;
    font-size: 0.85em;
  }

  .add {
    margin-top: 0.5rem;
    border: 0;
    color: var(--muted);
    background: transparent;
  }

  .add:hover {
    color: inherit;
    background: var(--hover);
  }

  .title {
    width: 100%;
    box-sizing: border-box;
    padding: 0.25rem 0.45rem;
    font: inherit;
  }
</style>

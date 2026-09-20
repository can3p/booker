<script lang="ts">
  /**
   * The preview: the book's pages, drawn.
   *
   * Two things make this cheap enough to scroll through a 200-page book.
   *
   * **Images come over the `booker://` protocol, not through IPC.** The
   * webview fetches them the way it fetches any image, so nothing is
   * base64-encoded through a JSON channel and the browser does the
   * scheduling.
   *
   * **Only the visible pages are asked for.** Every page keeps its place in
   * the scroll — the box is the right size before anything is drawn,
   * because the layout already told us how big the page is — and an
   * observer fills in the ones that come into view. Scrolling therefore
   * never reflows, and a page that scrolls past is never rendered.
   *
   * The revision is in each URL, so an edit changes every URL and the old
   * images are simply never asked for again (`app/src-tauri/src/protocol.rs`).
   */
  import { onMount } from "svelte";

  import type { Book } from "../book.svelte";
  import { pageImageUrl } from "../ipc";

  let { book }: { book: Book } = $props();

  /** How far outside the viewport a page is fetched, so scrolling finds it
   *  already drawn rather than blank. */
  const MARGIN = "600px";

  let zoom = $state(1);
  let visible = $state(new Set<number>());
  let observer: IntersectionObserver | null = null;

  /** A page is drawn at the window's pixel density, so it is sharp on a
   *  retina display and not needlessly large elsewhere. */
  let scale = $derived(zoom * (typeof devicePixelRatio === "number" ? devicePixelRatio : 1));

  // A new revision is a different book: everything on screen must be asked
  // for again, and nothing off screen should be.
  $effect(() => {
    void book.layout?.revision;
    visible = new Set();
  });

  onMount(() => {
    observer = new IntersectionObserver(
      (entries) => {
        const next = new Set(visible);
        for (const entry of entries) {
          const index = Number((entry.target as HTMLElement).dataset.page);
          if (entry.isIntersecting) next.add(index);
        }
        visible = next;
      },
      { rootMargin: MARGIN },
    );
    return () => observer?.disconnect();
  });

  /** Watch one page's box. Used as a Svelte action, so pages register
   *  themselves as they are created and unregister as they go. */
  function watch(node: HTMLElement) {
    observer?.observe(node);
    return {
      destroy() {
        observer?.unobserve(node);
      },
    };
  }

  /** Millimetres to CSS pixels, so the box is the page's real size before
   *  the image arrives. `Length` serialises as a string with its unit. */
  function toPixels(length: string): number {
    const match = /^(-?[\d.]+)(mm|cm|in|pt|px)$/.exec(length);
    if (!match) return 0;
    const value = Number(match[1]);
    const inches = { mm: 1 / 25.4, cm: 1 / 2.54, in: 1, pt: 1 / 72, px: 1 / 96 }[match[2]] ?? 0;
    return value * inches * 96;
  }
</script>

<section>
  <header>
    <span class="count">
      {book.pageCount}
      {book.pageCount === 1 ? "page" : "pages"}
    </span>
    <span class="zoom">
      <button onclick={() => (zoom = Math.max(0.25, zoom - 0.25))} aria-label="Zoom out">−</button>
      <span class="level">{Math.round(zoom * 100)}%</span>
      <button onclick={() => (zoom = Math.min(4, zoom + 0.25))} aria-label="Zoom in">+</button>
    </span>
  </header>

  <div class="scroller">
    {#if book.layout && book.layout.pages.length > 0}
      {#each book.layout.pages as page (page.index)}
        <figure
          class="page"
          data-page={page.index}
          use:watch
          style="width: {toPixels(page.width) * zoom}px; height: {toPixels(page.height) * zoom}px"
        >
          {#if visible.has(page.index)}
            <img
              src={pageImageUrl(book.layout.revision, page.index, scale)}
              alt="Page {page.label ?? page.index + 1}"
              loading="lazy"
            />
          {/if}
          <figcaption>{page.label ?? page.index + 1}</figcaption>
        </figure>
      {/each}
    {:else}
      <p class="empty">
        {#if book.problems.some((p) => p.severity === "error")}
          The book did not lay out. See the problems below.
        {:else}
          Nothing laid out yet.
        {/if}
      </p>
    {/if}
  </div>
</section>

<style>
  section {
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 0;
    background: var(--sunken);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border);
    font-size: 0.85em;
    color: var(--muted);
  }

  .zoom {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .zoom button {
    padding: 0 0.4rem;
    line-height: 1.4;
  }

  .level {
    font-variant-numeric: tabular-nums;
    min-width: 3ch;
    text-align: center;
  }

  .scroller {
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .page {
    position: relative;
    margin: 0;
    background: white;
    box-shadow: 0 1px 4px rgb(0 0 0 / 0.25);
    flex: none;
  }

  .page img {
    display: block;
    width: 100%;
    height: 100%;
  }

  figcaption {
    position: absolute;
    bottom: -1.25rem;
    left: 0;
    right: 0;
    text-align: center;
    font-size: 0.75em;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .empty {
    color: var(--muted);
  }
</style>

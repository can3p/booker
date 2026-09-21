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
  import { onMount, tick } from "svelte";

  import type { Book } from "../book.svelte";
  import { pageImageUrl } from "../ipc";
  import { PX_PER_MM, toPixels } from "../units";

  let { book }: { book: Book } = $props();

  /** How far outside the viewport a page is fetched, so scrolling finds it
   *  already drawn rather than blank. */
  const MARGIN = "600px";
  /** Room around a page inside the scroller, so "fit" leaves a gap. */
  const GUTTER = 32;

  /** Fit the page's width to the pane, fit a whole page in view, or a
   *  fixed size. Fit width is where a book is read from. */
  type Zoom = "width" | "page" | number;
  let mode = $state<Zoom>("width");
  let visible = $state(new Set<number>());
  let observer: IntersectionObserver | null = null;
  let scroller = $state<HTMLElement | null>(null);
  let paneWidth = $state(0);
  let paneHeight = $state(0);

  /** The first page stands for the book: fitting is by its size. */
  let first = $derived(book.layout?.pages[0] ?? null);

  let zoom = $derived.by(() => {
    if (typeof mode === "number") return mode;
    if (!first || paneWidth === 0) return 1;
    const byWidth = (paneWidth - GUTTER) / toPixels(first.width);
    if (mode === "width") return Math.max(0.1, byWidth);
    const byHeight = (paneHeight - GUTTER) / toPixels(first.height);
    return Math.max(0.1, Math.min(byWidth, byHeight));
  });

  /** A page is drawn at the window's pixel density, so it is sharp on a
   *  retina display and not needlessly large elsewhere. Rounded, so a pane
   *  resized by a pixel does not ask for every page again. */
  let scale = $derived(
    Math.round(zoom * (typeof devicePixelRatio === "number" ? devicePixelRatio : 1) * 4) / 4 ||
      0.25,
  );

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

  /** Change the zoom and keep the same part of the book in view: the
   *  scroll position is a fraction of the whole, and stays one. */
  async function setZoom(next: Zoom) {
    const before = scroller;
    const fraction = before ? before.scrollTop / Math.max(1, before.scrollHeight) : 0;
    mode = next;
    await tick();
    if (scroller) scroller.scrollTop = fraction * scroller.scrollHeight;
  }

  function step(by: number) {
    void setZoom(Math.min(4, Math.max(0.25, Math.round((zoom + by) * 4) / 4)));
  }

  /** Click-to-source: where on the page, in millimetres, the click was. */
  function clicked(event: MouseEvent, index: number) {
    const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const perMm = PX_PER_MM * zoom;
    void book.showSource(
      index,
      (event.clientX - box.left) / perMm,
      (event.clientY - box.top) / perMm,
    );
  }

  // Cursor-to-page: bring the cursor's page into view when it moves to
  // another one. Within a page the preview stays still — following every
  // line would make it jitter while someone types.
  let shownPage = -1;
  $effect(() => {
    const spot = book.cursorSpot;
    if (!spot || !scroller || spot.page === shownPage) return;
    shownPage = spot.page;
    scroller
      .querySelector(`[data-page="${spot.page}"]`)
      ?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });
</script>

<section>
  <header>
    <span class="count">
      {book.pageCount}
      {book.pageCount === 1 ? "page" : "pages"}
    </span>
    <span class="zoom">
      <button class:on={mode === "width"} onclick={() => setZoom("width")} title="Fit the page's width">Width</button>
      <button class:on={mode === "page"} onclick={() => setZoom("page")} title="Fit a whole page">Page</button>
      <button onclick={() => step(-0.25)} aria-label="Zoom out">−</button>
      <button class="level" onclick={() => setZoom(1)} title="Actual size">{Math.round(zoom * 100)}%</button>
      <button onclick={() => step(0.25)} aria-label="Zoom in">+</button>
    </span>
  </header>

  <div
    class="scroller"
    bind:this={scroller}
    bind:clientWidth={paneWidth}
    bind:clientHeight={paneHeight}
  >
    {#if book.layout && book.layout.pages.length > 0}
      {#each book.layout.pages as page (page.index)}
        <figure
          class="page"
          data-page={page.index}
          use:watch
          style="width: {toPixels(page.width) * zoom}px; height: {toPixels(page.height) * zoom}px"
        >
          {#if visible.has(page.index)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
            <img
              src={pageImageUrl(book.layout.revision, page.index, scale)}
              alt="Page {page.label ?? page.index + 1}"
              loading="lazy"
              title="Click to find this in the text"
              onclick={(event) => clicked(event, page.index)}
            />
          {/if}
          {#if book.cursorSpot?.page === page.index}
            <span
              class="cursor"
              aria-hidden="true"
              style="left: {(book.cursorSpot.x - 1.2) * PX_PER_MM * zoom}px; top: {(book.cursorSpot.y - 3.5) *
                PX_PER_MM *
                zoom}px; height: {4.5 * PX_PER_MM * zoom}px"
            ></span>
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

  .zoom button.on {
    background: var(--selected);
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
    cursor: text;
  }

  /* Where the editor's cursor is, on the page: a caret in the margin
     beside the line. */
  .cursor {
    position: absolute;
    width: 3px;
    border-radius: 2px;
    background: var(--accent, #3b6ea5);
    opacity: 0.75;
    pointer-events: none;
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

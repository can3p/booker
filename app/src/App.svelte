<script lang="ts">
  /**
   * The window: chapter sidebar, editor, preview, problems panel, status
   * bar — the layout `docs/waves/wave-1.md` asks track B for.
   *
   * It does no work of its own. Every action goes through the store, and
   * the store goes through `lib/ipc`, so the menu and the buttons run one
   * code path and the UI holds no opinion about what a book is.
   */
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";

  import { Book } from "./lib/book.svelte";
  import * as ipc from "./lib/ipc";
  import Editor from "./lib/panes/Editor.svelte";
  import Preview from "./lib/panes/Preview.svelte";
  import Problems from "./lib/panes/Problems.svelte";
  import Sidebar from "./lib/panes/Sidebar.svelte";
  import StatusBar from "./lib/panes/StatusBar.svelte";

  const book = new Book();

  let problemsOpen = $state(false);
  let updateNote = $state<string | null>(null);

  void book.restore();

  // The menu does no work itself: it says what was chosen and this runs it.
  void ipc.onMenu((id) => {
    if (id === "open") void chooseFolder();
    else if (id === "close") void book.close();
    else if (id === "export-pdf") void chooseDestination();
    else if (id === "check-for-updates") void checkForUpdates();
  });

  // An outside edit — another editor, an agent, a checkout — arrives as
  // one coalesced event; the store decides what it means for unsaved text.
  void ipc.onProjectChanged(() => void book.reloadFromDisk());

  async function chooseFolder() {
    const chosen = await openDialog({ directory: true, title: "Open a book" });
    if (typeof chosen === "string") await book.open(chosen);
  }

  async function chooseDestination() {
    if (!book.info) return;
    const suggested = `${book.info.config.title || "book"}.pdf`;
    const chosen = await saveDialog({
      title: "Export PDF",
      defaultPath: suggested,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (typeof chosen === "string") await book.exportPdf(chosen);
  }

  async function checkForUpdates() {
    updateNote = "Checking…";
    try {
      const update = await check();
      updateNote = update
        ? `Version ${update.version} is available.`
        : "Booker is up to date.";
    } catch (error: unknown) {
      // No release endpoint yet, no network, an unsigned build: none of
      // these are worth an error dialog.
      updateNote = `Could not check for updates: ${String(error)}`;
    }
  }
</script>

<div class="window">
  <header>
    <h1>Booker</h1>
    <div class="actions">
      <button onclick={chooseFolder} disabled={book.busy}>Open a book…</button>
      {#if book.info}
        <button onclick={chooseDestination} disabled={book.busy}>Export PDF…</button>
        <button onclick={() => book.close()}>Close</button>
      {/if}
    </div>
  </header>

  {#if book.failure}
    <p class="failure">
      {book.failure}
      <button class="dismiss" onclick={() => (book.failure = null)}>Dismiss</button>
    </p>
  {/if}

  {#if updateNote}
    <p class="note">
      {updateNote}
      <button class="dismiss" onclick={() => (updateNote = null)}>Dismiss</button>
    </p>
  {/if}

  {#if book.info}
    <div class="panes">
      <Sidebar {book} onopen={(root) => book.open(root)} />
      <Editor {book} />
      <Preview {book} />
    </div>
    <Problems {book} open={problemsOpen} ontoggle={() => (problemsOpen = !problemsOpen)} />
  {:else}
    <div class="panes empty-panes">
      <Sidebar {book} onopen={(root) => book.open(root)} />
      <p class="empty">
        No book open. Choose a folder that contains one — a
        <code>book.toml</code> and a <code>content/</code> folder.
      </p>
    </div>
  {/if}

  <StatusBar {book} />
</div>

<style>
  .window {
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    border-bottom: 1px solid var(--border);
    padding: 0.4rem 0.75rem;
  }

  h1 {
    font-size: 0.75rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 0.4rem;
  }

  .panes {
    display: grid;
    /* Text and pages side by side, equally: the preview is half of what a
       person is looking at, not a thumbnail beside it. */
    grid-template-columns: 15rem minmax(20rem, 1fr) minmax(18rem, 1fr);
    min-height: 0;
  }

  .empty-panes {
    grid-template-columns: 15rem 1fr;
  }

  .empty {
    color: var(--muted);
    padding: 2rem;
    max-width: 28rem;
  }

  .failure,
  .note {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin: 0;
    padding: 0.4rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .failure {
    border-left: 3px solid var(--error);
  }

  .dismiss {
    margin-left: auto;
    padding: 0.1rem 0.5rem;
    font-size: 0.85em;
  }
</style>

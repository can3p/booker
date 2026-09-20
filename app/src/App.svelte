<script lang="ts">
  /**
   * The window, as far as Wave 1's contracts step takes it: open a folder,
   * and show what the core says is in it.
   *
   * Track B replaces this with the real layout — chapter sidebar, editor
   * pane, preview, status bar, problems panel. What it must keep is the
   * shape below: the UI asks `lib/ipc`, renders what comes back, and holds
   * no opinion of its own about what a book is.
   */
  import { open } from "@tauri-apps/plugin-dialog";

  import { closeProject, openProject, projectInfo } from "./lib/ipc";
  import type { ProjectInfo } from "./lib/bindings/ProjectInfo";

  let info = $state<ProjectInfo | null>(null);
  let failure = $state<string | null>(null);
  let busy = $state(false);

  // A window that was open on a project should say so again after a reload.
  projectInfo()
    .then((current) => (info = current))
    .catch((error: unknown) => (failure = String(error)));

  async function chooseFolder() {
    const chosen = await open({ directory: true, title: "Open a book" });
    if (typeof chosen !== "string") return;

    busy = true;
    failure = null;
    try {
      info = await openProject(chosen);
    } catch (error: unknown) {
      // The folder could not be read at all. A book with *faults* in it
      // opens normally and reports them in `diagnostics` below.
      failure = String(error);
      info = null;
    } finally {
      busy = false;
    }
  }

  async function close() {
    await closeProject();
    info = null;
    failure = null;
  }
</script>

<main>
  <header>
    <h1>Booker</h1>
    <div class="actions">
      <button onclick={chooseFolder} disabled={busy}>Open a book…</button>
      {#if info}
        <button onclick={close}>Close</button>
      {/if}
    </div>
  </header>

  {#if failure}
    <p class="failure">{failure}</p>
  {/if}

  {#if info}
    <section>
      <h2>{info.config.title || "(untitled)"}</h2>
      <p class="meta">
        format {info.config.format} · language {info.config.language} · revision
        {info.revision}
      </p>

      <h3>Chapters: {info.chapters.length}</h3>
      <ul class="chapters">
        {#each info.chapters as chapter (chapter.path)}
          <li>
            <code>{chapter.path}</code>
            <span class="counts">
              {chapter.words} words, {chapter.headings} headings, {chapter.images}
              images
            </span>
            {#if chapter.title}<span class="title">“{chapter.title}”</span>{/if}
          </li>
        {/each}
      </ul>

      <h3>
        Problems: {info.diagnostics.length === 0 ? "none" : info.diagnostics.length}
      </h3>
      {#if info.diagnostics.length > 0}
        <ul class="problems">
          {#each info.diagnostics as problem (problem.rule + problem.message)}
            <li>
              <strong>{problem.severity}</strong>
              <code>{problem.rule}</code>
              {problem.message}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {:else if !failure}
    <p class="empty">No book open. Choose a folder that contains one.</p>
  {/if}
</main>

<style>
  main {
    padding: 1.5rem;
    max-width: 60rem;
    margin: 0 auto;
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
  }

  h1 {
    font-size: 1.1rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .meta {
    color: var(--muted);
  }

  .chapters,
  .problems {
    list-style: none;
    padding: 0;
    display: grid;
    gap: 0.4rem;
  }

  .counts,
  .title {
    color: var(--muted);
    margin-left: 0.5rem;
  }

  .failure {
    border-left: 3px solid var(--error);
    padding-left: 0.75rem;
  }

  .empty {
    color: var(--muted);
  }
</style>

<script lang="ts">
  /**
   * The problems panel.
   *
   * Every diagnostic names the file it came from and, where it applies, the
   * line (`AGENTS.md` §6) — so this panel shows both, and nothing here
   * invents a message of its own. The rule id is shown because it is what
   * `booker explain` will take as its argument from Wave 7.
   */
  import type { Book } from "../book.svelte";

  let { book, open, ontoggle }: { book: Book; open: boolean; ontoggle: () => void } =
    $props();

  let errors = $derived(book.problems.filter((p) => p.severity === "error").length);
  let warnings = $derived(book.problems.filter((p) => p.severity === "warning").length);

  function summary(): string {
    if (book.problems.length === 0) return "Problems: none";
    const parts = [];
    if (errors > 0) parts.push(`${errors} ${errors === 1 ? "error" : "errors"}`);
    if (warnings > 0) parts.push(`${warnings} ${warnings === 1 ? "warning" : "warnings"}`);
    const rest = book.problems.length - errors - warnings;
    if (rest > 0) parts.push(`${rest} more`);
    return `Problems: ${parts.join(", ")}`;
  }
</script>

<section class:open>
  <button class="summary" onclick={ontoggle} aria-expanded={open}>
    <span class:bad={errors > 0}>{summary()}</span>
    <span class="chevron">{open ? "▾" : "▸"}</span>
  </button>

  {#if open && book.problems.length > 0}
    <ul>
      {#each book.problems as problem, index (problem.rule + index)}
        <li>
          <span class="severity {problem.severity}">{problem.severity}</span>
          <span class="where">
            {#if problem.source}
              {problem.source.file}:{problem.source.line}
            {:else}
              —
            {/if}
          </span>
          <span class="message">{problem.message}</span>
          <code class="rule">{problem.rule}</code>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  section {
    border-top: 1px solid var(--border);
    max-height: 40vh;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .summary {
    display: flex;
    justify-content: space-between;
    width: 100%;
    border: 0;
    border-radius: 0;
    padding: 0.35rem 0.75rem;
  }

  .summary:hover {
    background: var(--hover);
    color: inherit;
  }

  .bad {
    color: var(--error);
  }

  .chevron {
    color: var(--muted);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 0.5rem;
    overflow-y: auto;
  }

  li {
    display: grid;
    grid-template-columns: 5rem 14rem 1fr auto;
    gap: 0.6rem;
    padding: 0.2rem 0.75rem;
    align-items: baseline;
  }

  li:hover {
    background: var(--hover);
  }

  .severity {
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 0.06em;
  }

  .severity.error {
    color: var(--error);
  }

  .severity.warning {
    color: var(--warning);
  }

  .severity.info {
    color: var(--muted);
  }

  .where,
  .rule {
    color: var(--muted);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 0.85em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

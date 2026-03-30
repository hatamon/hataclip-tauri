<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  /** @type {string[]} */
  let history = $state([]);

  /** @type {(() => void) | null} */
  let unsubscribe = null;

  async function loadHistory() {
    history = await invoke("get_clipboard_history");
  }

  onMount(() => {
    let disposed = false;

    void (async () => {
      await loadHistory();
      if (disposed) {
        return;
      }

      unsubscribe = await listen("clipboard-history-updated", (event) => {
        const payload = event.payload;
        if (payload && typeof payload === "object" && Array.isArray(payload.items)) {
          history = payload.items;
        }
      });
    })();

    return () => {
      disposed = true;
      if (unsubscribe) {
        unsubscribe();
      }
    };
  });
</script>

<main class="container">
  <header>
    <h1>Clipboard History</h1>
    <p class="hint">Press <kbd>Ctrl</kbd> + <kbd>7</kbd> to show this window.</p>
  </header>

  {#if history.length === 0}
    <p class="empty">Clipboard text history is empty.</p>
  {:else}
    <ul class="history-list">
      {#each history as item, index (item + index)}
        <li class="history-item">
          <span class="index">{index + 1}.</span>
          <pre>{item}</pre>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  .container {
    margin: 0 auto;
    max-width: 900px;
    padding: 1rem;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }

  h1 {
    margin: 0;
  }

  .hint {
    margin-top: 0.5rem;
    color: #666;
  }

  kbd {
    border: 1px solid #999;
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    background: #f3f3f3;
    font-family: inherit;
    font-size: 0.9rem;
  }

  .empty {
    margin-top: 1rem;
    color: #888;
  }

  .history-list {
    margin-top: 1rem;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 0.75rem;
  }

  .history-item {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 0.6rem 0.8rem;
    background: #fff;
  }

  .index {
    color: #666;
    min-width: 2ch;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Courier New", monospace;
  }

  @media (prefers-color-scheme: dark) {
    .container {
      color: #f5f5f5;
      background: #2b2b2b;
    }

    .hint,
    .empty,
    .index {
      color: #b8b8b8;
    }

    kbd {
      border-color: #777;
      background: #454545;
    }

    .history-item {
      border-color: #4f4f4f;
      background: #343434;
    }
  }
</style>

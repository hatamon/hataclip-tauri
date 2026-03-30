<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  /** @type {string[]} */
  let history = $state([]);
  let selectedIndex = $state(0);

  /** @type {(() => void) | null} */
  let unsubscribe = null;

  async function loadHistory() {
    history = await invoke("get_clipboard_history");
    if (selectedIndex >= history.length) {
      selectedIndex = Math.max(0, history.length - 1);
    }
    await invoke("set_selected_history_index", { index: selectedIndex });
  }

  async function bringToFront() {
    await invoke("bring_history_window_to_front");
  }

  /**
   * @param {number} index
   */
  async function selectIndex(index) {
    selectedIndex = index;
    await invoke("set_selected_history_index", { index });
  }

  async function pasteSelected() {
    await invoke("paste_selected_history_item");
  }

  /**
   * @param {KeyboardEvent} event
   */
  async function onKeydown(event) {
    if (history.length === 0) {
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      const next = Math.min(history.length - 1, selectedIndex + 1);
      await selectIndex(next);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      const next = Math.max(0, selectedIndex - 1);
      await selectIndex(next);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      await pasteSelected();
    }
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
          if (selectedIndex >= history.length) {
            selectedIndex = Math.max(0, history.length - 1);
          }
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

<svelte:window onkeydown={onKeydown} onclick={bringToFront} />

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
        <li class="history-item" class:selected={index === selectedIndex}>
          <button
            type="button"
            class="history-button"
            onclick={() => selectIndex(index)}
            ondblclick={pasteSelected}
          >
            <span class="index">{index + 1}.</span>
            <pre>{item}</pre>
          </button>
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
    border: 1px solid #ddd;
    border-radius: 8px;
    background: #fff;
  }

  .history-item.selected {
    border-color: #396cd8;
    box-shadow: 0 0 0 1px #396cd8;
  }

  .history-button {
    width: 100%;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.5rem;
    text-align: left;
    border: 0;
    background: transparent;
    padding: 0.6rem 0.8rem;
    cursor: pointer;
    color: inherit;
    font: inherit;
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

    .history-item.selected {
      border-color: #6ea8ff;
      box-shadow: 0 0 0 1px #6ea8ff;
    }
  }
</style>

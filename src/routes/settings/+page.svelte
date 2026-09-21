<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { fromEvent, toLabel, type Shortcuts } from "$lib/shortcut";

  type Slot = "register" | "show" | "expand";

  let register = $state("");
  let show = $state("");
  let expand = $state("");
  let quickPaste = $state(true);
  let recording = $state<Slot | null>(null);
  let message = $state("");
  let error = $state("");

  onMount(async () => {
    const shortcuts = await invoke<Shortcuts>("get_shortcuts");
    register = shortcuts.register;
    show = shortcuts.show;
    expand = shortcuts.expand;
    quickPaste = shortcuts.quickPaste;
  });

  $effect(() => {
    if (recording === null) {
      return;
    }
    const slot = recording;
    const onKey = (event: KeyboardEvent) => {
      event.preventDefault();
      if (event.key === "Escape" || (event.ctrlKey && event.key === "[")) {
        recording = null;
        return;
      }
      const value = fromEvent(event);
      if (value === null) {
        return;
      }
      if (slot === "register") {
        register = value;
      } else if (slot === "show") {
        show = value;
      } else {
        expand = value;
      }
      recording = null;
      message = "";
      error = "";
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  async function save() {
    message = "";
    error = "";
    try {
      await invoke("set_shortcuts", { register, show, quickPaste, expand });
      message = "保存した";
    } catch (reason) {
      error = String(reason);
    }
  }

  function label(slot: Slot, value: string): string {
    if (recording === slot) {
      return "キーを押す";
    }
    return value.length > 0 ? toLabel(value) : "未設定";
  }
</script>

<div class="settings">
  <h1>ショートカット</h1>

  <div class="row">
    <span class="name">登録</span>
    <button type="button" class:recording={recording === "register"} onclick={() => (recording = "register")}>
      {label("register", register)}
    </button>
  </div>

  <div class="row">
    <span class="name">表示</span>
    <button type="button" class:recording={recording === "show"} onclick={() => (recording = "show")}>
      {label("show", show)}
    </button>
  </div>

  <div class="row">
    <span class="name">タグ</span>
    <button type="button" class:recording={recording === "expand"} onclick={() => (recording = "expand")}>
      {label("expand", expand)}
    </button>
  </div>

  <label class="row">
    <span class="name">速貼</span>
    <input type="checkbox" bind:checked={quickPaste} />
    <span class="hint">Ctrl+Shift+1〜9 で一覧を出さずに貼る</span>
  </label>

  <p class="hint">押したい組み合わせを押す。修飾キーが要る。`Esc` で取り消し。タグは前面で `#foo` / `{{date}}` / `:sh dir` を選んで押すと置き換える（初期値 Ctrl+Shift+H）。</p>

  <div class="actions">
    <button type="button" class="save" onclick={save}>保存</button>
    {#if message.length > 0}
      <span class="message">{message}</span>
    {/if}
  </div>

  {#if error.length > 0}
    <p class="error">{error}</p>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: #1c1c1c;
    color: #eee;
    font: 13px/1.35 ui-sans-serif, system-ui, sans-serif;
  }

  .settings {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h1 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .name {
    width: 40px;
    color: #9aa;
  }

  .row button {
    flex: 1;
    border: 1px solid #3a3a3a;
    background: #111;
    color: #eee;
    padding: 6px 10px;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .row button.recording {
    border-color: #2c4a6e;
    background: #2c4a6e;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .save {
    border: 0;
    background: #333;
    color: #ddd;
    border-radius: 10px;
    padding: 4px 14px;
    font: inherit;
    cursor: pointer;
  }

  .hint,
  .message {
    color: #9aa;
    font-size: 11px;
    margin: 0;
  }

  .error {
    color: #e88;
    font-size: 11px;
    margin: 0;
  }
</style>

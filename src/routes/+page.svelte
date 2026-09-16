<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { fuzzyFilter } from "$lib/fuzzy";
  import { parseQuery } from "$lib/query";
  import {
    applyTagCompletion,
    currentTagPrefix,
    matchingTags,
    uniqueTags,
  } from "$lib/tags";
  import type { Item } from "$lib/types";

  type Mode = "normal" | "search" | "editing";

  let items = $state<Item[]>([]);
  let selected = $state(0);
  let mode = $state<Mode>("normal");
  let query = $state("");
  let pending = $state("");
  let searchEl = $state<HTMLInputElement | undefined>(undefined);
  let listEl = $state<HTMLUListElement | undefined>(undefined);
  let editText = $state("");
  let editTags = $state<string[]>([]);
  let tagDraft = $state("");
  let editingId = $state<string | null>(null);

  const filtered = $derived.by(() => {
    const parsed = parseQuery(query);
    let list = items.filter((item) => parsed.tags.every((tag) => item.tags.includes(tag)));
    if (parsed.text.length > 0) {
      list = fuzzyFilter(parsed.text, list);
    }
    return list;
  });

  const allTags = $derived(uniqueTags(items));
  const searchTagPrefix = $derived(mode === "search" ? currentTagPrefix(query) : null);
  const searchSuggestions = $derived(
    searchTagPrefix === null ? [] : matchingTags(searchTagPrefix, allTags),
  );
  const editSuggestions = $derived(
    tagDraft.length === 0 ? [] : matchingTags(tagDraft, allTags.filter((tag) => !editTags.includes(tag))),
  );

  $effect(() => {
    if (selected >= filtered.length) {
      selected = Math.max(0, filtered.length - 1);
    }
  });

  $effect(() => {
    if (mode === "search") {
      queueMicrotask(() => searchEl?.focus());
    }
  });

  $effect(() => {
    if (filtered.length === 0) {
      return;
    }
    listEl?.children[selected]?.scrollIntoView({ block: "nearest" });
  });

  function currentItem(): Item | undefined {
    return filtered[selected];
  }

  async function pasteSelected() {
    const item = currentItem();
    if (!item) {
      return;
    }
    await invoke("paste_item", { id: item.id });
  }

  async function hidePicker() {
    await invoke("hide_picker");
  }

  async function deleteSelected() {
    const item = currentItem();
    if (!item) {
      return;
    }
    await invoke("delete_item", { id: item.id });
    items = items.filter((entry) => entry.id !== item.id);
  }

  function startEdit() {
    const item = currentItem();
    if (!item) {
      return;
    }
    editingId = item.id;
    editText = item.text;
    editTags = [...item.tags];
    tagDraft = "";
    mode = "editing";
  }

  function cancelEdit() {
    editingId = null;
    mode = "normal";
    pending = "";
  }

  async function saveEdit() {
    if (editingId === null) {
      return;
    }
    const tags = tagDraft.trim().length > 0 ? [...editTags, tagDraft.trim()] : editTags;
    await invoke("update_item", { id: editingId, text: editText, tags });
    items = items.map((item) =>
      item.id === editingId ? { ...item, text: editText, tags } : item,
    );
    cancelEdit();
  }

  function addEditTag(tag: string) {
    const value = tag.trim();
    if (value.length === 0 || editTags.includes(value)) {
      tagDraft = "";
      return;
    }
    editTags = [...editTags, value];
    tagDraft = "";
  }

  function move(delta: number) {
    if (filtered.length === 0) {
      return;
    }
    selected = (selected + delta + filtered.length) % filtered.length;
  }

  function openPicker(next: Item[]) {
    items = next;
    selected = 0;
    mode = "normal";
    query = "";
    pending = "";
    editingId = null;
  }

  function onSearchKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      mode = "normal";
      pending = "";
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      void pasteSelected();
      return;
    }
    if (event.key === "Tab" && searchSuggestions.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      query = applyTagCompletion(query, searchSuggestions[0]);
    }
  }

  function onDocumentKeydown(event: KeyboardEvent) {
    if (event.isComposing || event.defaultPrevented) {
      return;
    }
    if (mode === "search") {
      return;
    }
    if (mode === "editing") {
      if (event.key === "Escape") {
        event.preventDefault();
        cancelEdit();
      }
      if (event.key === "Enter" && event.ctrlKey) {
        event.preventDefault();
        void saveEdit();
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      void hidePicker();
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      void pasteSelected();
      return;
    }
    if (event.key === "/") {
      event.preventDefault();
      mode = "search";
      pending = "";
      return;
    }
    if (event.key === "j") {
      event.preventDefault();
      pending = "";
      move(1);
      return;
    }
    if (event.key === "k") {
      event.preventDefault();
      pending = "";
      move(-1);
      return;
    }
    if (event.key === "e") {
      event.preventDefault();
      pending = "";
      startEdit();
      return;
    }
    if (event.key === "G") {
      event.preventDefault();
      pending = "";
      if (filtered.length > 0) {
        selected = filtered.length - 1;
      }
      return;
    }
    if (event.key === "g") {
      event.preventDefault();
      if (pending === "g") {
        selected = 0;
        pending = "";
      } else {
        pending = "g";
      }
      return;
    }
    if (event.key === "d") {
      event.preventDefault();
      if (pending === "d") {
        pending = "";
        void deleteSelected();
      } else {
        pending = "d";
      }
      return;
    }
    pending = "";
  }

  onMount(() => {
    const onKey = (event: KeyboardEvent) => onDocumentKeydown(event);
    window.addEventListener("keydown", onKey);

    const unlistenOpened = listen<Item[]>("picker-opened", (event) => {
      openPicker(event.payload);
    });
    const unlistenChanged = listen<Item[]>("items-changed", (event) => {
      items = event.payload;
    });

    return () => {
      window.removeEventListener("keydown", onKey);
      void unlistenOpened.then((stop) => stop());
      void unlistenChanged.then((stop) => stop());
    };
  });
</script>

<div class="picker">
  {#if mode === "editing"}
    <div class="edit">
      <textarea bind:value={editText} rows="6"></textarea>
      <div class="tags">
        {#each editTags as tag (tag)}
          <button
            type="button"
            class="chip"
            onclick={() => {
              editTags = editTags.filter((entry) => entry !== tag);
            }}>{tag} ×</button
          >
        {/each}
        <input
          bind:value={tagDraft}
          placeholder="tag"
          onkeydown={(event) => {
            if (event.isComposing) {
              return;
            }
            if (event.key === "Enter" && !event.ctrlKey) {
              event.preventDefault();
              if (editSuggestions.length > 0 && tagDraft !== editSuggestions[0]) {
                addEditTag(editSuggestions[0]);
              } else {
                addEditTag(tagDraft);
              }
            }
            if (event.key === "Tab" && editSuggestions.length > 0) {
              event.preventDefault();
              addEditTag(editSuggestions[0]);
            }
          }}
        />
      </div>
      {#if editSuggestions.length > 0}
        <ul class="suggest">
          {#each editSuggestions as tag (tag)}
            <li>
              <button type="button" onclick={() => addEditTag(tag)}>{tag}</button>
            </li>
          {/each}
        </ul>
      {/if}
      <p class="hint">Ctrl+Enter save · Esc cancel</p>
    </div>
  {:else}
    {#if mode === "search"}
      <input
        bind:this={searchEl}
        bind:value={query}
        class="search"
        placeholder="search  #tag"
        onkeydown={onSearchKeydown}
      />
      {#if searchSuggestions.length > 0}
        <ul class="suggest">
          {#each searchSuggestions as tag (tag)}
            <li>
              <button
                type="button"
                onclick={() => {
                  query = applyTagCompletion(query, tag);
                  searchEl?.focus();
                }}>{tag}</button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
    <ul class="list" bind:this={listEl}>
      {#each filtered as item, index (item.id)}
        <li class:active={index === selected}>
          <button
            type="button"
            class="row"
            onclick={() => {
              selected = index;
            }}
            ondblclick={() => {
              selected = index;
              void pasteSelected();
            }}
          >
            <span class="text">{item.text}</span>
            {#if item.tags.length > 0}
              <span class="item-tags">{item.tags.map((tag) => `#${tag}`).join(" ")}</span>
            {/if}
          </button>
        </li>
      {:else}
        <li class="empty">empty</li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: #1c1c1c;
    color: #eee;
    font: 13px/1.35 ui-sans-serif, system-ui, sans-serif;
  }

  .picker {
    height: 100vh;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    border: 1px solid #3a3a3a;
  }

  .search,
  textarea,
  .tags input {
    box-sizing: border-box;
    width: 100%;
    border: 0;
    border-bottom: 1px solid #333;
    background: #111;
    color: #eee;
    padding: 8px 10px;
    font: inherit;
    outline: none;
  }

  textarea {
    resize: none;
    min-height: 120px;
    border-bottom: 1px solid #333;
  }

  .list {
    margin: 0;
    padding: 0;
    list-style: none;
    overflow: auto;
    flex: 1;
  }

  .row {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    width: 100%;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    padding: 8px 10px;
    font: inherit;
    cursor: pointer;
  }

  li.active .row,
  .row:hover {
    background: #2c4a6e;
  }

  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .item-tags,
  .hint,
  .empty {
    color: #9aa;
    font-size: 11px;
  }

  .empty,
  .hint {
    padding: 8px 10px;
  }

  .edit {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px 8px;
    align-items: center;
  }

  .chip,
  .suggest button {
    border: 0;
    background: #333;
    color: #ddd;
    border-radius: 10px;
    padding: 2px 8px;
    font: inherit;
    cursor: pointer;
  }

  .tags input {
    width: auto;
    flex: 1;
    min-width: 80px;
    border: 0;
    padding: 4px;
    background: transparent;
  }

  .suggest {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    list-style: none;
    margin: 0;
    padding: 4px 8px;
  }
</style>

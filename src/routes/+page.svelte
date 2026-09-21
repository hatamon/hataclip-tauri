<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { uniqueAskNames } from "$lib/ask";
  import {
    applyColonCompletion,
    matchingColonCommands,
  } from "$lib/colon";
  import {
    combinePending,
    eventToToken,
    formatMaps,
    hasLeaderMaps,
    isLeaderToken,
    matchMap,
    parseMapArgs,
    parseMapleaderArgs,
    parseUnmapArgs,
    removeMap,
    tokenizeKeys,
    tokenToEventKey,
    upsertMap,
    whichKeysForMaps,
    DEFAULT_LEADER,
    type KeyMap,
  } from "$lib/map";
  import { uniquePickSpecs, type PickSpec } from "$lib/pick";
  import { fuzzyFilter } from "$lib/fuzzy";
  import { parseQuery } from "$lib/query";
  import {
    applyTagCompletion,
    currentTagPrefix,
    isSecret,
    matchesAlias,
    matchingTags,
    tagsByCount,
    uniqueTags,
  } from "$lib/tags";
  import type { Item } from "$lib/types";

  type Mode = "normal" | "search" | "editing" | "tag" | "colon" | "help" | "ask" | "pick";

  // `.` で繰り返せる変更。移動やヤンクは覚えない。
  type Change =
    | { kind: "delete" }
    | { kind: "put"; above: boolean }
    | { kind: "tag"; tag: string; add: boolean }
    | { kind: "pin"; pinned: boolean }
    | { kind: "split" }
    | { kind: "move"; delta: number }
    | { kind: "merge" }
    | { kind: "clone" }
    | { kind: "sub"; old: string; new: string }
    | { kind: "sort" };

  let items = $state<Item[]>([]);
  let selected = $state(0);
  let mode = $state<Mode>("normal");
  let query = $state("");
  let pending = $state("");
  let searchEl = $state<HTMLInputElement | undefined>(undefined);
  let tagEl = $state<HTMLInputElement | undefined>(undefined);
  let listEl = $state<HTMLUListElement | undefined>(undefined);
  let editText = $state("");
  let editTags = $state<string[]>([]);
  let tagDraft = $state("");
  let editingId = $state<string | null>(null);
  let yanked = $state<{ text: string; tags: string[] } | null>(null);
  let anchor = $state<number | null>(null);
  let tagInput = $state("");
  let tagAdd = $state(true);
  let lastChange = $state<Change | null>(null);
  let preview = $state(false);
  let tagCycle = $state(-1);
  let draftNewId = $state<string | null>(null);
  let colonEl = $state<HTMLInputElement | undefined>(undefined);
  let colonInput = $state("");
  let colonHistory = $state<string[]>([]);
  let colonHistIndex = $state(-1);
  let colonDraft = $state("");
  let helpText = $state("");
  let helpEl = $state<HTMLPreElement | undefined>(undefined);
  let helpTopics = $state<string[]>([]);
  let contextKey = $state<string | null>(null);
  let contextOnly = $state(false);
  let findChar = $state("");
  let askEl = $state<HTMLInputElement | undefined>(undefined);
  let askQueue = $state<string[]>([]);
  let askIndex = $state(0);
  let askDraft = $state("");
  let askAnswers = $state<Record<string, string>>({});
  let pendingPaste = $state<{
    ids: string[];
    keepOpen: boolean;
    format: boolean;
    raw: boolean;
    separator: string;
    prefix?: string;
    typed?: boolean;
  } | null>(null);
  let lastPaste = $state<{
    ids: string[];
    keepOpen: boolean;
    format: boolean;
    raw: boolean;
    separator: string;
    prefix?: string;
    typed?: boolean;
    answers: Record<string, string>;
  } | null>(null);
  let pickQueue = $state<PickSpec[]>([]);
  let pickIndex = $state(0);
  let pickChoice = $state(0);
  let info = $state(false);
  let whichPrefix = $state("");
  let whichTimer: ReturnType<typeof setTimeout> | null = null;
  let maps = $state<KeyMap[]>([]);
  let mapLeader = $state(DEFAULT_LEADER);
  let mapsEnabled = true;

  const filtered = $derived.by(() => {
    const parsed = parseQuery(query);
    let list = items.filter((item) => parsed.tags.every((tag) => item.tags.includes(tag)));
    if (contextOnly && contextKey) {
      list = list.filter((item) => item.contexts.includes(contextKey!));
    }
    if (parsed.text.length > 0) {
      const fuzzy = fuzzyFilter(parsed.text, list);
      const seen = new Set(fuzzy.map((item) => item.id));
      for (const item of list) {
        if (!seen.has(item.id) && matchesAlias(item, parsed.text)) {
          fuzzy.push(item);
        }
      }
      list = fuzzy;
    }
    return list;
  });

  const range = $derived.by<[number, number]>(() => {
    if (anchor === null) {
      return [selected, selected];
    }
    return anchor <= selected ? [anchor, selected] : [selected, anchor];
  });

  const selectedItems = $derived(filtered.slice(range[0], range[1] + 1));
  const selectedIds = $derived(selectedItems.map((item) => item.id));

  const allTags = $derived(uniqueTags(items));
  const rankedTags = $derived(tagsByCount(items));
  const searchTagPrefix = $derived(mode === "search" ? currentTagPrefix(query) : null);
  const searchSuggestions = $derived(
    searchTagPrefix === null ? [] : matchingTags(searchTagPrefix, allTags),
  );
  const tagSuggestions = $derived(
    mode === "tag" && tagInput.length > 0 ? matchingTags(tagInput, allTags) : [],
  );
  const editSuggestions = $derived(
    tagDraft.length === 0 ? [] : matchingTags(tagDraft, allTags.filter((tag) => !editTags.includes(tag))),
  );
  const colonSuggestions = $derived(
    mode === "colon" ? matchingColonCommands(colonInput) : [],
  );

  const whichKeys = $derived.by(() => {
    if (whichPrefix.length === 0) {
      return [];
    }
    const mapped = whichKeysForMaps(maps, whichPrefix);
    if (whichPrefix === "g") {
      return [
        { key: "g", label: "先頭" },
        { key: "f", label: "開く" },
        { key: "e", label: "全文" },
        { key: "Enter", label: "本文のまま" },
        { key: "J", label: "カンマ" },
        { key: "T", label: "タブ" },
        { key: ">", label: "引用" },
        { key: "*", label: "箇条書き" },
        { key: "p", label: "直前の貼り付け" },
        { key: "?", label: "行の情報" },
        ...mapped,
      ];
    }
    if (whichPrefix === "d") {
      return [{ key: "d", label: "削除" }, ...mapped];
    }
    if (whichPrefix === "y") {
      return [{ key: "y", label: "ヤンク" }, ...mapped];
    }
    if (whichPrefix === "f") {
      return [{ key: "文字", label: "その文字へ" }, ...mapped];
    }
    return mapped;
  });

  $effect(() => {
    if (selected >= filtered.length) {
      selected = Math.max(0, filtered.length - 1);
    }
  });

  $effect(() => {
    if (mode === "search") {
      queueMicrotask(() => searchEl?.focus());
    }
    if (mode === "tag") {
      queueMicrotask(() => tagEl?.focus());
    }
    if (mode === "colon") {
      queueMicrotask(() => colonEl?.focus());
    }
    if (mode === "ask") {
      queueMicrotask(() => askEl?.focus());
    }
  });

  $effect(() => {
    const prefix = pending;
    if (whichTimer) {
      clearTimeout(whichTimer);
      whichTimer = null;
    }
    whichPrefix = "";
    const mapWait =
      prefix.length > 0 &&
      maps.some((entry) => entry.lhs.startsWith(prefix) && entry.lhs.length > prefix.length);
    if (
      prefix === "g" ||
      prefix === "d" ||
      prefix === "y" ||
      prefix === "f" ||
      prefix === "<leader>" ||
      mapWait
    ) {
      whichTimer = setTimeout(() => {
        whichPrefix = prefix;
      }, 400);
    }
    return () => {
      if (whichTimer) {
        clearTimeout(whichTimer);
        whichTimer = null;
      }
    };
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

  function clearSelection() {
    anchor = null;
  }

  function selectById(id: string) {
    queueMicrotask(() => {
      const next = filtered.findIndex((item) => item.id === id);
      if (next >= 0) {
        selected = next;
      }
    });
  }

  async function pasteSelection(
    keepOpen: boolean,
    format = false,
    raw = false,
    separator = "\n",
    prefix?: string,
    typed = false,
  ) {
    if (selectedIds.length === 0) {
      return;
    }
    const ids = selectedIds;
    const rows = selectedItems;
    if (!raw) {
      const names = uniqueAskNames(rows);
      if (names.length > 0) {
        clearSelection();
        askQueue = names;
        askIndex = 0;
        askDraft = "";
        askAnswers = {};
        pendingPaste = { ids, keepOpen, format, raw, separator, prefix, typed };
        mode = "ask";
        return;
      }
      const picks = uniquePickSpecs(rows);
      if (picks.length > 0) {
        clearSelection();
        pendingPaste = { ids, keepOpen, format, raw, separator, prefix, typed };
        startPicks(picks);
        return;
      }
    }
    clearSelection();
    await invokePaste({ ids, keepOpen, format, raw, separator, prefix, typed, answers: {} });
  }

  async function pasteRow(index: number, keepOpen: boolean) {
    const item = filtered[index];
    if (!item) {
      return;
    }
    clearSelection();
    selected = index;
    const names = uniqueAskNames([item]);
    if (names.length > 0) {
      askQueue = names;
      askIndex = 0;
      askDraft = "";
      askAnswers = {};
      pendingPaste = { ids: [item.id], keepOpen, format: false, raw: false, separator: "\n" };
      mode = "ask";
      return;
    }
    const picks = uniquePickSpecs([item]);
    if (picks.length > 0) {
      pendingPaste = { ids: [item.id], keepOpen, format: false, raw: false, separator: "\n" };
      startPicks(picks);
      return;
    }
    await invokePaste({
      ids: [item.id],
      keepOpen,
      format: false,
      raw: false,
      separator: "\n",
      answers: {},
    });
  }

  async function invokePaste(opts: {
    ids: string[];
    keepOpen: boolean;
    format: boolean;
    raw: boolean;
    separator: string;
    prefix?: string;
    typed?: boolean;
    answers: Record<string, string>;
  }) {
    const ok = await invoke<boolean>("paste_items", {
      ids: opts.ids,
      keepOpen: opts.keepOpen,
      format: opts.format,
      raw: opts.raw,
      separator: opts.separator,
      answers: opts.answers,
      prefix: opts.prefix ?? null,
      typed: opts.typed ?? false,
    });
    if (ok) {
      lastPaste = { ...opts };
    }
  }

  function startPicks(picks: PickSpec[]) {
    pickQueue = picks;
    pickIndex = 0;
    pickChoice = 0;
    mode = "pick";
  }

  async function continueAfterPrompts() {
    const pending = pendingPaste;
    if (pending === null) {
      return;
    }
    const rows = pending.ids
      .map((id) => items.find((item) => item.id === id))
      .filter((item): item is Item => item !== undefined);
    const remaining = uniquePickSpecs(rows).filter(
      (entry) => askAnswers[`pick:${entry.spec}`] === undefined,
    );
    if (remaining.length > 0 && pickQueue.length === 0) {
      startPicks(remaining);
      return;
    }
    pendingPaste = null;
    pickQueue = [];
    mode = "normal";
    await invokePaste({ ...pending, answers: askAnswers });
    askAnswers = {};
  }

  async function copySelection() {
    if (selectedIds.length === 0) {
      return;
    }
    await invoke("copy_items", { ids: selectedIds });
  }

  async function hidePicker() {
    await invoke("hide_picker");
  }

  async function deleteSelection() {
    if (selectedIds.length === 0) {
      return;
    }
    yanked = {
      text: selectedItems.map((item) => item.text).join("\n"),
      tags: selectedItems.length === 1 ? [...selectedItems[0].tags] : [],
    };
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("delete_items", { ids });
    lastChange = { kind: "delete" };
  }

  async function undoChange() {
    items = await invoke<Item[]>("undo_change");
  }

  async function redoChange() {
    items = await invoke<Item[]>("redo_change");
  }

  function yankSelection() {
    if (selectedItems.length === 0) {
      return;
    }
    yanked = {
      text: selectedItems.map((item) => item.text).join("\n"),
      tags: selectedItems.length === 1 ? [...selectedItems[0].tags] : [],
    };
    clearSelection();
  }

  async function putYanked(above: boolean) {
    if (yanked === null) {
      return;
    }
    const result = await invoke<{ item: Item; items: Item[] }>("put_item", {
      text: yanked.text,
      tags: yanked.tags,
      anchorId: currentItem()?.id ?? null,
      above,
    });
    items = result.items;
    selectById(result.item.id);
    lastChange = { kind: "put", above };
  }

  async function applyTag(tag: string, add: boolean) {
    const value = tag.trim();
    if (value.length === 0 || selectedIds.length === 0) {
      return;
    }
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("set_tag", { ids, tag: value, add });
    lastChange = { kind: "tag", tag: value, add };
  }

  async function applyPin(pinned: boolean) {
    if (selectedIds.length === 0) {
      return;
    }
    const ids = selectedIds;
    const id = selectedItems[0].id;
    clearSelection();
    items = await invoke<Item[]>("set_pinned", { ids, pinned });
    selectById(id);
    lastChange = { kind: "pin", pinned };
  }

  async function repeatChange() {
    if (lastChange === null) {
      return;
    }
    switch (lastChange.kind) {
      case "delete":
        await deleteSelection();
        return;
      case "put":
        await putYanked(lastChange.above);
        return;
      case "tag":
        await applyTag(lastChange.tag, lastChange.add);
        return;
      case "pin":
        await applyPin(lastChange.pinned);
        return;
      case "split":
        await splitSelection();
        return;
      case "move":
        await movePins(lastChange.delta);
        return;
      case "merge":
        await mergeSelection();
        return;
      case "clone":
        await cloneSelection();
        return;
      case "sub":
        await substituteSelection(lastChange.old, lastChange.new);
        return;
      case "sort":
        await sortSelection();
    }
  }

  async function splitSelection() {
    if (selectedIds.length === 0) {
      return;
    }
    const id = selectedItems[0].id;
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("split_items", { ids });
    selectById(id);
    lastChange = { kind: "split" };
  }

  async function movePins(delta: number) {
    if (selectedIds.length === 0 || !selectedItems.every((item) => item.pinned)) {
      return;
    }
    const id = selectedItems[0].id;
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("move_pins", { ids, delta });
    selectById(id);
    lastChange = { kind: "move", delta };
  }

  async function mergeSelection() {
    if (anchor === null || selectedIds.length < 2) {
      return;
    }
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("merge_items", { ids });
    lastChange = { kind: "merge" };
  }

  async function cloneSelection() {
    if (selectedIds.length === 0) {
      return;
    }
    const id = selectedItems[0].id;
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("clone_items", { ids });
    selectById(id);
    lastChange = { kind: "clone" };
  }

  async function substituteSelection(old: string, next: string) {
    if (old.length === 0 || selectedIds.length === 0) {
      return;
    }
    const id = selectedItems[0].id;
    const ids = selectedIds;
    clearSelection();
    items = await invoke<Item[]>("substitute_items", { ids, old, new: next });
    selectById(id);
    lastChange = { kind: "sub", old, new: next };
  }

  async function sortSelection() {
    if (anchor === null || selectedIds.length < 2) {
      return;
    }
    const ids = selectedIds;
    const id = ids[0];
    clearSelection();
    items = await invoke<Item[]>("sort_items", { ids });
    selectById(id);
    lastChange = { kind: "sort" };
  }

  function pageMove(direction: number) {
    const height = listEl?.clientHeight ?? 140;
    const step = Math.max(1, Math.floor(height / 48));
    move(direction * step);
  }

  async function replayLastPaste() {
    if (lastPaste === null) {
      return;
    }
    if (!lastPaste.ids.every((id) => items.some((item) => item.id === id))) {
      lastPaste = null;
      return;
    }
    await invokePaste(lastPaste);
  }

  function startEdit() {
    const item = currentItem();
    if (!item) {
      return;
    }
    draftNewId = null;
    clearSelection();
    editingId = item.id;
    editText = item.text;
    editTags = [...item.tags];
    tagDraft = "";
    mode = "editing";
  }

  async function editExternal() {
    const item = currentItem();
    if (!item) {
      return;
    }
    clearSelection();
    try {
      await invoke("edit_external", { id: item.id });
    } catch {
      // エディタが無いときは何もしない。
    }
  }

  function cancelEdit() {
    const created = draftNewId;
    editingId = null;
    draftNewId = null;
    mode = "normal";
    pending = "";
    if (created) {
      void invoke<Item[]>("delete_items", { ids: [created] }).then((next) => {
        items = next;
      });
    }
  }

  async function saveEdit() {
    if (editingId === null) {
      return;
    }
    const tags = tagDraft.trim().length > 0 ? [...editTags, tagDraft.trim()] : editTags;
    items = await invoke<Item[]>("update_item", { id: editingId, text: editText, tags });
    draftNewId = null;
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

  function startTagInput(add: boolean) {
    if (filtered.length === 0) {
      return;
    }
    tagAdd = add;
    tagInput = "";
    mode = "tag";
  }

  function move(delta: number) {
    if (filtered.length === 0) {
      return;
    }
    selected = Math.min(filtered.length - 1, Math.max(0, selected + delta));
  }

  function openPicker(next: Item[]) {
    items = next;
    selected = 0;
    mode = "normal";
    query = "";
    pending = "";
    editingId = null;
    draftNewId = null;
    anchor = null;
    preview = false;
    tagCycle = -1;
    colonInput = "";
    helpText = "";
    contextOnly = false;
    findChar = "";
    askQueue = [];
    pendingPaste = null;
    pickQueue = [];
    info = false;
    whichPrefix = "";
    colonHistIndex = -1;
    void invoke<string | null>("get_context").then((key) => {
      contextKey = key;
    });
  }

  async function openBlank() {
    const result = await invoke<{ item: Item; items: Item[] }>("put_item", {
      text: "",
      tags: [],
      anchorId: null,
      above: true,
    });
    items = result.items;
    draftNewId = result.item.id;
    editingId = result.item.id;
    editText = "";
    editTags = [];
    tagDraft = "";
    mode = "editing";
    clearSelection();
    selectById(result.item.id);
  }

  function cycleTag(direction: number) {
    if (rankedTags.length === 0) {
      return;
    }
    const next = tagCycle + direction;
    if (next < -1) {
      tagCycle = rankedTags.length - 1;
    } else if (next >= rankedTags.length) {
      tagCycle = -1;
    } else {
      tagCycle = next;
    }
    query = tagCycle < 0 ? "" : `#${rankedTags[tagCycle]} `;
    selected = 0;
  }

  async function finishAsk(commit: boolean) {
    askQueue = [];
    askDraft = "";
    if (!commit) {
      pendingPaste = null;
      pickQueue = [];
      mode = "normal";
      askAnswers = {};
      return;
    }
    mode = "normal";
    await continueAfterPrompts();
  }

  function onAskKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (isEscape(event)) {
      event.preventDefault();
      event.stopPropagation();
      void finishAsk(false);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      const name = askQueue[askIndex];
      askAnswers = { ...askAnswers, [name]: askDraft };
      askDraft = "";
      if (askIndex + 1 >= askQueue.length) {
        void finishAsk(true);
      } else {
        askIndex += 1;
      }
    }
  }

  function onPickKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    if (isEscape(event)) {
      pendingPaste = null;
      pickQueue = [];
      askAnswers = {};
      mode = "normal";
      return;
    }
    const options = pickQueue[pickIndex]?.options ?? [];
    if (event.key === "j" || event.key === "ArrowDown") {
      if (options.length > 0) {
        pickChoice = Math.min(options.length - 1, pickChoice + 1);
      }
      return;
    }
    if (event.key === "k" || event.key === "ArrowUp") {
      pickChoice = Math.max(0, pickChoice - 1);
      return;
    }
    if (event.key === "Enter") {
      const spec = pickQueue[pickIndex];
      if (!spec) {
        return;
      }
      const value = spec.options[pickChoice] ?? "";
      askAnswers = { ...askAnswers, [`pick:${spec.spec}`]: value };
      if (pickIndex + 1 >= pickQueue.length) {
        pickQueue = [];
        mode = "normal";
        void continueAfterPrompts();
      } else {
        pickIndex += 1;
        pickChoice = 0;
      }
    }
  }

  function gotoFile() {
    const item = selectedItems[0];
    if (!item) {
      return;
    }
    clearSelection();
    void invoke("open_target", { text: item.text });
  }

  async function saveKeymaps() {
    const next = await invoke<{ leader: string; maps: KeyMap[] }>("set_keymaps", {
      keymaps: { leader: mapLeader, maps },
    });
    mapLeader = next.leader;
    maps = next.maps;
  }

  function runMappedRhs(rhs: { kind: "keys"; keys: string } | { kind: "cmd"; command: string }) {
    if (rhs.kind === "cmd") {
      void runColon(rhs.command);
      return;
    }
    mapsEnabled = false;
    pending = "";
    whichPrefix = "";
    try {
      for (const token of tokenizeKeys(rhs.keys)) {
        const key = tokenToEventKey(token);
        if (key === null) {
          continue;
        }
        onDocumentKeydown(new KeyboardEvent("keydown", { key, cancelable: true, bubbles: true }));
      }
    } finally {
      mapsEnabled = true;
    }
  }

  async function runColon(raw: string) {
    const line = raw.trim().replace(/^:/, "");
    mode = "normal";
    colonInput = "";
    colonHistIndex = -1;
    if (line.length > 0 && colonHistory[colonHistory.length - 1] !== line) {
      colonHistory = [...colonHistory, line];
    }
    if (line === "help" || line.startsWith("help ")) {
      const topic = line === "help" ? null : line.slice(5).trim();
      helpText = await invoke<string>("get_help", { topic });
      helpTopics = await invoke<string[]>("help_topics");
      mode = "help";
      return;
    }
    if (line.startsWith("export ")) {
      const path = line.slice(7).trim();
      try {
        await invoke("export_items", { path });
      } catch {
        // 書けなければそのまま
      }
      return;
    }
    if (line.startsWith("import ")) {
      const path = line.slice(7).trim();
      try {
        items = await invoke<Item[]>("import_items", { path });
      } catch {
        // 読めなければそのまま
      }
      return;
    }
    if (line === "clear") {
      colonInput = "clear yes";
      mode = "colon";
      return;
    }
    if (line === "clear yes") {
      items = await invoke<Item[]>("clear_unpinned");
      return;
    }
    if (line === "quote") {
      void pasteSelection(false, false, false, "\n", "> ");
      return;
    }
    if (line === "bullet") {
      void pasteSelection(false, false, false, "\n", "* ");
      return;
    }
    if (line === "type") {
      void pasteSelection(false, false, false, "\n", undefined, true);
      return;
    }
    if (line === "echo" || line.startsWith("echo ")) {
      const expr = line === "echo" ? "" : line.slice(5);
      if (expr.trim().length === 0) {
        return;
      }
      try {
        await invoke("paste_echo", { expr });
      } catch {
        // 計算できなければ貼らない
      }
      return;
    }
    if (line.startsWith("s/")) {
      const rest = line.slice(2);
      const cut = rest.indexOf("/");
      if (cut <= 0) {
        return;
      }
      const old = rest.slice(0, cut);
      const next = rest.slice(cut + 1);
      void substituteSelection(old, next);
      return;
    }
    if (line === "dedup") {
      colonInput = "dedup yes";
      mode = "colon";
      return;
    }
    if (line === "dedup yes") {
      items = await invoke<Item[]>("dedup_items");
      return;
    }
    if (line === "settings") {
      try {
        await invoke("open_settings_file");
      } catch {
        // エディタが無いときは何もしない。
      }
      return;
    }
    if (line === "set" || line.startsWith("set ")) {
      const rest = line === "set" ? "" : line.slice(4);
      try {
        const listed = await invoke<string | null>("apply_set", { rest });
        if (listed != null && listed.length > 0) {
          helpText = listed;
          mode = "help";
        }
      } catch {
        // 書き方が違うときは何もしない
      }
      return;
    }
    if (line === "sort") {
      void sortSelection();
      return;
    }
    if (line === "sh" || line === "sh ") {
      return;
    }
    if (line.startsWith("sh ")) {
      try {
        await invoke("paste_script", { script: line.slice(3) });
      } catch {
        // 失敗したら貼らない
      }
      return;
    }
    if (line === "@") {
      try {
        await invoke("paste_last_script");
      } catch {
        // 無ければ何もしない
      }
      return;
    }
    if (line === "mapleader" || line.startsWith("mapleader ")) {
      const parsed = parseMapleaderArgs(line === "mapleader" ? "" : line.slice(10));
      if (parsed?.kind === "show") {
        helpText = formatMaps(mapLeader, maps);
        mode = "help";
        return;
      }
      if (parsed?.kind === "set") {
        mapLeader = parsed.leader;
        void saveKeymaps();
      }
      return;
    }
    if (line === "unmap" || line.startsWith("unmap ")) {
      const lhs = parseUnmapArgs(line === "unmap" ? "" : line.slice(6));
      if (lhs) {
        maps = removeMap(maps, lhs);
        void saveKeymaps();
      }
      return;
    }
    if (line === "map" || line.startsWith("map ")) {
      const parsed = parseMapArgs(line === "map" ? "" : line.slice(4));
      if (parsed?.kind === "list") {
        helpText = formatMaps(mapLeader, maps);
        mode = "help";
        return;
      }
      if (parsed?.kind === "set") {
        maps = upsertMap(maps, parsed.lhs, parsed.rhs);
        void saveKeymaps();
      }
    }
  }

  function onColonKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (isEscape(event)) {
      event.preventDefault();
      event.stopPropagation();
      mode = "normal";
      colonInput = "";
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      void runColon(colonInput);
      return;
    }
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      if (colonHistory.length === 0) {
        return;
      }
      if (colonHistIndex < 0) {
        colonDraft = colonInput;
      }
      if (event.key === "ArrowUp") {
        colonHistIndex = colonHistIndex < 0 ? colonHistory.length - 1 : Math.max(0, colonHistIndex - 1);
        colonInput = colonHistory[colonHistIndex];
      } else if (colonHistIndex < 0) {
        return;
      } else if (colonHistIndex >= colonHistory.length - 1) {
        colonHistIndex = -1;
        colonInput = colonDraft;
      } else {
        colonHistIndex += 1;
        colonInput = colonHistory[colonHistIndex];
      }
      return;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      event.stopPropagation();
      const trimmed = colonInput.trim();
      if (trimmed.startsWith("help")) {
        const prefix = trimmed === "help" ? "" : trimmed.slice(4).trim();
        const list = helpTopics.filter((name) => name.startsWith(prefix));
        if (list.length === 0) {
          return;
        }
        const current = prefix;
        let index = list.indexOf(current);
        if (event.shiftKey) {
          index = index <= 0 ? list.length - 1 : index - 1;
        } else {
          index = index < 0 || index >= list.length - 1 ? 0 : index + 1;
        }
        colonInput = `help ${list[index]}`;
        return;
      }
      if (colonSuggestions.length === 0) {
        return;
      }
      const token = trimmed.split(/[\s/]/)[0] ?? "";
      let index = colonSuggestions.indexOf(token);
      if (event.shiftKey) {
        index = index <= 0 ? colonSuggestions.length - 1 : index - 1;
      } else {
        index = index < 0 || index >= colonSuggestions.length - 1 ? 0 : index + 1;
      }
      colonInput = applyColonCompletion(colonInput, colonSuggestions[index]);
    }
  }

  function jumpFind(dir: number) {
    if (findChar.length === 0 || filtered.length === 0) {
      return;
    }
    const needle = findChar.toLocaleLowerCase();
    for (let step = 1; step <= filtered.length; step += 1) {
      const index = (selected + dir * step + filtered.length * 8) % filtered.length;
      const first = filtered[index].text.charAt(0).toLocaleLowerCase();
      if (first === needle) {
        selected = index;
        return;
      }
    }
  }

  function isCtrl(event: KeyboardEvent, key: string) {
    return (
      event.ctrlKey &&
      !event.altKey &&
      !event.metaKey &&
      event.key.toLowerCase() === key
    );
  }

  function isEscape(event: KeyboardEvent) {
    return event.key === "Escape" || isCtrl(event, "[");
  }

  function handleWindowKeys(event: KeyboardEvent) {
    if (!event.ctrlKey || event.altKey || event.metaKey) {
      return false;
    }
    const step = 24;
    const dx = event.key === "ArrowLeft" ? -step : event.key === "ArrowRight" ? step : 0;
    const dy = event.key === "ArrowUp" ? -step : event.key === "ArrowDown" ? step : 0;
    if (dx === 0 && dy === 0) {
      return false;
    }
    event.preventDefault();
    event.stopPropagation();
    pending = "";
    if (event.shiftKey) {
      void invoke("resize_window", { dw: dx, dh: dy });
    } else {
      void invoke("nudge_window", { dx, dy });
    }
    return true;
  }

  function onSearchKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (isEscape(event)) {
      event.preventDefault();
      event.stopPropagation();
      mode = "normal";
      pending = "";
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      void pasteSelection(event.ctrlKey, event.shiftKey);
      return;
    }
    if (handleWindowKeys(event)) {
      return;
    }
    if (isCtrl(event, "n")) {
      event.preventDefault();
      event.stopPropagation();
      pending = "";
      move(1);
      return;
    }
    if (isCtrl(event, "p")) {
      event.preventDefault();
      event.stopPropagation();
      pending = "";
      move(-1);
      return;
    }
    if (isCtrl(event, "c")) {
      event.preventDefault();
      event.stopPropagation();
      void copySelection();
      return;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      move(event.key === "ArrowDown" ? 1 : -1);
      return;
    }
    if (event.key === "Tab" && searchSuggestions.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      query = applyTagCompletion(query, searchSuggestions[0]);
    }
  }

  function onTagKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (isEscape(event)) {
      event.preventDefault();
      event.stopPropagation();
      mode = "normal";
      pending = "";
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      const tag = tagInput.trim().length > 0 ? tagInput : (tagSuggestions[0] ?? "");
      mode = "normal";
      void applyTag(tag, tagAdd);
      return;
    }
    if (handleWindowKeys(event)) {
      return;
    }
    if (event.key === "Tab" && tagSuggestions.length > 0) {
      event.preventDefault();
      event.stopPropagation();
      tagInput = tagSuggestions[0];
    }
  }

  function onDocumentKeydown(event: KeyboardEvent) {
    if (event.isComposing) {
      return;
    }
    if (
      event.key === "Shift" ||
      event.key === "Control" ||
      event.key === "Alt" ||
      event.key === "Meta"
    ) {
      return;
    }
    if (mode === "search" || mode === "tag" || mode === "colon" || mode === "ask") {
      return;
    }
    if (mode === "pick") {
      onPickKeydown(event);
      return;
    }
    if (mode === "help") {
      if (isEscape(event)) {
        event.preventDefault();
        mode = "normal";
        helpText = "";
        return;
      }
      if (event.key === "j" || event.key === "ArrowDown") {
        event.preventDefault();
        helpEl?.scrollBy(0, 24);
        return;
      }
      if (event.key === "k" || event.key === "ArrowUp") {
        event.preventDefault();
        helpEl?.scrollBy(0, -24);
        return;
      }
      return;
    }
    if (mode === "editing") {
      if (isEscape(event)) {
        event.preventDefault();
        cancelEdit();
      }
      if (event.key === "Enter" && event.ctrlKey) {
        event.preventDefault();
        void saveEdit();
      }
      return;
    }

    if (isEscape(event)) {
      event.preventDefault();
      if (pending.length > 0 || whichPrefix.length > 0) {
        pending = "";
        whichPrefix = "";
        return;
      }
      if (anchor !== null) {
        clearSelection();
      } else {
        void hidePicker();
      }
      return;
    }
    if (mapsEnabled) {
      const token = eventToToken(event);
      if (token !== null) {
        const leaderWait =
          pending === "<leader>" ||
          (pending === "" && hasLeaderMaps(maps) && isLeaderToken(token, mapLeader));
        const maybeMap =
          leaderWait ||
          pending.length > 0 ||
          maps.some((entry) => entry.lhs.startsWith(token));
        if (maybeMap) {
          const result = matchMap(maps, pending, token, mapLeader);
          if (result.kind === "hit") {
            event.preventDefault();
            event.stopPropagation();
            pending = "";
            whichPrefix = "";
            runMappedRhs(result.rhs);
            return;
          }
          if (result.kind === "prefix") {
            event.preventDefault();
            event.stopPropagation();
            pending = combinePending(pending, token, mapLeader);
            return;
          }
          if (pending === "<leader>") {
            event.preventDefault();
            event.stopPropagation();
            pending = "";
            whichPrefix = "";
            return;
          }
        }
      }
    }
    if (pending === "f") {
      event.preventDefault();
      pending = "";
      if (event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey) {
        findChar = event.key;
        jumpFind(1);
      }
      return;
    }
    if (pending === "g" && event.key === "Enter") {
      event.preventDefault();
      pending = "";
      void pasteSelection(false, false, true);
      return;
    }
    if (pending === "g" && event.key === "J") {
      event.preventDefault();
      pending = "";
      if (anchor !== null) {
        void pasteSelection(false, false, false, ",");
      }
      return;
    }
    if (pending === "g" && event.key === "T") {
      event.preventDefault();
      pending = "";
      if (anchor !== null) {
        void pasteSelection(false, false, false, "\t");
      }
      return;
    }
    if (pending === "g" && event.key === ">") {
      event.preventDefault();
      pending = "";
      void pasteSelection(false, false, false, "\n", "> ");
      return;
    }
    if (pending === "g" && event.key === "*") {
      event.preventDefault();
      pending = "";
      void pasteSelection(false, false, false, "\n", "* ");
      return;
    }
    if (pending === "g" && event.key === "p") {
      event.preventDefault();
      pending = "";
      void replayLastPaste();
      return;
    }
    if (pending === "g" && event.key === "e") {
      event.preventDefault();
      pending = "";
      preview = !preview;
      return;
    }
    if (pending === "g" && event.key === "?") {
      event.preventDefault();
      pending = "";
      info = !info;
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      pending = "";
      void pasteSelection(event.ctrlKey, event.shiftKey);
      return;
    }
    if (event.key === "J") {
      event.preventDefault();
      pending = "";
      if (anchor !== null) {
        void pasteSelection(false, false, false, " ");
      }
      return;
    }
    if (isCtrl(event, "r")) {
      event.preventDefault();
      pending = "";
      void redoChange();
      return;
    }
    if (isCtrl(event, "d")) {
      event.preventDefault();
      pending = "";
      pageMove(1);
      return;
    }
    if (isCtrl(event, "u")) {
      event.preventDefault();
      pending = "";
      pageMove(-1);
      return;
    }
    if (isCtrl(event, "c")) {
      event.preventDefault();
      pending = "";
      void copySelection();
      return;
    }
    if (isCtrl(event, "n")) {
      event.preventDefault();
      pending = "";
      move(1);
      return;
    }
    if (isCtrl(event, "p")) {
      event.preventDefault();
      pending = "";
      move(-1);
      return;
    }
    if (handleWindowKeys(event)) {
      return;
    }
    if (!event.shiftKey && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      pending = "";
      void pasteRow(Number(event.key) - 1, event.ctrlKey);
      return;
    }
    if (event.key === "/") {
      event.preventDefault();
      mode = "search";
      pending = "";
      return;
    }
    if (event.key === "j" || event.key === "ArrowDown") {
      event.preventDefault();
      pending = "";
      move(1);
      return;
    }
    if (event.key === "k" || event.key === "ArrowUp") {
      event.preventDefault();
      pending = "";
      move(-1);
      return;
    }
    if (event.key === "V") {
      event.preventDefault();
      pending = "";
      anchor = anchor === null ? selected : null;
      return;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      pending = "";
      cycleTag(event.shiftKey ? -1 : 1);
      return;
    }
    if (event.key === "o") {
      event.preventDefault();
      pending = "";
      void openBlank();
      return;
    }
    if (event.key === ":") {
      event.preventDefault();
      pending = "";
      colonInput = "";
      mode = "colon";
      if (helpTopics.length === 0) {
        void invoke<string[]>("help_topics").then((topics) => {
          helpTopics = topics;
        });
      }
      return;
    }
    if (event.key === "e") {
      event.preventDefault();
      pending = "";
      startEdit();
      return;
    }
    if (event.key === "E") {
      event.preventDefault();
      pending = "";
      void editExternal();
      return;
    }
    if (event.key === "t") {
      event.preventDefault();
      pending = "";
      startTagInput(true);
      return;
    }
    if (event.key === "T") {
      event.preventDefault();
      pending = "";
      startTagInput(false);
      return;
    }
    if (event.key === "m") {
      event.preventDefault();
      pending = "";
      void applyPin(!selectedItems.every((item) => item.pinned));
      return;
    }
    if (event.key === "a") {
      event.preventDefault();
      pending = "";
      if (contextKey) {
        contextOnly = !contextOnly;
        selected = 0;
      }
      return;
    }
    if (event.key === "+" || event.key === "=") {
      if (event.key === "=" && !event.shiftKey) {
        pending = "";
        return;
      }
      event.preventDefault();
      pending = "";
      void movePins(-1);
      return;
    }
    if (event.key === "-" || event.key === "_") {
      event.preventDefault();
      pending = "";
      void movePins(1);
      return;
    }
    if (event.key === "S") {
      event.preventDefault();
      pending = "";
      void splitSelection();
      return;
    }
    if (event.key === "M") {
      event.preventDefault();
      pending = "";
      void mergeSelection();
      return;
    }
    if (event.key === "c") {
      event.preventDefault();
      pending = "";
      void cloneSelection();
      return;
    }
    if (event.key === ";") {
      event.preventDefault();
      pending = "";
      jumpFind(1);
      return;
    }
    if (event.key === ",") {
      event.preventDefault();
      pending = "";
      jumpFind(-1);
      return;
    }
    if (event.key === "u") {
      event.preventDefault();
      pending = "";
      void undoChange();
      return;
    }
    if (event.key === ".") {
      event.preventDefault();
      pending = "";
      void repeatChange();
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
    if (event.key === "f") {
      event.preventDefault();
      if (pending === "g") {
        pending = "";
        gotoFile();
      } else {
        pending = "f";
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
    if (event.key === "p") {
      event.preventDefault();
      pending = "";
      void putYanked(false);
      return;
    }
    if (event.key === "P") {
      event.preventDefault();
      pending = "";
      void putYanked(true);
      return;
    }
    if (event.key === "Y") {
      event.preventDefault();
      pending = "";
      yankSelection();
      return;
    }
    if (event.key === "y") {
      event.preventDefault();
      if (pending === "y") {
        pending = "";
        yankSelection();
      } else {
        pending = "y";
      }
      return;
    }
    if (event.key === "d") {
      event.preventDefault();
      if (pending === "d") {
        pending = "";
        void deleteSelection();
      } else {
        pending = "d";
      }
      return;
    }
    pending = "";
  }

  onMount(() => {
    const onKey = (event: KeyboardEvent) => onDocumentKeydown(event);
    window.addEventListener("keydown", onKey, true);

    const unlistenOpened = listen<Item[]>("picker-opened", (event) => {
      openPicker(event.payload);
    });
    const unlistenChanged = listen<Item[]>("items-changed", (event) => {
      items = event.payload;
    });

    void invoke<{ leader: string; maps: KeyMap[] }>("get_keymaps").then((next) => {
      mapLeader = next.leader;
      maps = next.maps;
    });
    const unlistenSettings = listen<{ leader: string; maps: KeyMap[] }>("settings-changed", (event) => {
      mapLeader = event.payload.leader;
      maps = event.payload.maps;
    });

    let stopDrop: (() => void) | undefined;
    void import("@tauri-apps/api/webview").then(({ getCurrentWebview }) => {
      void getCurrentWebview()
        .onDragDropEvent((event) => {
          if (event.payload.type !== "drop") {
            return;
          }
          void invoke<Item[]>("drop_paths", { paths: event.payload.paths }).then((next) => {
            items = next;
          });
        })
        .then((stop) => {
          stopDrop = stop;
        });
    });

    return () => {
      window.removeEventListener("keydown", onKey, true);
      void unlistenOpened.then((stop) => stop());
      void unlistenChanged.then((stop) => stop());
      void unlistenSettings.then((stop) => stop());
      stopDrop?.();
    };
  });
</script>

<div class="picker">
  <div class="drag" data-tauri-drag-region></div>
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
  {:else if mode === "help"}
    <pre class="help" bind:this={helpEl}>{helpText}</pre>
  {:else}
    {#if mode === "colon"}
      <input
        bind:this={colonEl}
        bind:value={colonInput}
        class="search"
        placeholder=":help  :map  :quote  :bullet"
        onkeydown={onColonKeydown}
      />
      {#if colonSuggestions.length > 0}
        <ul class="suggest">
          {#each colonSuggestions as name (name)}
            <li>
              <button
                type="button"
                onclick={() => {
                  colonInput = applyColonCompletion(colonInput, name);
                  colonEl?.focus();
                }}>{name}</button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
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
    {#if mode === "ask"}
      <input
        bind:this={askEl}
        bind:value={askDraft}
        class="search"
        placeholder={askQueue[askIndex] ?? ""}
        onkeydown={onAskKeydown}
      />
    {/if}
    {#if mode === "pick" && pickQueue[pickIndex]}
      <div class="search pick">
        <span class="hint">{pickQueue[pickIndex].spec}</span>
        <ul class="suggest">
          {#each pickQueue[pickIndex].options as option, index (option)}
            <li>
              <button
                type="button"
                class:active={index === pickChoice}
                onclick={() => {
                  pickChoice = index;
                  const spec = pickQueue[pickIndex];
                  askAnswers = { ...askAnswers, [`pick:${spec.spec}`]: option };
                  if (pickIndex + 1 >= pickQueue.length) {
                    pickQueue = [];
                    mode = "normal";
                    void continueAfterPrompts();
                  } else {
                    pickIndex += 1;
                    pickChoice = 0;
                  }
                }}>{option}</button
              >
            </li>
          {/each}
        </ul>
      </div>
    {/if}
    {#if mode === "tag"}
      <input
        bind:this={tagEl}
        bind:value={tagInput}
        class="search"
        placeholder={tagAdd ? `tag +  (${selectedIds.length})` : `tag −  (${selectedIds.length})`}
        onkeydown={onTagKeydown}
      />
      {#if tagSuggestions.length > 0}
        <ul class="suggest">
          {#each tagSuggestions as tag (tag)}
            <li>
              <button
                type="button"
                onclick={() => {
                  tagInput = tag;
                  tagEl?.focus();
                }}>{tag}</button
              >
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
    {#if preview && currentItem()}
      <pre class="preview">{isSecret(currentItem()!) ? "••••" : currentItem()!.text}</pre>
    {/if}
    {#if info && currentItem()}
      <pre class="preview">{`count ${currentItem()!.paste_count}
context ${currentItem()!.contexts.join(" ") || "—"}
tags ${currentItem()!.tags.map((tag) => `#${tag}`).join(" ") || "—"}`}</pre>
    {/if}
    {#if whichKeys.length > 0}
      <ul class="suggest which">
        {#each whichKeys as entry (entry.key)}
          <li><span class="pin">{entry.key}</span> {entry.label}</li>
        {/each}
      </ul>
    {/if}
    <ul class="list" bind:this={listEl}>
      {#each filtered as item, index (item.id)}
        <li
          class:active={index === selected}
          class:ranged={anchor !== null && index >= range[0] && index <= range[1]}
        >
          <button
            type="button"
            class="row"
            tabindex="-1"
            onclick={() => {
              clearSelection();
              selected = index;
            }}
            ondblclick={() => {
              selected = index;
              void pasteSelection(false);
            }}
          >
            <span class="gutter">{index < 9 ? index + 1 : ""}</span>
            <span class="body">
              <span class="text">{isSecret(item) ? "••••" : item.text}</span>
              {#if item.pinned || item.tags.length > 0}
                <span class="meta">
                  {#if item.pinned}<span class="pin">pin</span>{/if}
                  {item.tags.map((tag) => `#${tag}`).join(" ")}
                </span>
              {/if}
            </span>
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

  .drag {
    height: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #151515;
    border-bottom: 1px solid #333;
    cursor: grab;
    app-region: drag;
    -webkit-app-region: drag;
  }

  .drag::after {
    content: "";
    width: 36px;
    height: 3px;
    border-radius: 2px;
    background: #666;
    pointer-events: none;
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
    flex-direction: row;
    align-items: flex-start;
    gap: 6px;
    width: 100%;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    padding: 2px 8px;
    font: inherit;
    line-height: 1.25;
    cursor: pointer;
  }

  li.ranged .row {
    background: #234;
  }

  .suggest button.active,
  li.active .row,
  .row:hover {
    background: #2c4a6e;
  }

  .gutter {
    flex-shrink: 0;
    width: 10px;
    text-align: right;
    color: #778;
    font-size: 11px;
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .text {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 4;
    line-clamp: 4;
    overflow: hidden;
    max-width: 100%;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .meta,
  .hint,
  .empty {
    color: #9aa;
    font-size: 11px;
  }

  .pin {
    color: #e0b060;
    margin-right: 4px;
  }

  .empty,
  .hint {
    padding: 8px 10px;
  }

  .edit {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
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

  .help,
  .preview {
    margin: 0;
    padding: 8px 10px;
    white-space: pre-wrap;
    overflow: auto;
    font: 12px/1.4 ui-monospace, monospace;
    background: #111;
    border-bottom: 1px solid #333;
    max-height: 40%;
  }

  .help {
    flex: 1;
    max-height: none;
  }
</style>

export const DEFAULT_LEADER = "\\";

export type KeyMap = { lhs: string; rhs: string };

export type MapRhs = { kind: "keys"; keys: string } | { kind: "cmd"; command: string };

export type MapMatch = { kind: "hit"; rhs: MapRhs } | { kind: "prefix" } | { kind: "miss" };

const FORBIDDEN = new Set(["<Esc>", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);

export function normalizeTags(value: string): string {
  return value
    .replace(/<leader>/gi, "<leader>")
    .replace(/<cmd>/gi, "<cmd>")
    .replace(/<CR>/gi, "<CR>")
    .replace(/<Enter>/gi, "<CR>")
    .replace(/<Esc>/gi, "<Esc>")
    .replace(/<Space>/gi, "<Space>")
    .replace(/<Tab>/gi, "<Tab>");
}

export function tokenizeKeys(seq: string): string[] {
  const normalized = normalizeTags(seq);
  const tokens: string[] = [];
  let i = 0;
  while (i < normalized.length) {
    if (normalized[i] === "<") {
      const close = normalized.indexOf(">", i);
      if (close > i) {
        tokens.push(normalized.slice(i, close + 1));
        i = close + 1;
        continue;
      }
    }
    tokens.push(normalized[i]);
    i += 1;
  }
  return tokens;
}

export function tokenToEventKey(token: string): string | null {
  if (token === "<leader>" || token.startsWith("<cmd")) {
    return null;
  }
  if (token === "<CR>") {
    return "Enter";
  }
  if (token === "<Tab>") {
    return "Tab";
  }
  if (token === "<Space>") {
    return " ";
  }
  if (token === "<Esc>") {
    return "Escape";
  }
  if (token.length === 1) {
    return token;
  }
  return null;
}

export function eventToToken(event: KeyboardEvent): string | null {
  if (event.altKey || event.metaKey || event.ctrlKey) {
    return null;
  }
  if (event.key === "Escape") {
    return "<Esc>";
  }
  if (event.key === "Enter") {
    return "<CR>";
  }
  if (event.key === "Tab") {
    return "<Tab>";
  }
  if (event.key === " ") {
    return "<Space>";
  }
  if (event.key.length === 1) {
    return event.key;
  }
  return null;
}

export function isLeaderToken(token: string, leader: string): boolean {
  if (token === "<leader>") {
    return true;
  }
  if (leader === "\\" && (token === "\\" || token === "¥" || token === "￥")) {
    return true;
  }
  return token === leader;
}

export function forbiddenToken(token: string): boolean {
  return FORBIDDEN.has(token);
}

export function validLeader(value: string): boolean {
  const tokens = tokenizeKeys(value.trim());
  if (tokens.length !== 1) {
    return false;
  }
  const token = tokens[0];
  if (token === "<Space>") {
    return true;
  }
  if (token.length !== 1) {
    return false;
  }
  return !forbiddenToken(token);
}

export function canonicalizeLeader(value: string): string {
  const token = tokenizeKeys(value.trim())[0] ?? "";
  return token === "<Space>" ? " " : token;
}

export function validLhs(lhs: string): boolean {
  const tokens = tokenizeKeys(lhs);
  if (tokens.length === 0 || tokens.some((token) => token === "<cmd>" || forbiddenToken(token))) {
    return false;
  }
  if (tokens[0] === "<leader>") {
    return tokens.length === 2;
  }
  return true;
}

export function parseRhs(raw: string): MapRhs | null {
  const value = normalizeTags(raw.trim());
  if (value.length === 0) {
    return null;
  }
  if (value.startsWith("<cmd>")) {
    const command = value.slice(5).trim().replace(/^:/, "");
    return command.length > 0 ? { kind: "cmd", command } : null;
  }
  if (value.startsWith(":")) {
    const command = value.slice(1).trim();
    return command.length > 0 ? { kind: "cmd", command } : null;
  }
  return { kind: "keys", keys: value };
}

export function parseMapArgs(rest: string): { kind: "list" } | { kind: "set"; lhs: string; rhs: string } | null {
  const line = normalizeTags(rest.trim());
  if (line.length === 0) {
    return { kind: "list" };
  }
  const cut = line.search(/\s/);
  if (cut < 0) {
    return null;
  }
  const lhs = line.slice(0, cut);
  const rhs = line.slice(cut).trim();
  if (!validLhs(lhs) || parseRhs(rhs) === null) {
    return null;
  }
  return { kind: "set", lhs, rhs };
}

export function parseUnmapArgs(rest: string): string | null {
  const lhs = normalizeTags(rest.trim());
  return validLhs(lhs) ? lhs : null;
}

export function parseMapleaderArgs(rest: string): { kind: "show" } | { kind: "set"; leader: string } | null {
  const line = rest.trim();
  if (line.length === 0) {
    return { kind: "show" };
  }
  if (!validLeader(line)) {
    return null;
  }
  return { kind: "set", leader: canonicalizeLeader(line) };
}

export function combinePending(pending: string, token: string, leader: string): string {
  if (pending === "<leader>") {
    return `<leader>${token}`;
  }
  if (pending.length === 0 && isLeaderToken(token, leader)) {
    return "<leader>";
  }
  return pending + token;
}

export function matchMap(maps: KeyMap[], pending: string, token: string, leader: string): MapMatch {
  const sequence = combinePending(pending, token, leader);
  const hit = maps.find((entry) => entry.lhs === sequence);
  if (hit) {
    const rhs = parseRhs(hit.rhs);
    if (rhs) {
      return { kind: "hit", rhs };
    }
  }
  if (maps.some((entry) => entry.lhs.startsWith(sequence) && entry.lhs.length > sequence.length)) {
    return { kind: "prefix" };
  }
  return { kind: "miss" };
}

export function hasLeaderMaps(maps: KeyMap[]): boolean {
  return maps.some((entry) => entry.lhs.startsWith("<leader>"));
}

export function whichKeysForMaps(maps: KeyMap[], prefix: string): { key: string; label: string }[] {
  return maps
    .filter((entry) => entry.lhs.startsWith(prefix) && entry.lhs.length > prefix.length)
    .map((entry) => ({
      key: entry.lhs.slice(prefix.length),
      label: entry.rhs,
    }));
}

export function formatMaps(leader: string, maps: KeyMap[]): string {
  const head = `mapleader ${leader === " " ? "<Space>" : leader}`;
  if (maps.length === 0) {
    return `${head}\n\nマップなし`;
  }
  const rows = maps.map((entry) => `${entry.lhs.padEnd(16)}${entry.rhs}`);
  return `${head}\n\n${rows.join("\n")}`;
}

export function upsertMap(maps: KeyMap[], lhs: string, rhs: string): KeyMap[] {
  return [...maps.filter((entry) => entry.lhs !== lhs), { lhs, rhs }];
}

export function removeMap(maps: KeyMap[], lhs: string): KeyMap[] {
  return maps.filter((entry) => entry.lhs !== lhs);
}

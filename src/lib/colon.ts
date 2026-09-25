export const COLON_COMMANDS = [
  "help",
  "showerror",
  "sh",
  "!!",
  "export",
  "import",
  "clear",
  "quote",
  "type",
  "format",
  "raw",
  "join",
  "open",
  "log",
  "echo",
  "s",
  "@",
  "dedup",
  "sort",
  "map",
  "unmap",
  "mapleader",
  "set",
  "settings",
  "tags",
  "from",
  "clip",
  "sel",
  "camel",
  "pascal",
  "snake",
  "kebab",
  "upper",
  "lower",
  "json",
  "xml",
  "split",
  "col",
  "get",
  "diff",
  "only",
  "filter",
  "put",
  "each",
  "crypt",
  "decrypt",
];

/** `> clip` は行き先にしない。呼び出し側の形だけ残す。 */
export function stripClipSink(line: string): { cmd: string; clip: boolean } {
  return { cmd: line.trim().replace(/^:/, ""), clip: false };
}

function colonSegments(input: string): string[] {
  return splitUnquotedPipe(stripClipSink(input).cmd);
}

function activeColonLine(input: string): string {
  const segments = colonSegments(input);
  return segments[segments.length - 1] ?? "";
}

export function colonCommandToken(input: string): string {
  const line = activeColonLine(input);
  if (line.startsWith("help")) {
    return "help";
  }
  if (line.startsWith("!!")) {
    return "!!";
  }
  return line.split(/\s/)[0] ?? "";
}

/** `:` の一覧で残す行。`V` なら範囲の先頭。それ以外は選択行。 */
export function keepVisibleIndex(selected: number, anchor: number | null, colon: boolean): number {
  if (!colon || anchor === null) {
    return selected;
  }
  return Math.min(anchor, selected);
}

export function matchingColonCommands(input: string): string[] {
  const segments = colonSegments(input);
  const line = segments[segments.length - 1] ?? "";
  if (line.startsWith("help ") || line === "help") {
    return [];
  }
  const token = line.startsWith("!!") ? "!!" : (line.split(/[\s/]/)[0] ?? "");
  const names = segments.length > 1 ? [...COLON_COMMANDS, "add", "show"] : COLON_COMMANDS;
  return names.filter((name) => name.startsWith(token));
}

function completedColonCommand(command: string): string {
  if (command === "s") {
    return "s/";
  }
  if (command === "!!") {
    return "!!sh ";
  }
  if (command === "export" || command === "import" || command === "sh" || command === "help" || command === "echo" || command === "log") {
    return `${command} `;
  }
  if (command === "map" || command === "unmap" || command === "mapleader" || command === "set" || command === "quote" || command === "join" || command === "filter" || command === "crypt" || command === "decrypt") {
    return `${command} `;
  }
  return command;
}

export function applyColonCompletion(input: string, command: string): string {
  const next = completedColonCommand(command);
  const segments = colonSegments(input);
  if (segments.length < 2) {
    return next;
  }
  return `${segments.slice(0, -1).join(" | ")} | ${next}`;
}

export function bangFilterScript(line: string): string | null {
  if (line.startsWith("!!sh ")) {
    return line.slice(5);
  }
  if (line.startsWith("!! sh ")) {
    return line.slice(6);
  }
  return null;
}

/** `"* "` や `"\\t"`。閉じられていなければ null。 */
export function unquoteColonArg(raw: string): string | null {
  const text = raw.trimStart();
  const quote = text[0];
  if (quote !== '"' && quote !== "'") {
    return null;
  }
  let out = "";
  let escaped = false;
  for (let i = 1; i < text.length; i += 1) {
    const ch = text[i];
    if (escaped) {
      if (ch === "t") {
        out += "\t";
      } else if (ch === "n") {
        out += "\n";
      } else {
        out += ch;
      }
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === quote) {
      if (text.slice(i + 1).trim().length > 0) {
        return null;
      }
      return out;
    }
    out += ch;
  }
  return null;
}

function colonArg(rest: string, fallback: string): string {
  const quoted = unquoteColonArg(rest);
  if (quoted !== null) {
    return quoted;
  }
  const plain = rest.trim();
  return plain.length === 0 ? fallback : plain;
}

const QUOTE_SPLIT = "\u0001";

function readQuoted(raw: string): { value: string; rest: string } | null {
  const text = raw.trimStart();
  const quote = text[0];
  if (quote !== '"' && quote !== "'") {
    return null;
  }
  let out = "";
  let escaped = false;
  for (let i = 1; i < text.length; i += 1) {
    const ch = text[i];
    if (escaped) {
      if (ch === "t") {
        out += "\t";
      } else if (ch === "n") {
        out += "\n";
      } else {
        out += ch;
      }
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === quote) {
      return { value: out, rest: text.slice(i + 1) };
    }
    out += ch;
  }
  return null;
}

/** 行頭と行末。引数1つなら行末は空。2つは両方引用。書けなければ null。 */
export function quoteMarks(line: string): { prefix: string; suffix: string } | null {
  if (line === "quote") {
    return { prefix: "> ", suffix: "" };
  }
  if (!line.startsWith("quote ") && !line.startsWith("quote\t")) {
    return null;
  }
  const rest = line.slice(6);
  if (rest.trim().length === 0) {
    return { prefix: "> ", suffix: "" };
  }
  const first = readQuoted(rest);
  if (first && first.rest.trim().length === 0) {
    return { prefix: first.value, suffix: "" };
  }
  if (first) {
    const second = readQuoted(first.rest);
    if (!second || second.rest.trim().length > 0) {
      return null;
    }
    return { prefix: first.value, suffix: second.value };
  }
  return { prefix: colonArg(rest, "> "), suffix: "" };
}

export function encodeQuote(prefix: string, suffix: string): string {
  if (suffix.length === 0) {
    return prefix;
  }
  return `${prefix}${QUOTE_SPLIT}${suffix}`;
}

/** `:quote` の行頭。未指定なら `> `。`:quote "* "` で空白も含めて指定。 */
export function quotePrefix(line: string): string | null {
  const marks = quoteMarks(line);
  if (!marks) {
    return null;
  }
  return marks.prefix;
}

/** `:join` の区切り。未指定なら `,`。`:join "\\t"` はタブ。 */
export function joinSeparator(line: string): string | null {
  if (line === "join") {
    return ",";
  }
  if (line.startsWith("join ") || line.startsWith("join\t")) {
    return colonArg(line.slice(5), ",");
  }
  return null;
}

export type PipeOp = {
  kind:
    | "sh"
    | "quote"
    | "format"
    | "join"
    | "raw"
    | "dot"
    | "clip"
    | "camel"
    | "pascal"
    | "snake"
    | "kebab"
    | "upper"
    | "lower"
    | "json"
    | "xml"
    | "echo"
    | "sel"
    | "filter"
    | "show"
    | "each"
    | "split"
    | "col"
    | "get"
    | "diff"
    | "only"
    | "put"
    | "sub"
    | "crypt"
    | "decrypt";
  arg: string;
  selectionStdin: boolean;
};

export type PipeSink =
  | { kind: "paste" }
  | { kind: "clip" }
  | { kind: "add" }
  | { kind: "set"; name: string }
  | { kind: "open" }
  | { kind: "show" }
  | { kind: "log"; path: string };

export type ParsedPipe =
  | { kind: "none" }
  | { kind: "bad" }
  | { kind: "ok"; ops: PipeOp[]; sink: PipeSink; usesSelection: boolean };

/** `"` `'` の外の `|` で分ける。`\"` は引用の中だけを抜ける。 */
export function splitUnquotedPipe(line: string): string[] {
  const parts: string[] = [];
  let buf = "";
  let quote: string | null = null;
  let escaped = false;
  for (const ch of line) {
    if (escaped) {
      buf += ch;
      escaped = false;
      continue;
    }
    if (quote && ch === "\\") {
      buf += ch;
      escaped = true;
      continue;
    }
    if (quote) {
      if (ch === quote) {
        quote = null;
      }
      buf += ch;
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      buf += ch;
      continue;
    }
    if (ch === "|") {
      parts.push(buf.trim());
      buf = "";
      continue;
    }
    buf += ch;
  }
  parts.push(buf.trim());
  return parts;
}

function varSinkName(part: string): string | null {
  if (!part.startsWith("set ") && !part.startsWith("set\t")) {
    return null;
  }
  const name = part.slice(4).trim();
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name) || name === "paste") {
    return null;
  }
  return name;
}

function filterArg(part: string): string | null {
  if (!part.startsWith("filter ") && !part.startsWith("filter\t")) {
    return null;
  }
  const rest = part.slice(7).trim();
  if (rest.length === 0 || rest === "not") {
    return null;
  }
  let invert = false;
  let raw = rest;
  if (rest.startsWith("not ") || rest.startsWith("not\t")) {
    invert = true;
    raw = rest.slice(4).trim();
    if (raw.length === 0) {
      return null;
    }
  }
  const quoted = unquoteColonArg(raw);
  let needle: string;
  if (quoted !== null) {
    if (quoted.length === 0) {
      return null;
    }
    needle = quoted;
  } else if (raw.startsWith('"') || raw.startsWith("'")) {
    return null;
  } else {
    needle = raw;
  }
  return invert ? `\u0001${needle}` : needle;
}

function splitSeparator(part: string): string | null {
  if (!part.startsWith("split ") && !part.startsWith("split\t")) {
    return null;
  }
  const rest = part.slice(6).trim();
  if (rest.length === 0) {
    return null;
  }
  const quoted = unquoteColonArg(rest);
  if (quoted !== null) {
    return quoted.length > 0 ? quoted : null;
  }
  if (rest.startsWith('"') || rest.startsWith("'")) {
    return null;
  }
  return rest;
}

function colArg(part: string): string | null {
  if (!part.startsWith("col ") && !part.startsWith("col\t")) {
    return null;
  }
  const rest = part.slice(4).trim();
  if (!/^-?\d+$/.test(rest)) {
    return null;
  }
  return rest;
}

function putArg(part: string): string | null {
  if (!part.startsWith("put ") && !part.startsWith("put\t")) {
    return null;
  }
  const rest = part.slice(4).trimStart();
  const split = rest.search(/\s/);
  if (split < 0) {
    return null;
  }
  const pointer = rest.slice(0, split);
  const raw = rest.slice(split).trim();
  if (!pointer.startsWith("/") || raw.length === 0) {
    return null;
  }
  const quoted = unquoteColonArg(raw);
  if (quoted !== null) {
    return `${pointer}\u0001${quoted}`;
  }
  if (raw.startsWith('"') || raw.startsWith("'")) {
    return null;
  }
  return `${pointer}\u0001${raw}`;
}

function subArg(part: string): string | null {
  if (!part.startsWith("s/")) {
    return null;
  }
  const rest = part.slice(2);
  const cut = rest.indexOf("/");
  if (cut <= 0) {
    return null;
  }
  return `${rest.slice(0, cut)}\u0001${rest.slice(cut + 1)}`;
}

function sideArg(part: string, name: string): string | null {
  const prefix = `${name} `;
  const tab = `${name}\t`;
  if (!part.startsWith(prefix) && !part.startsWith(tab)) {
    return null;
  }
  const rest = part.slice(name.length + 1).trim();
  if (rest === "." || rest === "clip") {
    return rest;
  }
  return null;
}

function pointerArg(part: string, name: string): string | null {
  const prefix = `${name} `;
  const tab = `${name}\t`;
  if (!part.startsWith(prefix) && !part.startsWith(tab)) {
    return null;
  }
  const rest = part.slice(name.length + 1).trim();
  if (!rest.startsWith("/")) {
    return null;
  }
  return rest;
}

function sinkOf(part: string): PipeSink | null {
  if (part === "clip") {
    return { kind: "clip" };
  }
  if (part === "add") {
    return { kind: "add" };
  }
  if (part === "open") {
    return { kind: "open" };
  }
  if (part === "show") {
    return { kind: "show" };
  }
  if (part === "log") {
    return { kind: "log", path: "" };
  }
  if (part.startsWith("log ") || part.startsWith("log\t")) {
    return { kind: "log", path: part.slice(4).trim() };
  }
  const name = varSinkName(part);
  if (name) {
    return { kind: "set", name };
  }
  return null;
}

function keyedArg(part: string, name: string): string | null {
  if (part !== name && !part.startsWith(`${name} `) && !part.startsWith(`${name}\t`)) {
    return null;
  }
  const arg = part === name ? "" : part.slice(name.length).trim();
  if (arg.length === 0) {
    return null;
  }
  return arg;
}

function shScript(raw: string): string {
  const quoted = unquoteColonArg(raw);
  if (quoted !== null) {
    return quoted.trim();
  }
  return raw.trim();
}

function stageOf(part: string): PipeOp | null {
  if (part === ".") {
    return { kind: "dot", arg: "", selectionStdin: false };
  }
  if (part === "clip") {
    return { kind: "clip", arg: "", selectionStdin: false };
  }
  if (part === "show") {
    return { kind: "show", arg: "", selectionStdin: false };
  }
  if (part === "sel") {
    return { kind: "sel", arg: "", selectionStdin: false };
  }
  if (part === "raw" || part === "each") {
    return { kind: part, arg: "", selectionStdin: false };
  }
  if (part === "format") {
    return { kind: "format", arg: "", selectionStdin: false };
  }
  if (
    part === "camel" ||
    part === "pascal" ||
    part === "snake" ||
    part === "kebab" ||
    part === "upper" ||
    part === "lower"
  ) {
    return { kind: part, arg: "", selectionStdin: false };
  }
  if (part === "json" || part === "xml") {
    return { kind: part, arg: "", selectionStdin: false };
  }
  if (part === "echo" || part.startsWith("echo ") || part.startsWith("echo\t")) {
    const expr = (part === "echo" ? "" : part.slice(5)).trim();
    if (expr.length === 0) {
      return null;
    }
    return { kind: "echo", arg: expr, selectionStdin: false };
  }
  const marks = quoteMarks(part);
  if (marks) {
    return { kind: "quote", arg: encodeQuote(marks.prefix, marks.suffix), selectionStdin: false };
  }
  const sep = joinSeparator(part);
  if (sep !== null) {
    return { kind: "join", arg: sep, selectionStdin: false };
  }
  const split = splitSeparator(part);
  if (split !== null) {
    return { kind: "split", arg: split, selectionStdin: false };
  }
  const filter = filterArg(part);
  if (filter !== null) {
    return { kind: "filter", arg: filter, selectionStdin: false };
  }
  const column = colArg(part);
  if (column !== null) {
    return { kind: "col", arg: column, selectionStdin: false };
  }
  const pointer = pointerArg(part, "get");
  if (pointer !== null) {
    return { kind: "get", arg: pointer, selectionStdin: false };
  }
  const diffSide = sideArg(part, "diff");
  if (diffSide !== null) {
    return { kind: "diff", arg: diffSide, selectionStdin: false };
  }
  const onlySide = sideArg(part, "only");
  if (onlySide !== null) {
    return { kind: "only", arg: onlySide, selectionStdin: false };
  }
  const put = putArg(part);
  if (put !== null) {
    return { kind: "put", arg: put, selectionStdin: false };
  }
  const sub = subArg(part);
  if (sub !== null) {
    return { kind: "sub", arg: sub, selectionStdin: false };
  }
  const crypt = keyedArg(part, "crypt");
  if (crypt !== null) {
    return { kind: "crypt", arg: crypt, selectionStdin: false };
  }
  const decrypt = keyedArg(part, "decrypt");
  if (decrypt !== null) {
    return { kind: "decrypt", arg: decrypt, selectionStdin: false };
  }
  if (part === "sh" || part.startsWith("sh ") || part.startsWith("sh\t")) {
    const script = shScript(part === "sh" ? "" : part.slice(3));
    if (script.length === 0) {
      return null;
    }
    return { kind: "sh", arg: script, selectionStdin: false };
  }
  if (part.startsWith(".!sh ") || part.startsWith(".! sh ") || part.startsWith(".!sh\t") || part.startsWith(".! sh\t")) {
    const raw = part.startsWith(".! sh ") || part.startsWith(".! sh\t") ? part.slice(6) : part.slice(5);
    const script = shScript(raw);
    if (script.length === 0) {
      return null;
    }
    return { kind: "sh", arg: script, selectionStdin: true };
  }
  return null;
}

function peelClip(part: string): { part: string; clip: boolean } {
  const match = /^(.*?)\s*>\s*clip\s*$/i.exec(part);
  if (!match) {
    return { part, clip: false };
  }
  return { part: match[1].trim(), clip: true };
}

/** 流れができたあとの `clip` は読む段として不正。 */
function shAfterEach(ops: PipeOp[]): boolean {
  let seen = false;
  for (const op of ops) {
    if (op.kind === "each") {
      seen = true;
    }
    if (seen && op.kind === "sh") {
      return true;
    }
  }
  return false;
}

function middleClip(ops: PipeOp[]): boolean {
  let produced = false;
  for (const op of ops) {
    if (op.kind === "clip") {
      if (produced) {
        return true;
      }
      produced = true;
      continue;
    }
    if (op.kind === "raw") {
      continue;
    }
    produced = true;
  }
  return false;
}

function pipeUsesSelection(ops: PipeOp[]): boolean {
  let produced = false;
  for (const op of ops) {
    if (op.kind === "raw") {
      continue;
    }
    if (op.kind === "clip" || op.kind === "sel") {
      produced = true;
      continue;
    }
    if (op.kind === "dot") {
      if (!produced) {
        return true;
      }
      produced = true;
      continue;
    }
    if (op.kind === "sh") {
      if (!produced && op.selectionStdin) {
        return true;
      }
      produced = true;
      continue;
    }
    if (op.kind === "echo") {
      produced = true;
      continue;
    }
    if (!produced) {
      return true;
    }
  }
  return !produced;
}

/** 段が1つでもパイプ。未知の1語は none。段が空や未知なら bad。 */
export function parseColonPipe(input: string): ParsedPipe {
  const line = input.trim().replace(/^:/, "");
  const parts = splitUnquotedPipe(line);
  if (parts.length < 2) {
    const only = parts[0] ?? "";
    if (only.length === 0) {
      return { kind: "none" };
    }
    if (peelClip(only).clip) {
      return { kind: "bad" };
    }
    const stage = stageOf(only);
    if (!stage) {
      return { kind: "none" };
    }
    return { kind: "ok", ops: [stage], sink: { kind: "paste" }, usesSelection: pipeUsesSelection([stage]) };
  }
  if (parts.some((part) => part.length === 0)) {
    return { kind: "bad" };
  }
  let sink: PipeSink = { kind: "paste" };
  let body = parts;
  const peeled = peelClip(parts[parts.length - 1]);
  if (peeled.clip) {
    return { kind: "bad" };
  } else {
    const last = sinkOf(parts[parts.length - 1]);
    if (last) {
      sink = last;
      body = parts.slice(0, -1);
    }
  }
  if (body.length === 0) {
    return { kind: "bad" };
  }
  const ops: PipeOp[] = [];
  for (const part of body) {
    const stage = stageOf(part);
    if (!stage) {
      return { kind: "bad" };
    }
    ops.push(stage);
  }
  if (middleClip(ops) || shAfterEach(ops)) {
    return { kind: "bad" };
  }
  return { kind: "ok", ops, sink, usesSelection: pipeUsesSelection(ops) };
}

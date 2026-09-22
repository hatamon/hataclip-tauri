export const COLON_COMMANDS = [
  "help",
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
  "echo",
  "s",
  "@",
  "dedup",
  "sort",
  "map",
  "unmap",
  "mapleader",
  "set",
  "n",
  "settings",
  "tags",
  "clip",
  "camel",
  "pascal",
  "snake",
  "kebab",
  "upper",
  "lower",
  "json",
  "xml",
];

/** 末尾の `> clip` を行き先として外す。 */
export function stripClipSink(line: string): { cmd: string; clip: boolean } {
  const trimmed = line.trim().replace(/^:/, "");
  const match = /^(.*?)\s*>\s*clip\s*$/i.exec(trimmed);
  if (match) {
    return { cmd: match[1].trim(), clip: true };
  }
  return { cmd: trimmed, clip: false };
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
  if (command === "export" || command === "import" || command === "sh" || command === "help" || command === "echo") {
    return `${command} `;
  }
  if (command === "map" || command === "unmap" || command === "mapleader" || command === "set" || command === "n" || command === "quote" || command === "join") {
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

/** `:quote` の行頭。未指定なら `> `。`:quote "* "` で空白も含めて指定。 */
export function quotePrefix(line: string): string | null {
  if (line === "quote") {
    return "> ";
  }
  if (line.startsWith("quote ") || line.startsWith("quote\t")) {
    return colonArg(line.slice(6), "> ");
  }
  return null;
}

/** `:join` の区切り。未指定なら `,`。`:join "\\t"` はタブ。`:comma` / `:tab` は別名。 */
export function joinSeparator(line: string): string | null {
  if (line === "comma") {
    return ",";
  }
  if (line === "tab") {
    return "\t";
  }
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
    | "echo";
  arg: string;
  selectionStdin: boolean;
};

export type PipeSink =
  | { kind: "paste" }
  | { kind: "clip" }
  | { kind: "add" }
  | { kind: "set"; name: string }
  | { kind: "open" }
  | { kind: "show" };

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
  const name = varSinkName(part);
  if (name) {
    return { kind: "set", name };
  }
  return null;
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
  if (part === "raw") {
    return { kind: "raw", arg: "", selectionStdin: false };
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
  const prefix = quotePrefix(part);
  if (prefix !== null) {
    return { kind: "quote", arg: prefix, selectionStdin: false };
  }
  const sep = joinSeparator(part);
  if (sep !== null) {
    return { kind: "join", arg: sep, selectionStdin: false };
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

function pipeUsesSelection(ops: PipeOp[]): boolean {
  let produced = false;
  for (const op of ops) {
    if (op.kind === "raw") {
      continue;
    }
    if (op.kind === "clip") {
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

/** トップレベルの `|` が無ければ none。段が空や未知なら bad。 */
export function parseColonPipe(input: string): ParsedPipe {
  const line = input.trim().replace(/^:/, "");
  const parts = splitUnquotedPipe(line);
  if (parts.length < 2) {
    return { kind: "none" };
  }
  if (parts.some((part) => part.length === 0)) {
    return { kind: "bad" };
  }
  let sink: PipeSink = { kind: "paste" };
  let body = parts;
  const peeled = peelClip(parts[parts.length - 1]);
  if (peeled.clip) {
    sink = { kind: "clip" };
    body = peeled.part.length === 0 ? parts.slice(0, -1) : [...parts.slice(0, -1), peeled.part];
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
  return { kind: "ok", ops, sink, usesSelection: pipeUsesSelection(ops) };
}

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

export function colonCommandToken(input: string): string {
  const line = stripClipSink(input).cmd;
  if (line.startsWith("help")) {
    return "help";
  }
  if (line.startsWith("!!")) {
    return "!!";
  }
  return line.split(/\s/)[0] ?? "";
}

export function matchingColonCommands(input: string): string[] {
  const line = stripClipSink(input).cmd;
  if (line.startsWith("help ") || line === "help") {
    return [];
  }
  const token = line.startsWith("!!") ? "!!" : (line.split(/[\s/]/)[0] ?? "");
  return COLON_COMMANDS.filter((name) => name.startsWith(token));
}

export function applyColonCompletion(input: string, command: string): string {
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

/** `:quote` の行頭。未指定なら `> `。`:quote "* "` で空白も含めて指定。`:bullet` は `"* "`。 */
export function quotePrefix(line: string): string | null {
  if (line === "bullet") {
    return "* ";
  }
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

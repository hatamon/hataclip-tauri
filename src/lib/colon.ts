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
  "comma",
  "tab",
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
  if (command === "map" || command === "unmap" || command === "mapleader" || command === "set" || command === "n" || command === "quote") {
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

/** `:quote` の行頭。未指定なら `> `。`:bullet` は `* `。 */
export function quotePrefix(line: string): string | null {
  if (line === "bullet") {
    return "* ";
  }
  if (line === "quote") {
    return "> ";
  }
  if (line.startsWith("quote ") || line.startsWith("quote\t")) {
    const rest = line.slice(6);
    return rest.trim().length === 0 ? "> " : rest;
  }
  return null;
}

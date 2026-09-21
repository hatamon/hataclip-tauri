export const COLON_COMMANDS = [
  "help",
  "sh",
  "export",
  "import",
  "clear",
  "quote",
  "bullet",
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
  return line.split(/\s/)[0] ?? "";
}

export function matchingColonCommands(input: string): string[] {
  const line = stripClipSink(input).cmd;
  if (line.startsWith("help ") || line === "help") {
    return [];
  }
  const token = line.split(/[\s/]/)[0] ?? "";
  return COLON_COMMANDS.filter((name) => name.startsWith(token));
}

export function applyColonCompletion(input: string, command: string): string {
  if (command === "s") {
    return "s/";
  }
  if (command === "export" || command === "import" || command === "sh" || command === "help" || command === "echo") {
    return `${command} `;
  }
  if (command === "map" || command === "unmap" || command === "mapleader" || command === "set" || command === "n") {
    return `${command} `;
  }
  return command;
}

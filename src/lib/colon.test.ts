import { describe, expect, it } from "vitest";
import { applyColonCompletion, bangFilterScript, joinSeparator, matchingColonCommands, parseColonPipe, quotePrefix, unquoteColonArg, stripClipSink } from "./colon";

describe("matchingColonCommands", () => {
  it("filters by prefix and skips help topics", () => {
    expect(matchingColonCommands("ex")).toEqual(["export"]);
    expect(matchingColonCommands("")).toEqual([
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
    ]);
    expect(matchingColonCommands("help ")).toEqual([]);
    expect(matchingColonCommands("!")).toEqual(["!!"]);
  });

  it("ignores a trailing clip sink when matching", () => {
    expect(matchingColonCommands("quote > clip")).toEqual(["quote"]);
  });

  it("completes the stage after a pipe", () => {
    expect(matchingColonCommands("sh dir | qu")).toEqual(["quote"]);
    expect(matchingColonCommands("sh dir | c")).toEqual(["clear", "clip"]);
  });
});

describe("stripClipSink", () => {
  it("splits > clip from the command", () => {
    expect(stripClipSink("quote > clip")).toEqual({ cmd: "quote", clip: true });
    expect(stripClipSink('quote "* " > clip')).toEqual({ cmd: 'quote "* "', clip: true });
    expect(stripClipSink(":sh dir>clip")).toEqual({ cmd: "sh dir", clip: true });
    expect(stripClipSink("echo 2+3")).toEqual({ cmd: "echo 2+3", clip: false });
  });
});

describe("applyColonCompletion", () => {
  it("adds a trailing slash for substitute", () => {
    expect(applyColonCompletion("", "s")).toBe("s/");
    expect(applyColonCompletion("", "quote")).toBe("quote ");
    expect(applyColonCompletion("sh dir | qu", "quote")).toBe("sh dir | quote ");
    expect(applyColonCompletion("", "join")).toBe("join ");
    expect(applyColonCompletion("", "!!")).toBe("!!sh ");
  });
});

describe("bangFilterScript", () => {
  it("reads !!sh and !! sh", () => {
    expect(bangFilterScript("!!sh jq .")).toBe("jq .");
    expect(bangFilterScript("!! sh sort")).toBe("sort");
    expect(bangFilterScript("!!sh")).toBeNull();
    expect(bangFilterScript(".!sh jq .")).toBeNull();
  });
});

describe("quotePrefix", () => {
  it("defaults to >  and keeps quoted spaces", () => {
    expect(quotePrefix("quote")).toBe("> ");
    expect(quotePrefix("quote ")).toBe("> ");
    expect(quotePrefix('quote "* "')).toBe("* ");
    expect(quotePrefix('quote ">"')).toBe(">");
    expect(quotePrefix("quote - [ ] ")).toBe("- [ ]");
    expect(quotePrefix(stripClipSink('quote "* " > clip').cmd)).toBe("* ");
    expect(quotePrefix("bullet")).toBe("* ");
    expect(quotePrefix("format")).toBeNull();
  });
});

describe("joinSeparator", () => {
  it("defaults to comma and reads quoted separators", () => {
    expect(joinSeparator("join")).toBe(",");
    expect(joinSeparator('join "\\t"')).toBe("\t");
    expect(joinSeparator('join ", "')).toBe(", ");
    expect(joinSeparator("comma")).toBe(",");
    expect(joinSeparator("tab")).toBe("\t");
    expect(joinSeparator("quote")).toBeNull();
  });
});

describe("parseColonPipe", () => {
  it("leaves a single command alone", () => {
    expect(parseColonPipe("quote > clip")).toEqual({ kind: "none" });
    expect(parseColonPipe('quote "|"')).toEqual({ kind: "none" });
  });

  it("chains sh into quote and a clip sink", () => {
    expect(parseColonPipe('sh dir | sh sort | quote "x " | clip')).toEqual({
      kind: "ok",
      sink: { kind: "clip" },
      usesSelection: false,
      ops: [
        { kind: "sh", arg: "dir", selectionStdin: false },
        { kind: "sh", arg: "sort", selectionStdin: false },
        { kind: "quote", arg: "x ", selectionStdin: false },
      ],
    });
  });

  it("reads the selection for quote and dot-sh", () => {
    expect(parseColonPipe('quote "> " | quote "x " | add')).toMatchObject({
      kind: "ok",
      sink: { kind: "add" },
      usesSelection: true,
    });
    expect(parseColonPipe(".!sh sort | set files")).toMatchObject({
      kind: "ok",
      sink: { kind: "set", name: "files" },
      usesSelection: true,
    });
  });

  it("keeps a quoted pipe and accepts a trailing > clip", () => {
    expect(parseColonPipe('sh "dir | sort" | show')).toMatchObject({
      kind: "ok",
      sink: { kind: "show" },
      ops: [{ kind: "sh", arg: "dir | sort", selectionStdin: false }],
    });
    expect(parseColonPipe('sh dir | quote "> " > clip')).toMatchObject({
      kind: "ok",
      sink: { kind: "clip" },
      ops: [
        { kind: "sh", arg: "dir", selectionStdin: false },
        { kind: "quote", arg: "> ", selectionStdin: false },
      ],
    });
  });

  it("reads the selection or the clipboard as a source", () => {
    expect(parseColonPipe(". | sh sort | clip")).toMatchObject({
      kind: "ok",
      sink: { kind: "clip" },
      usesSelection: true,
      ops: [
        { kind: "dot", arg: "", selectionStdin: false },
        { kind: "sh", arg: "sort", selectionStdin: false },
      ],
    });
    expect(parseColonPipe("clip | show")).toMatchObject({
      kind: "ok",
      sink: { kind: "show" },
      usesSelection: false,
      ops: [{ kind: "clip", arg: "", selectionStdin: false }],
    });
  });

  it("rejects an empty stage", () => {
    expect(parseColonPipe("sh dir | | clip")).toEqual({ kind: "bad" });
    expect(parseColonPipe("| clip")).toEqual({ kind: "bad" });
  });
});

describe("unquoteColonArg", () => {
  it("keeps spaces and escapes inside quotes", () => {
    expect(unquoteColonArg('" * "')).toBe(" * ");
    expect(unquoteColonArg("'\\t'")).toBe("\t");
    expect(unquoteColonArg('"* " extra')).toBeNull();
    expect(unquoteColonArg("* ")).toBeNull();
  });
});

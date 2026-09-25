import { describe, expect, it } from "vitest";
import { applyColonCompletion, bangFilterScript, joinSeparator, keepVisibleIndex, matchingColonCommands, parseColonPipe, quotePrefix, unquoteColonArg, stripClipSink } from "./colon";

describe("keepVisibleIndex", () => {
  it("keeps the current row, or the start of a visual range while colon is open", () => {
    expect(keepVisibleIndex(4, null, false)).toBe(4);
    expect(keepVisibleIndex(4, 2, false)).toBe(4);
    expect(keepVisibleIndex(4, null, true)).toBe(4);
    expect(keepVisibleIndex(4, 2, true)).toBe(2);
    expect(keepVisibleIndex(2, 4, true)).toBe(2);
  });
});

describe("matchingColonCommands", () => {
  it("filters by prefix and skips help topics", () => {
    expect(matchingColonCommands("ex")).toEqual(["export"]);
    expect(matchingColonCommands("")).toEqual([
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
    ]);
    expect(matchingColonCommands("help ")).toEqual([]);
    expect(matchingColonCommands("!")).toEqual(["!!"]);
  });

  it("ignores a trailing clip sink when matching", () => {
    expect(matchingColonCommands("quote > clip")).toEqual(["quote"]);
  });

  it("completes the stage after a pipe", () => {
    expect(matchingColonCommands("sh dir | qu")).toEqual(["quote"]);
    expect(matchingColonCommands("sh dir | c")).toEqual(["clear", "clip", "camel", "col", "crypt"]);
  });
});

describe("stripClipSink", () => {
  it("does not treat > clip as a sink", () => {
    expect(stripClipSink("quote > clip")).toEqual({ cmd: "quote > clip", clip: false });
    expect(stripClipSink('quote "* " > clip')).toEqual({ cmd: 'quote "* " > clip', clip: false });
    expect(stripClipSink(":sh dir>clip")).toEqual({ cmd: "sh dir>clip", clip: false });
    expect(stripClipSink("echo 2+3")).toEqual({ cmd: "echo 2+3", clip: false });
  });
});

describe("applyColonCompletion", () => {
  it("adds a trailing slash for substitute", () => {
    expect(applyColonCompletion("", "s")).toBe("s/");
    expect(applyColonCompletion("", "quote")).toBe("quote ");
    expect(applyColonCompletion("sh dir | qu", "quote")).toBe("sh dir | quote ");
    expect(applyColonCompletion("", "join")).toBe("join ");
    expect(applyColonCompletion("", "crypt")).toBe("crypt ");
    expect(applyColonCompletion("", "decrypt")).toBe("decrypt ");
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
    expect(quotePrefix("format")).toBeNull();
    expect(quotePrefix('quote "> " "<"')).toBe("> ");
    expect(quotePrefix('quote "> " <')).toBeNull();
  });
});

describe("joinSeparator", () => {
  it("defaults to comma and reads quoted separators", () => {
    expect(joinSeparator("join")).toBe(",");
    expect(joinSeparator('join "\\t"')).toBe("\t");
    expect(joinSeparator('join ", "')).toBe(", ");
    expect(joinSeparator("comma")).toBeNull();
    expect(joinSeparator("tab")).toBeNull();
    expect(joinSeparator("quote")).toBeNull();
  });
});

describe("parseColonPipe", () => {
  it("leaves a command that is not a stage alone", () => {
    expect(parseColonPipe("quote > clip")).toEqual({ kind: "bad" });
    expect(parseColonPipe('quote "* " > clip')).toEqual({ kind: "bad" });
    expect(parseColonPipe("help")).toEqual({ kind: "none" });
    expect(parseColonPipe("echo")).toEqual({ kind: "none" });
  });

  it("runs one stage without a bar", () => {
    expect(parseColonPipe("upper")).toEqual({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: true,
      ops: [{ kind: "upper", arg: "", selectionStdin: false }],
    });
    expect(parseColonPipe(":json")).toMatchObject({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: true,
    });
    expect(parseColonPipe("crypt secret")).toEqual({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: true,
      ops: [{ kind: "crypt", arg: "secret", selectionStdin: false }],
    });
    expect(parseColonPipe("crypt")).toEqual({ kind: "none" });
    expect(parseColonPipe("sel | crypt secret | clip")).toEqual({
      kind: "ok",
      sink: { kind: "clip" },
      usesSelection: false,
      ops: [
        { kind: "sel", arg: "", selectionStdin: false },
        { kind: "crypt", arg: "secret", selectionStdin: false },
      ],
    });
    expect(parseColonPipe("echo 3+4")).toEqual({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: false,
      ops: [{ kind: "echo", arg: "3+4", selectionStdin: false }],
    });
    expect(parseColonPipe('quote "> " "<"')).toMatchObject({
      kind: "ok",
      sink: { kind: "paste" },
      ops: [{ kind: "quote", arg: "> \u0001<" }],
    });
    expect(parseColonPipe('quote "|"')).toMatchObject({
      kind: "ok",
      sink: { kind: "paste" },
      ops: [{ kind: "quote", arg: "|" }],
    });
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

  it("keeps a quoted pipe and rejects a trailing > clip", () => {
    expect(parseColonPipe('sh "dir | sort" | show')).toMatchObject({
      kind: "ok",
      sink: { kind: "show" },
      ops: [{ kind: "sh", arg: "dir | sort", selectionStdin: false }],
    });
    expect(parseColonPipe('sh dir | quote "> " > clip')).toEqual({ kind: "bad" });
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
    expect(parseColonPipe(". | snake | clip")).toMatchObject({
      kind: "ok",
      sink: { kind: "clip" },
      usesSelection: true,
      ops: [
        { kind: "dot", arg: "", selectionStdin: false },
        { kind: "snake", arg: "", selectionStdin: false },
      ],
    });
    expect(parseColonPipe("clip | json | show")).toMatchObject({
      kind: "ok",
      sink: { kind: "show" },
      usesSelection: false,
      ops: [
        { kind: "clip", arg: "", selectionStdin: false },
        { kind: "json", arg: "", selectionStdin: false },
      ],
    });
    expect(parseColonPipe("echo 3+4|log")).toEqual({
      kind: "ok",
      sink: { kind: "log", path: "" },
      usesSelection: false,
      ops: [{ kind: "echo", arg: "3+4", selectionStdin: false }],
    });
    expect(parseColonPipe("echo 3+4 | log notes.log")).toMatchObject({
      sink: { kind: "log", path: "notes.log" },
    });
    expect(parseColonPipe("echo 3+4|show")).toEqual({
      kind: "ok",
      sink: { kind: "show" },
      usesSelection: false,
      ops: [{ kind: "echo", arg: "3+4", selectionStdin: false }],
    });
    expect(parseColonPipe("echo | show")).toEqual({ kind: "bad" });
    expect(parseColonPipe("clip | show")).toMatchObject({
      kind: "ok",
      sink: { kind: "show" },
      usesSelection: false,
      ops: [{ kind: "clip", arg: "", selectionStdin: false }],
    });
    expect(parseColonPipe("show | add")).toMatchObject({
      kind: "ok",
      sink: { kind: "add" },
      ops: [{ kind: "show", arg: "", selectionStdin: false }],
    });
    expect(parseColonPipe("s/old/new")).toMatchObject({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: true,
      ops: [{ kind: "sub", arg: "old\u0001new" }],
    });
    expect(parseColonPipe("s//new")).toEqual({ kind: "none" });
    expect(parseColonPipe("sel | s/old/new/tail")).toMatchObject({
      kind: "ok",
      ops: [{ kind: "sel" }, { kind: "sub", arg: "old\u0001new/tail" }],
    });
    expect(parseColonPipe("sel | upper")).toEqual({
      kind: "ok",
      sink: { kind: "paste" },
      usesSelection: false,
      ops: [
        { kind: "sel", arg: "", selectionStdin: false },
        { kind: "upper", arg: "", selectionStdin: false },
      ],
    });
    expect(parseColonPipe("sel | clip | upper")).toEqual({ kind: "bad" });
    expect(parseColonPipe("echo 1 | clip | show")).toEqual({ kind: "bad" });
  });

  it("rejects an empty stage", () => {
    expect(parseColonPipe("sh dir | | clip")).toEqual({ kind: "bad" });
    expect(parseColonPipe("| clip")).toEqual({ kind: "bad" });
    expect(parseColonPipe('sel | filter ".txt"')).toMatchObject({
      kind: "ok",
      ops: [
        { kind: "sel", arg: "", selectionStdin: false },
        { kind: "filter", arg: ".txt", selectionStdin: false },
      ],
    });
    expect(parseColonPipe('filter not ".txt"')).toMatchObject({
      ops: [{ kind: "filter", arg: "\u0001.txt" }],
    });
    expect(parseColonPipe('filter "not"')).toMatchObject({
      ops: [{ kind: "filter", arg: "not" }],
    });
    expect(parseColonPipe("filter")).toEqual({ kind: "none" });
    expect(parseColonPipe("filter not")).toEqual({ kind: "none" });
    expect(parseColonPipe('sel | filter ""')).toEqual({ kind: "bad" });
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

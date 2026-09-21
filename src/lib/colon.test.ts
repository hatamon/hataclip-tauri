import { describe, expect, it } from "vitest";
import { applyColonCompletion, bangFilterScript, joinSeparator, matchingColonCommands, quotePrefix, unquoteColonArg, stripClipSink } from "./colon";

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
    ]);
    expect(matchingColonCommands("help ")).toEqual([]);
    expect(matchingColonCommands("!")).toEqual(["!!"]);
  });

  it("ignores a trailing clip sink when matching", () => {
    expect(matchingColonCommands("quote > clip")).toEqual(["quote"]);
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

describe("unquoteColonArg", () => {
  it("keeps spaces and escapes inside quotes", () => {
    expect(unquoteColonArg('" * "')).toBe(" * ");
    expect(unquoteColonArg("'\\t'")).toBe("\t");
    expect(unquoteColonArg('"* " extra')).toBeNull();
    expect(unquoteColonArg("* ")).toBeNull();
  });
});

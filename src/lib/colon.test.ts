import { describe, expect, it } from "vitest";
import { applyColonCompletion, matchingColonCommands, stripClipSink } from "./colon";

describe("matchingColonCommands", () => {
  it("filters by prefix and skips help topics", () => {
    expect(matchingColonCommands("ex")).toEqual(["export"]);
    expect(matchingColonCommands("")).toEqual([
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
    ]);
    expect(matchingColonCommands("help ")).toEqual([]);
  });

  it("ignores a trailing clip sink when matching", () => {
    expect(matchingColonCommands("quote > clip")).toEqual(["quote"]);
  });
});

describe("stripClipSink", () => {
  it("splits > clip from the command", () => {
    expect(stripClipSink("quote > clip")).toEqual({ cmd: "quote", clip: true });
    expect(stripClipSink(":sh dir>clip")).toEqual({ cmd: "sh dir", clip: true });
    expect(stripClipSink("echo 2+3")).toEqual({ cmd: "echo 2+3", clip: false });
  });
});

describe("applyColonCompletion", () => {
  it("adds a trailing slash for substitute", () => {
    expect(applyColonCompletion("", "s")).toBe("s/");
    expect(applyColonCompletion("", "quote")).toBe("quote");
  });
});

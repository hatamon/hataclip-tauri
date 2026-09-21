import { describe, expect, it } from "vitest";
import { applyColonCompletion, matchingColonCommands } from "./colon";

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
      "s",
      "@",
      "dedup",
    ]);
    expect(matchingColonCommands("help ")).toEqual([]);
  });
});

describe("applyColonCompletion", () => {
  it("adds a trailing slash for substitute", () => {
    expect(applyColonCompletion("", "s")).toBe("s/");
    expect(applyColonCompletion("", "quote")).toBe("quote");
  });
});

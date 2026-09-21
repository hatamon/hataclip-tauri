import { describe, expect, it } from "vitest";
import { askNames, uniqueAskNames } from "./ask";

describe("askNames", () => {
  it("keeps first-seen order and drops duplicates", () => {
    expect(askNames("{{ask:a}} {{ask:b}} {{ask:a}}")).toEqual(["a", "b"]);
    expect(askNames("{{ask a}} {{ask:b}}")).toEqual(["a", "b"]);
    expect(askNames("{{ ask: 名前 }}")).toEqual(["名前"]);
    expect(askNames("{{clip}}")).toEqual([]);
  });
});

describe("uniqueAskNames", () => {
  it("walks selected rows in order", () => {
    expect(
      uniqueAskNames([{ text: "{{ask:x}} y" }, { text: "{{ask:y}} {{ask:x}}" }]),
    ).toEqual(["x", "y"]);
  });
});

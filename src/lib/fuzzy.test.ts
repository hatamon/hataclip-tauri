import { describe, expect, it } from "vitest";
import { fuzzyFilter, fuzzyScore } from "./fuzzy";

describe("fuzzyScore", () => {
  it("matches subsequence and rejects missing chars", () => {
    expect(fuzzyScore("東", "東京")).not.toBeNull();
    expect(fuzzyScore("tokyo", "東京")).toBeNull();
  });

  it("empty query matches everything", () => {
    expect(fuzzyScore("", "anything")).toBe(0);
  });

  it("ignores text after the first 4096 characters", () => {
    const text = `${"a".repeat(4096)}z`;
    expect(fuzzyScore("z", text)).toBeNull();
    expect(fuzzyScore("a", text)).not.toBeNull();
  });
});

describe("fuzzyFilter", () => {
  it("keeps matching items", () => {
    const items = [{ text: "東京タワー" }, { text: "hello" }];
    expect(fuzzyFilter("東", items).map((item) => item.text)).toEqual(["東京タワー"]);
  });
});

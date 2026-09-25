import { describe, expect, it } from "vitest";
import { isLocked, isSecret, matchesAlias, tagWords, tagsByCount } from "./tags";

describe("tagWords", () => {
  it("splits on whitespace and drops empties", () => {
    expect(tagWords("  a b  c a ")).toEqual(["a", "b", "c"]);
    expect(tagWords("   ")).toEqual([]);
  });
});

describe("tagsByCount", () => {
  it("orders by frequency then name", () => {
    expect(
      tagsByCount([
        { tags: ["b", "a"] },
        { tags: ["a"] },
        { tags: ["a"] },
      ]),
    ).toEqual(["a", "b"]);
  });
});

describe("secret and alias", () => {
  it("detects secret and alias tags", () => {
    expect(isSecret({ tags: ["secret"] })).toBe(true);
    expect(isLocked({ tags: ["lock"] })).toBe(true);
    expect(isLocked({ tags: ["secret"] })).toBe(false);
    expect(matchesAlias({ tags: ["alias:foo"] }, "foo")).toBe(true);
    expect(matchesAlias({ tags: ["alias:foo"] }, "bar")).toBe(false);
    expect(matchesAlias({ tags: ["alias:x", "alias:y"] }, "x")).toBe(true);
    expect(matchesAlias({ tags: ["alias:x", "alias:y"] }, "y")).toBe(true);
    expect(matchesAlias({ tags: ["alias:x", "alias:y"] }, "x y")).toBe(true);
    expect(matchesAlias({ tags: ["alias:x", "alias:y"] }, "y x")).toBe(true);
    expect(matchesAlias({ tags: ["alias:x", "alias:y"] }, "x z")).toBe(false);
  });
});

import { describe, expect, it } from "vitest";
import { isLocked, isSecret, matchesAlias, tagsByCount } from "./tags";

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
  });
});

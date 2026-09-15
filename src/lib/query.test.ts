import { describe, expect, it } from "vitest";
import { parseQuery } from "./query";

describe("parseQuery", () => {
  it("extracts tags from anywhere and keeps remaining text", () => {
    expect(parseQuery("#work hello #dev world")).toEqual({
      tags: ["work", "dev"],
      text: "hello world",
    });
  });

  it("treats a lone hash-word as tag-only", () => {
    expect(parseQuery("#inbox")).toEqual({ tags: ["inbox"], text: "" });
  });

  it("returns empty tags when there is no hash", () => {
    expect(parseQuery("東京")).toEqual({ tags: [], text: "東京" });
  });
});

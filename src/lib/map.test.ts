import { describe, expect, it } from "vitest";
import {
  combinePending,
  formatMaps,
  matchMap,
  parseMapArgs,
  parseMapleaderArgs,
  parseRhs,
  parseUnmapArgs,
  upsertMap,
  validLhs,
  validLeader,
  whichKeysForMaps,
} from "./map";

describe("parseMapArgs", () => {
  it("lists when empty and sets nvim-style leader cmd", () => {
    expect(parseMapArgs("")).toEqual({ kind: "list" });
    expect(parseMapArgs("<leader>* :bullet")).toEqual({
      kind: "set",
      lhs: "<leader>*",
      rhs: ":bullet",
    });
    expect(parseMapArgs("j k")).toEqual({ kind: "set", lhs: "j", rhs: "k" });
    expect(parseMapArgs("gT :quote")).toEqual({ kind: "set", lhs: "gT", rhs: ":quote" });
    expect(parseMapArgs("1 dd")).toBeNull();
    expect(parseMapArgs("j")).toBeNull();
  });
});

describe("parseRhs", () => {
  it("treats :quote and <cmd>bullet as commands", () => {
    expect(parseRhs(":bullet")).toEqual({ kind: "cmd", command: "bullet" });
    expect(parseRhs("<cmd>bullet")).toEqual({ kind: "cmd", command: "bullet" });
    expect(parseRhs("dd")).toEqual({ kind: "keys", keys: "dd" });
  });
});

describe("validLhs and leader", () => {
  it("allows leader plus one key and rejects digits", () => {
    expect(validLhs("<leader>*")).toBe(true);
    expect(validLhs("<leader>dd")).toBe(false);
    expect(validLhs("dd")).toBe(true);
    expect(validLhs("1")).toBe(false);
    expect(validLeader(" ")).toBe(true);
    expect(parseMapleaderArgs(" ")).toEqual({ kind: "set", leader: " " });
    expect(parseMapleaderArgs("<Space>")).toEqual({ kind: "set", leader: " " });
    expect(validLeader(",")).toBe(true);
    expect(validLeader("1")).toBe(false);
    expect(parseMapleaderArgs("")).toEqual({ kind: "show" });
    expect(parseMapleaderArgs(",")).toEqual({ kind: "set", leader: "," });
    expect(parseUnmapArgs("<leader>*")).toBe("<leader>*");
  });
});

describe("matchMap", () => {
  it("waits on prefixes and does not recurse", () => {
    const maps = upsertMap([], "<leader>*", "<cmd>bullet");
    expect(matchMap(maps, "", "<Space>", " ")).toEqual({ kind: "prefix" });
    expect(matchMap(maps, "<leader>", "*", " ")).toEqual({
      kind: "hit",
      rhs: { kind: "cmd", command: "bullet" },
    });
    expect(combinePending("", "<Space>", " ")).toBe("<leader>");
    expect(matchMap([{ lhs: "j", rhs: "k" }], "", "j", "\\")).toEqual({
      kind: "hit",
      rhs: { kind: "keys", keys: "k" },
    });
    expect(matchMap([{ lhs: "dd", rhs: "k" }], "", "d", "\\")).toEqual({ kind: "prefix" });
    expect(formatMaps("\\", maps)).toContain("<leader>*");
    expect(whichKeysForMaps(maps, "")).toEqual([]);
    expect(whichKeysForMaps(maps, "<leader>")).toEqual([{ key: "*", label: "<cmd>bullet" }]);
  });
});

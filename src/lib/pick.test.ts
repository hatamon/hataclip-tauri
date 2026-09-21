import { describe, expect, it } from "vitest";
import { pickSpecs, uniquePickSpecs } from "./pick";

describe("pickSpecs", () => {
  it("parses options and drops duplicates", () => {
    expect(pickSpecs("{{pick: prod, stg}} {{pick: prod, stg}}")).toEqual([
      { spec: "prod, stg", options: ["prod", "stg"] },
    ]);
    expect(pickSpecs("{{pick prod, stg}}")).toEqual([
      { spec: "prod, stg", options: ["prod", "stg"] },
    ]);
    expect(pickSpecs("{{clip}}")).toEqual([]);
  });
});

describe("uniquePickSpecs", () => {
  it("walks rows in order", () => {
    expect(
      uniquePickSpecs([{ text: "{{pick:a,b}}" }, { text: "{{pick:c}}" }]).map(
        (entry) => entry.spec,
      ),
    ).toEqual(["a,b", "c"]);
  });
});

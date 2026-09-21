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
    expect(pickSpecs("{{pick hata007@x, {{var:a}}}}")).toEqual([
      { spec: "hata007@x, {{var:a}}", options: ["hata007@x", "{{var:a}}"] },
    ]);
    expect(
      pickSpecs("{{pick tag:env}}", [
        { text: "stg", tags: ["env"] },
        { text: "prod", tags: ["env"] },
        { text: "stg", tags: ["env"] },
        { text: "other", tags: ["work"] },
      ]),
    ).toEqual([{ spec: "tag:env", options: ["stg", "prod"] }]);
    expect(pickSpecs("{{pick tag:missing}}", [{ text: "x", tags: ["env"] }])).toEqual([]);
  });
});

describe("uniquePickSpecs", () => {
  it("walks rows in order", () => {
    expect(
      uniquePickSpecs([{ text: "{{pick:a,b}}" }, { text: "{{pick:c}}" }]).map(
        (entry) => entry.spec,
      ),
    ).toEqual(["a,b", "c"]);
    expect(
      uniquePickSpecs(
        [{ text: "{{pick tag:env}}" }],
        [
          { text: "stg", tags: ["env"] },
          { text: "prod", tags: ["env"] },
        ],
      ),
    ).toEqual([{ spec: "tag:env", options: ["stg", "prod"] }]);
  });
});

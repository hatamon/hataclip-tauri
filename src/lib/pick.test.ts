import { describe, expect, it } from "vitest";
import { pickByDigit, pickSpecs, uniquePickSpecs } from "./pick";

describe("pickSpecs", () => {
  it("parses list, tag, and search", () => {
    expect(pickSpecs("{{pick: prod, stg}}")).toEqual([]);
    expect(pickSpecs("{{pick list: prod, stg}} {{pick list: prod, stg}}")).toEqual([
      { spec: "list: prod, stg", options: ["prod", "stg"] },
    ]);
    expect(pickSpecs("{{clip}}")).toEqual([]);
    expect(pickSpecs("{{pick list: hata007@x, {{var:a}}}}")).toEqual([
      { spec: "list: hata007@x, {{var:a}}", options: ["hata007@x", "{{var:a}}"] },
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
    expect(pickSpecs('{{pick search: xx}}')).toEqual([]);
    expect(
      pickSpecs('{{pick search: "hata"}}', [
        { id: "self", text: "hata007@corp.jp", tags: [] },
        { id: "other", text: "hata007-admin", tags: [] },
        { id: "nope", text: "other", tags: [] },
      ], "", "self"),
    ).toEqual([{ spec: 'search: "hata"', options: ["hata007-admin"] }]);
  });
});

describe("uniquePickSpecs", () => {
  it("walks rows in order", () => {
    expect(
      uniquePickSpecs([{ text: "{{pick list: a, b}}" }, { text: "{{pick list: c}}" }]).map(
        (entry) => entry.spec,
      ),
    ).toEqual(["list: a, b", "list: c"]);
    expect(
      uniquePickSpecs(
        [{ id: "row", text: "{{pick tag:env}}" }],
        [
          { text: "stg", tags: ["env"] },
          { text: "prod", tags: ["env"] },
        ],
      ),
    ).toEqual([{ spec: "tag:env", options: ["stg", "prod"] }]);
  });
});

describe("pickByDigit", () => {
  it("confirms 1 through the option count when there are at most nine", () => {
    expect(pickByDigit(3, "1")).toBe(0);
    expect(pickByDigit(3, "3")).toBe(2);
    expect(pickByDigit(3, "4")).toBeNull();
    expect(pickByDigit(3, "j")).toBeNull();
    expect(pickByDigit(10, "1")).toBeNull();
    expect(pickByDigit(0, "1")).toBeNull();
  });
});

import { describe, expect, it } from "vitest";
import { applyWhen, contextApp } from "./when";

describe("applyWhen", () => {
  it("keeps the prefix and the matching section", () => {
    expect(applyWhen("id{{when app: chrome}}c{{when app: code}}k{{when}}x", "code")).toBe("idk");
    expect(applyWhen("{{when app: chrome}}c{{when app: excel}}e{{when}}d", "code")).toBe("d");
    expect(applyWhen("{{when app: chrome, msedge}}hit{{when}}miss", "EXCEL")).toBe("miss");
    expect(applyWhen("{{when app: chrome, msedge}}hit{{when}}miss", "msedge")).toBe("hit");
    expect(applyWhen("{{when chrome}}stay{{when}}x", "chrome")).toBe("{{when chrome}}stayx");
    expect(applyWhen("keep {{date}}", "code")).toBe("keep {{date}}");
    expect(applyWhen("a{{whenever}}b", "code")).toBe("a{{whenever}}b");
    expect(
      applyWhen('{{when var: a: "AAA"}}git{{when var: a: "BBB"}}hg{{when}}none', {
        vars: { a: "AAA" },
      }),
    ).toBe("git");
    expect(applyWhen("{{when focus: Edit}}box{{when}}other", { focus: "edit" })).toBe("box");
    expect(applyWhen("{{when app: chrome}}web{{when focus: Edit}}in{{when}}none", {})).toBe("none");
  });
});

describe("contextApp", () => {
  it("strips the browser page", () => {
    expect(contextApp("chrome|GitHub")).toBe("chrome");
    expect(contextApp("code")).toBe("code");
    expect(contextApp(null)).toBe("");
  });
});

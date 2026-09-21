import { describe, expect, it } from "vitest";
import { applyWhen, contextApp } from "./when";

describe("applyWhen", () => {
  it("keeps the prefix and the matching section", () => {
    expect(applyWhen("id{{when chrome}}c{{when code}}k{{when}}x", "code")).toBe("idk");
    expect(applyWhen("{{when chrome}}c{{when excel}}e{{when}}d", "code")).toBe("d");
    expect(applyWhen("{{when chrome, excel}}hit{{when}}miss", "EXCEL")).toBe("hit");
    expect(applyWhen("keep {{date}}", "code")).toBe("keep {{date}}");
    expect(applyWhen("a{{whenever}}b", "code")).toBe("a{{whenever}}b");
  });
});

describe("contextApp", () => {
  it("strips the browser page", () => {
    expect(contextApp("chrome|GitHub")).toBe("chrome");
    expect(contextApp("code")).toBe("code");
    expect(contextApp(null)).toBe("");
  });
});

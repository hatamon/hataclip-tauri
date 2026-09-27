// TEST.md「登録」

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { sleep } from "./dump";

describeE2e("登録（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness();
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("選択した hello が先頭に登録され、クリップボードにも残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("hello"));
    await h.selectCopyClear();
    await h.register();
    const dump = await h.waitDump((state) => state.items[0]?.text === "hello");
    expect(dump.items).toHaveLength(1);
    expect(dump.clipboard).toBe("hello");
  });

  it("同じ hello をもう一度登録しても2件にならず先頭へ移る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.openNote();
    await h.selectCopyClear();
    await h.register();
    const dump = await h.waitDump((state) => state.items[0]?.text === "hello");
    expect(dump.items).toHaveLength(1);
  });

  it("選択が無いときはクリップボードの文字が先頭に登録される", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("cliptext"));
    await h.selectCopyClear();
    await h.register();
    const dump = await h.waitDump((state) => state.items[0]?.text === "cliptext");
    expect(dump.items[0]?.text).toBe("cliptext");
    expect(dump.items.some((item) => item.text === "hello")).toBe(true);
  });

  it("Ctrl を押したまま 4 の次に 7。7 は入らず一覧が出て、選択は登録される", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("world"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.notepad.keys(["Control", "c", "Control"]);
    await sleep(200);
    const before = await h.noteText();
    await h.openNote();
    await h.notepad.keys(["Control", "4", "7", "Control"]);
    const dump = await h.waitDump(
      (state) => state.picker?.open === true && state.items[0]?.text === "world",
    );
    expect(dump.picker?.open).toBe(true);
    const after = await h.noteText();
    expect(after).toBe(before);
    expect(after).not.toContain("7");
    await h.hidePicker();
  });

  it("URL には #url、パスには #path が付く", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("https://example.com"));
    await h.selectCopyClear();
    await h.register();
    const url = await h.waitDump((state) =>
      state.items.some((item) => item.text === "https://example.com" && item.tags.includes("url")),
    );
    expect(url.items.find((item) => item.text === "https://example.com")?.tags).toContain("url");

    await h.clearNote();
    await h.typeNote(chars("C:/Windows"));
    await h.selectCopyClear();
    await h.register();
    const path = await h.waitDump((state) =>
      state.items.some((item) => item.text === "C:/Windows" && item.tags.includes("path")),
    );
    expect(path.items.find((item) => item.text === "C:/Windows")?.tags).toContain("path");
  });
});

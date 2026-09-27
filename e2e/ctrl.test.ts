// TEST.md「Ctrl」の残り。先頭の Ctrl+4 は e2e/ctrl-register.test.ts。

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { DATE, plain } from "./text";
import { sleep } from "./dump";

describeE2e("Ctrl（TEST.md、先頭以外）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({ history: [{ text: "slotone" }] });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("Ctrl を離さず 7。Esc のあと 4 と 7 は文字として入る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote([...chars("keep"), "Enter", ...chars("{{date}}")]);
    const before = plain(await h.noteText());
    await h.notepad.keys(["Control", "7", "Control"]);
    await h.waitDump((state) => state.picker?.open === true);
    await h.hidePicker();
    await h.openNote();
    await h.typeNote(["4", "7"]);
    const after = plain(await h.noteText());
    expect(after.startsWith(before)).toBe(true);
    expect(after.endsWith("47")).toBe(true);
    expect(after).not.toMatch(/keep47/);
    const dump = await h.waitDump((state) => state.picker?.open === false);
    expect(dump.items.filter((item) => item.text === "47")).toHaveLength(0);
  });

  it("Ctrl+8 で2行目だけが日付になる。8 も c も v も入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("keep"));
    await h.selectCopyClear();
    await h.typeNote(["Enter", ...chars("{{date}}")]);
    const before = plain(await h.noteText());
    expect(before.split("\n")[0]).toBe("keep");
    await h.expand();
    await h.waitNote((text) => DATE.test(plain(text).split("\n")[1] ?? ""));
    const after = plain(await h.noteText());
    expect(after.split("\n")[0]).toBe("keep");
    expect(after.split("\n")[1]).toMatch(DATE);
    expect(after).not.toContain("8");
    expect(after.toLowerCase()).not.toContain("c");
    expect(after.toLowerCase()).not.toContain("v");
  });

  it("日付の末尾で Ctrl+8 をもう一度。8 は入らず keep は残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const before = plain(await h.noteText());
    await h.expand();
    const after = plain(await h.noteText());
    expect(after.split("\n")[0]).toBe("keep");
    expect(after).toBe(before);
    expect(after).not.toContain("8");
  });

  it("Ctrl+9 で 9 は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const before = plain(await h.noteText());
    await h.complete();
    expect(plain(await h.noteText())).toBe(before);
  });

  it("Ctrl+Shift+1 で先頭行が貼られ、1 も ! も入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("slotone"));
    await h.selectCopyClear();
    await h.register();
    await h.waitDump((state) => state.items[0]?.text === "slotone");
    await h.clearNote();
    await h.openNote();
    await h.notepad.keys(["Control", "Shift", "1", "Shift", "Control"]);
    await sleep(400);
    const text = plain(await h.noteText());
    expect(text).toBe("slotone");
    expect(text).not.toContain("!");
  });

  it("{{date}} を選択して Ctrl+8。選択が日付になり 8/c/v は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("{{date}}"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    const text = plain(await h.noteText());
    expect(text).toMatch(DATE);
    expect(text).not.toContain("8");
    expect(text.toLowerCase()).not.toContain("c");
    expect(text.toLowerCase()).not.toContain("v");
  });

  it("Ctrl を離して a。小文字の a が入る。全文は選択されない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote([...chars("keep"), "Enter", ...chars("tail")]);
    await h.typeNote(["a"]);
    const text = plain(await h.noteText());
    expect(text).toContain("keep");
    expect(text.endsWith("a")).toBe(true);
    expect(text).not.toBe("a");
  });

  it("Ctrl を離したまま 8。8 が文字として入り、展開はしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote([...chars("{{date}}"), "8"]);
    expect(plain(await h.noteText())).toBe("{{date}}8");
  });
});

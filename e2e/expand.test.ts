// TEST.md「展開（Ctrl+8）」

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { DATE, plain } from "./text";
import { sleep } from "./dump";

describeE2e("展開（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness();
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("{{date}} を選択して Ctrl+8。選択が今日の日付になる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("{{date}}"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toMatch(DATE);
  });

  it("行頭の {{date}} を選択せず Ctrl+8。c にはならず日付になる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("{{date}}"));
    await h.expand();
    const text = plain(await h.noteText());
    expect(text).toMatch(DATE);
    expect(text.toLowerCase()).not.toContain("c");
  });

  it("Ctrl を押したまま Ctrl+8 を何度か押す。8 は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("keep"));
    await h.openNote();
    await h.notepad.keys(["Control", "8", "8", "8", "Control"]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("keep");
  });

  it("選択せず Ctrl+7。一覧は出る。7 は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("keep"));
    await h.showPicker();
    expect(plain(await h.noteText())).toBe("keep");
    await h.hidePicker();
  });

  it("abc {{date}} を選択せず Ctrl+8。abc が残り後ろが日付", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("abc {{date}}"));
    await h.expand();
    const text = plain(await h.noteText());
    expect(text.startsWith("abc ")).toBe(true);
    expect(text.slice(4)).toMatch(DATE);
  });

  it("abc :echo 2+3 を選択せず Ctrl+8。abc が残り後ろが 5", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("abc :echo 2+3"));
    await h.openNote();
    await h.expand();
    await h.waitNote((text) => plain(text) === "abc 5");
  });

  it("行頭までの選択に改行があるときは置き換わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote([...chars("{{date}}"), "Enter", ...chars("tail")]);
    await h.notepad.keys(["Control", "Shift", "Home", "Shift", "Control"]);
    const before = plain(await h.noteText());
    await h.expand();
    expect(plain(await h.noteText())).toBe(before);
  });

  it(":sh dir を選択して Ctrl+8。確定で置き換わり、取り消しは残す", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars(":sh dir"));
    await h.selectCopyClear();
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    const confirm = await h.waitDump((state) => state.picker?.mode === "help");
    expect(confirm.picker?.mode).toBe("help");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
    expect(plain(await h.noteText())).toBe(":sh dir");
    await h.clearNote();
    await h.notepad.keys(["Control", "v", "Control"]);
    await sleep(200);
    expect(plain(await h.noteText())).toBe(":sh dir");

    await h.clearNote();
    await h.typeNote(chars(":sh dir"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Enter"]);
    await h.waitNote((text) => plain(text).trim() !== ":sh dir" && plain(text).trim().length > 0);
  });

  it("{{focus}} を選択して Ctrl+8。そのまま残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("{{focus}}"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("{{focus}}");
  });

  it("展開結果が空なら選択もクリップボードも変わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("keepclip"));
    await h.selectCopyClear();
    await h.clearNote();
    await h.typeNote(chars("{{env:HATACLIP_NO_SUCH_E2E}}"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("{{env:HATACLIP_NO_SUCH_E2E}}");
    await h.clearNote();
    await h.notepad.keys(["Control", "v", "Control"]);
    await sleep(200);
    expect(plain(await h.noteText())).toBe("keepclip");
  });

  it(":showerror で直前の失敗が出て、成功すると消える", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars(":echo nope"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    await h.showPicker();
    await h.colon("showerror");
    const shown = await h.waitDump((state) => state.picker?.mode === "help");
    expect(shown.picker?.mode).toBe("help");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
    await h.clearNote();
    await h.typeNote(chars(":echo 1+1"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("2");
  });
});

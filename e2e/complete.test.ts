// TEST.md「補完」

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { DATE, plain } from "./text";
import { sleep } from "./dump";

const history = [
  { text: "work note" },
  { text: "work task" },
  { text: "work hidden", tags: ["secret"] },
];

describeE2e("補完（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({ history });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("/work を選択して Ctrl+8。先頭候補が貼られ / は検索語に入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("/work"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("work note");
  });

  it("/ だけの選択は何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(["/"]);
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("/");
  });

  it("/xxx {{date}} は検索語のまま。日付には展開しない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("otherclip"));
    await h.selectCopyClear();
    await h.clearNote();
    await h.typeNote(chars("/xxx {{date}}"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.notepad.keys(["Control", "8", "Control"]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("/xxx {{date}}");
  });

  it("行頭の abc /work を選択せず Ctrl+8。abc は残り補完される", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("abc /work"));
    await h.expand();
    expect(plain(await h.noteText())).toBe("abc work note");
  });

  it("abc /work {{date}} を選択せず Ctrl+8。最後の {{ から展開する", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("abc /work {{date}}"));
    await h.expand();
    const text = plain(await h.noteText());
    expect(text.startsWith("abc /work ")).toBe(true);
    expect(text.slice("abc /work ".length)).toMatch(DATE);
  });

  it("2秒以内にもう一度 Ctrl+8。次の候補があれば入れ替わる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("/work"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("work note");
    await h.expand();
    expect(plain(await h.noteText())).toBe("work task");
  });

  it("一覧の /work と Ctrl+8 の当たりが同じ。#secret の本文は使わない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("work");
    const listed = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    expect(listed.picker?.filtered[0]).toBe("work note");
    await h.hidePicker();
    await h.clearNote();
    await h.typeNote(chars("/work"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    expect(plain(await h.noteText())).toBe("work note");
  });

  it("Ctrl+9 は先頭の / が無くても同じ補完をする", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await sleep(2100);
    await h.clearNote();
    await h.typeNote(chars("work"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.complete();
    await h.waitNote((text) => plain(text) === "work note");
  });
});

// TEST.md「一覧を出す」。端へのドラッグと画面外は dump に位置が無いので見ない。

import { afterAll, beforeAll, expect, it } from "vitest";
import { existsSync, readFileSync } from "node:fs";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { sleep } from "./dump";

type SettingsFile = {
  font_px?: number;
  window?: { x: number; y: number; width: number; height: number };
};

function readSettings(h: Harness): SettingsFile {
  const path = `${h.testDirContainer}/settings.json`;
  if (!existsSync(path)) {
    return {};
  }
  try {
    return JSON.parse(readFileSync(path, "utf8")) as SettingsFile;
  } catch {
    return {};
  }
}

describeE2e("一覧を出す（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [{ text: "keep-sel" }, { text: "other" }],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("Ctrl+7 で一覧が出る。前回貼った行があればそこが選ばれる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("keep-sel");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const shown = await h.showPicker();
    expect(shown.picker?.open).toBe(true);
    expect(shown.picker?.selected).toBe("keep-sel");
    await h.hidePicker();
  });

  it("Ctrl+矢印で動き、閉じて settings.json に位置が残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["Control", "ArrowRight", "Control"]);
    await sleep(300);
    await h.hidePicker();
    await sleep(300);
    const settings = readSettings(h);
    expect(settings.window).toBeTruthy();
    expect(typeof settings.window?.x).toBe("number");
    expect(typeof settings.window?.y).toBe("number");
  });

  it("Ctrl+Shift+左右で幅が変わり、下限より小さくならない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    for (let i = 0; i < 40; i += 1) {
      await h.pickerKeys(["Control", "Shift", "ArrowLeft", "Shift", "Control"]);
    }
    await sleep(300);
    await h.hidePicker();
    await sleep(300);
    const settings = readSettings(h);
    expect(settings.window?.width ?? 0).toBeGreaterThanOrEqual(200);
  });

  it("Ctrl+; で文字が大きく、Ctrl+- で小さくなる。9〜32 の外には出ない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["Control", ";", "Control"]);
    await sleep(300);
    const up = readSettings(h).font_px ?? 13;
    expect(up).toBeGreaterThanOrEqual(13);
    expect(up).toBeLessThanOrEqual(32);
    for (let i = 0; i < 30; i += 1) {
      await h.pickerKeys(["Control", "-", "Control"]);
    }
    await sleep(300);
    const down = readSettings(h).font_px ?? 13;
    expect(down).toBeGreaterThanOrEqual(9);
    expect(down).toBeLessThanOrEqual(13);
    for (let i = 0; i < 30; i += 1) {
      await h.pickerKeys(["Control", ";", "Control"]);
    }
    await sleep(300);
    const max = readSettings(h).font_px ?? 13;
    expect(max).toBeLessThanOrEqual(32);
    await h.hidePicker();
  });

  it("一覧からフォーカスが外れると隠れる。編集・コロン・ヘルプ・タグのあいだは残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.openNote();
    await h.waitDump((state) => state.picker?.open === false);

    await h.showPicker();
    await h.pickerKeys([":"]);
    await h.waitDump((state) => state.picker?.mode === "colon");
    expect((await h.waitDump(() => true)).picker?.open).toBe(true);
    await h.pickerKeys(["Escape"]);

    await h.showPicker();
    await h.colon("help");
    await h.waitDump((state) => state.picker?.mode === "help" && state.picker.open);
    await h.pickerKeys(["Escape"]);

    await h.showPicker();
    await h.pickerKeys(["e"]);
    await h.waitDump((state) => state.picker?.mode === "editing");
    expect((await h.waitDump(() => true)).picker?.open).toBe(true);
    await h.pickerKeys(["Escape"]);

    await h.showPicker();
    await h.pickerKeys(["t"]);
    await h.waitDump((state) => state.picker?.mode === "tag");
    expect((await h.waitDump(() => true)).picker?.open).toBe(true);
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
  });
});

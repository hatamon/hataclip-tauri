// TEST.md「コマンド」

import { afterAll, beforeAll, expect, it } from "vitest";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { containerPath, hostPath } from "./paths";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { plain } from "./text";
import { sleep } from "./dump";

describeE2e("コマンド（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "alpha" },
        { text: "old word" },
        { text: "zebra" },
        { text: "plain" },
        { text: "{{n}}" },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it(": のあと Tab でコマンド名が補完される。選択行は見える", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("alpha");
    await h.pickerKeys([":"]);
    const colon = await h.waitDump((state) => state.picker?.mode === "colon");
    expect(colon.picker?.selected).toBe("alpha");
    await h.pickerKeys(["e", "Tab"]);
    await sleep(200);
    expect((await h.waitDump(() => true)).picker?.selected).toBe("alpha");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
  });

  it(":help で目次。:help template で説明。Esc でヘルプだけ閉じる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("help");
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Escape"]);
    await h.waitDump((state) => state.picker?.mode === "normal" && state.picker.open);
    await h.colon("help template");
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Alt", "ArrowLeft", "Alt"]);
    await sleep(200);
    await h.pickerKeys(["Alt", "ArrowRight", "Alt"]);
    await sleep(200);
    await h.pickerKeys(["Escape"]);
    const back = await h.waitDump((state) => state.picker?.mode === "normal");
    expect(back.picker?.open).toBe(true);
    await h.hidePicker();
  });

  it(":echo 2+3 は 5。:echo 0xff は 255。失敗したら貼らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 2+3");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("5");

    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 0xff");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("255");

    await h.clearNote();
    await h.showPicker();
    await h.colon("echo nope");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });

  it(":sh echo hi は hi を貼る。履歴には残らない。:@ でもう一度", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("sh echo hi");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).trim()).toBe("hi");
    const afterSh = await h.waitDump((state) => state.items.length > 0);
    expect(afterSh.items.some((item) => item.text === "hi")).toBe(false);

    await h.clearNote();
    await h.showPicker();
    await h.colon("@");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).trim()).toBe("hi");
  });

  it(":.! は選択を stdin にして貼る。:!! は本文を書き換え u で戻る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("alpha");
    await h.colon(".!sh echo hi");
    await sleep(600);
    expect(plain(await h.noteText()).trim()).toBe("hi");
    expect((await h.waitDump(() => true)).items.some((item) => item.text === "alpha")).toBe(true);

    await h.revealRow("zebra");
    await h.colon("!!sh echo hi");
    const rewritten = await h.waitDump((state) => state.items.some((item) => item.text.trim() === "hi"));
    expect(rewritten.items.some((item) => item.text === "zebra")).toBe(false);
    await h.pickerKeys(["u"]);
    const undone = await h.waitDump((state) => state.items.some((item) => item.text === "zebra"));
    expect(undone.items.some((item) => item.text === "zebra")).toBe(true);
    await h.hidePicker();
  });

  it(":s/old/new は選択の old を new にして貼る。履歴は変わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("old word");
    await h.colon("s/old/new");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("new word");
    const dump = await h.waitDump((state) => state.items.some((item) => item.text === "old word"));
    expect(dump.items.find((item) => item.text === "old word")).toBeTruthy();
  });

  it("V 中の :sort で本文の順に並び、u で戻る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["g", "g"]);
    await h.pickerKeys(["V"]);
    const listed = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 1);
    const count = listed.picker?.filtered.length ?? 0;
    for (let i = 1; i < count; i += 1) {
      await h.pickerKeys(["j"]);
    }
    const before = listed.picker?.filtered ?? [];
    await h.colon("sort");
    const sorted = await h.waitDump((state) => {
      const rows = state.picker?.filtered ?? [];
      const copy = [...rows].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
      return rows.length > 1 && rows.every((row, index) => row === copy[index]);
    });
    const rows = sorted.picker?.filtered ?? [];
    expect(rows).not.toEqual(before);
    await h.pickerKeys(["u"]);
    await h.waitDump((state) => {
      const rows = state.picker?.filtered ?? [];
      return rows.length === before.length && rows.every((row, index) => row === before[index]);
    });
    await h.hidePicker();
  });

  it(":set a=1 のあと :set で見え、:set a= で消える", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("set a=1");
    await h.colon("set");
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Escape"]);
    await h.colon("set a=");
    await h.colon("set");
    await sleep(300);
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
  });

  it(":set copy / paste / home は前面アプリのキー。:set cut でも切り取りは送らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("set copy ctrl+shift+c");
    await h.colon("set paste ctrl+shift+v");
    await h.colon("set home shift+home");
    await h.colon("set cut ctrl+x");
    await sleep(300);
    const raw = readFileSync(`${h.testDirContainer}/settings.json`, "utf8");
    expect(raw.toLowerCase()).toContain("ctrl+shift+c");
    expect(plain(await h.noteText())).not.toContain("x");
    await h.hidePicker();
  });

  it(":map でキーを付け替え、:unmap で戻る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("map q :echo 1");
    await sleep(200);
    await h.pickerKeys(["q"]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("1");
    await h.showPicker();
    await h.colon("unmap q");
    await h.clearNote();
    await h.pickerKeys(["q"]);
    await sleep(300);
    expect(plain(await h.noteText())).not.toBe("1");
    await h.hidePicker();
  });

  it(":export で Markdown が書け、:import で読み戻せる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const out = hostPath("exp.md").replace(/\\/g, "/");
    await h.showPicker();
    await h.colon(`export ${out}`);
    await sleep(400);
    expect(existsSync(containerPath("exp.md"))).toBe(true);
    expect(readFileSync(containerPath("exp.md"), "utf8")).toContain("alpha");
    await h.colon(`import ${out}`);
    await sleep(400);
    const dump = await h.waitDump((state) => state.items.some((item) => item.text === "alpha"));
    expect(dump.items.some((item) => item.text === "alpha")).toBe(true);
    await h.hidePicker();
  });

  it(":crypt 鍵で暗号文になり、:decrypt で前面へ。違う鍵では何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("plain");
    await h.colon("crypt k1");
    const sealed = await h.waitDump((state) =>
      state.items.some((item) => item.text.startsWith("hataclip1.")),
    );
    const cipher = sealed.items.find((item) => item.text.startsWith("hataclip1."))?.text ?? "";
    expect(cipher.startsWith("hataclip1.")).toBe(true);
    await h.revealRow(cipher);
    await h.colon("decrypt k2");
    await sleep(300);
    expect(plain(await h.noteText())).not.toBe("plain");
    await h.revealRow(cipher);
    await h.colon("decrypt k1");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("plain");
    const still = await h.waitDump((state) => state.items.some((item) => item.text === cipher));
    expect(still.items.some((item) => item.text === cipher)).toBe(true);
  });

  it(":settings で settings.json を開こうとする。壊れていても今の設定のまま再開できる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const path = `${h.testDirContainer}/settings.json`;
    const previous = existsSync(path) ? readFileSync(path, "utf8") : "{}";
    writeFileSync(path, "{");
    await h.showPicker();
    await h.colon("settings");
    await sleep(400);
    await h.quitNvim();
    const opened = await h.waitDump((state) => typeof state.lastOpen === "string");
    expect(opened.lastOpen ?? "").toContain("settings.json");
    writeFileSync(path, previous);
    await h.showPicker();
    expect((await h.waitDump(() => true)).picker?.open).toBe(true);
    await h.hidePicker();
  });

  it(":n は何もしない。{{n}} は展開されない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("n");
    await sleep(300);
    expect(plain(await h.noteText())).toBe("");
    await h.revealRow("{{n}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("{{n}}");
  });
});

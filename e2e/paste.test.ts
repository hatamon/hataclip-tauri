// TEST.md「一覧から貼る」

import { afterAll, beforeAll, expect, it } from "vitest";
import { existsSync, readFileSync } from "node:fs";
import { containerPath, hostPath } from "./paths";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { sleep } from "./dump";

const formatSrc = "> hello\n\n    indented";
const formatOut = "hello\n\n    indented";

const history = [
  { text: "asdf" },
  { text: ":echo 3+3" },
  { text: "{{date}}" },
  { text: formatSrc },
  { text: "alpha" },
  { text: "bravo" },
  { text: "quote-me" },
  { text: "typedok" },
  { text: "https://example.com" },
  { text: "loghello" },
];

function plain(text: string): string {
  return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

async function pick(h: Harness, query: string, selected: string): Promise<void> {
  await h.showPicker();
  await h.search(query);
  await h.waitDump((state) => state.picker?.selected === selected);
  // Esc だと絞りが外れて先頭行に飛ぶ。Tab なら選んだ行のまま一覧へ戻る。
  await h.pickerKeys(["Tab"]);
  await h.waitDump(
    (state) => state.picker?.mode === "normal" && state.picker.selected === selected,
  );
}

async function revealRow(h: Harness, selected: string): Promise<void> {
  await h.showPicker();
  await h.pickerKeys(["g", "g"]);
  const list = await h.waitDump((state) => (state.picker?.filtered ?? []).includes(selected));
  const at = list.picker?.filtered.indexOf(selected) ?? -1;
  for (let i = 0; i < at; i += 1) {
    await h.pickerKeys(["j"]);
  }
  await h.waitDump((state) => state.picker?.selected === selected);
}

describeE2e("一覧から貼る（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({ history });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("asdf で Enter。一覧が閉じ、メモ帳に asdf が貼られ、v は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "asdf", "asdf");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const text = await h.noteText();
    expect(plain(text)).toBe("asdf");
    expect(text).not.toContain("v");
  });

  it(":echo 3+3 で Enter。一覧が閉じ、メモ帳に 6 が貼られる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "3+3", ":echo 3+3");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("6");
  });

  it("1 で1番目が貼られて閉じ、Ctrl+1 では貼ったあと一覧が残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.pickerKeys(["g", "g"]);
    const first = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    expect(first.picker?.filtered?.[0]).toBe("asdf");
    await h.pickerKeys(["1"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("asdf");

    await h.clearNote();
    await h.showPicker();
    await h.pickerKeys(["g", "g"]);
    await h.waitDump((state) => state.picker?.open === true);
    await h.pickerKeys(["Control", "1", "Control"]);
    const kept = await h.waitDump((state) => state.picker?.open === true && (state.clipboard ?? "").length > 0);
    expect(kept.picker?.open).toBe(true);
    await h.hidePicker();
    expect(plain(await h.noteText()).length).toBeGreaterThan(0);
  });

  it("貼ったあと Ctrl+7 するとその行が選ばれる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "quote-me", "quote-me");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const again = await h.showPicker();
    expect(again.picker?.selected).toBe("quote-me");
    await h.hidePicker();
  });

  it("g. でもう一度同じ行が貼られる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "asdf", "asdf");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    await h.showPicker();
    await h.pickerKeys(["g", "."]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("asdfasdf");
    await h.hidePicker();
  });

  it("Ctrl+C で展開結果がクリップボードに入り、一覧の本文は変わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await pick(h, "{{date}}", "{{date}}");
    await h.pickerKeys(["Control", "c", "Control"]);
    // コピーだけでは dump が書き直されないので、表示を一度動かす。
    await h.pickerKeys(["g", "e"]);
    const dump = await h.waitDump((state) => /^\d{4}\/\d{2}\/\d{2}$/.test(state.clipboard ?? ""));
    expect(dump.clipboard).toMatch(/^\d{4}\/\d{2}\/\d{2}$/);
    expect(dump.items.some((item) => item.text === "{{date}}")).toBe(true);
    await h.hidePicker();
  });

  it(":raw は {{date}} を日付にせず本文のまま貼る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "{{date}}", "{{date}}");
    await h.pickerKeys([":", ..."raw".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("{{date}}");
  });

  it(":format は引用と余分な空行と囲み引用符を外して貼る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "indented", formatSrc);
    await h.pickerKeys([":", ..."format".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe(formatOut);
  });

  it(":join は選択行を , でつなぎ、区切りを付けるとそれでつなぐ", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await revealRow(h, "alpha");
    await h.pickerKeys(["V", "j"]);
    await h.pickerKeys([":", ..."join".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("alpha,bravo");

    await h.clearNote();
    await revealRow(h, "alpha");
    await h.pickerKeys(["V", "j"]);
    // JIS では WAD の \\ が ] になるので、タブの代わりに | で区切り指定を見る。
    await h.pickerKeys([":", ..."join \"|\"".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("alpha|bravo");
  });

  it(":quote は行頭に > を付けて貼り、履歴の本文は変わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "quote-me", "quote-me");
    await h.pickerKeys([":", ..."quote".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("> quote-me");
    const dump = await h.waitDump((state) => state.items.some((item) => item.text === "quote-me"));
    expect(dump.items.find((item) => item.text === "quote-me")?.text).toBe("quote-me");
  });

  it(":type は1文字ずつ送り、v は入らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "typedok", "typedok");
    await h.pickerKeys([":", ..."type".split(""), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("typedok");
    expect(await h.noteText()).not.toContain("v");
  });

  it(":open は URL ならその行き先を開く", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await pick(h, "example.com", "https://example.com");
    await h.pickerKeys([":", ..."open".split(""), "Enter"]);
    const dump = await h.waitDump((state) => state.lastOpen === "https://example.com");
    expect(dump.lastOpen).toBe("https://example.com");
    await h.hidePicker();
  });

  it(":log は変数のファイルへ追記し、前面には貼らない。無ければ何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await pick(h, "loghello", "loghello");
    await h.pickerKeys([":", ..."log".split(""), "Enter"]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    expect(existsSync(containerPath("e2e-log.txt"))).toBe(false);

    const logHost = hostPath("e2e-log.txt").replace(/\\/g, "/");
    await pick(h, "loghello", "loghello");
    await h.pickerKeys([":", ...`log ${logHost}`.split(""), "Enter"]);
    await sleep(400);
    const written = readFileSync(containerPath("e2e-log.txt"), "utf8");
    expect(written).toContain("loghello");
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });

  it("貼ったあともクリップボードは戻さない。最後に書いた文字が残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("oldclip"));
    await h.selectCopyClear();
    await h.clearNote();
    await pick(h, "asdf", "asdf");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const dump = await h.waitDump((state) => (state.clipboard ?? "").includes("asdf"));
    expect(dump.clipboard).toContain("asdf");
    expect(dump.clipboard).not.toBe("oldclip");
  });
});

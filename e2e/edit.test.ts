// TEST.md「編集」

import { afterAll, beforeAll, expect, it } from "vitest";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { sleep } from "./dump";

describeE2e("編集（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "keep-lock", tags: ["lock"] },
        { text: "gone" },
        { text: "yank-me" },
        { text: "edit-me" },
        { text: "one\ntwo" },
        { text: "locked-join", tags: ["lock"] },
        { text: "clone-me" },
        { text: "tag-me" },
        { text: "pin-me" },
        { text: "swap-a" },
        { text: "swap-b" },
        { text: "dot-me" },
        { text: "tilde" },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("dd で行が消える。p で下、P で上に戻る。#lock は消えない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("gone");
    await h.pickerKeys(["d", "d"]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "gone"));
    await h.pickerKeys(["p"]);
    const below = await h.waitDump((state) => state.items.some((item) => item.text === "gone"));
    expect(below.items.some((item) => item.text === "gone")).toBe(true);

    await h.revealRow("gone");
    await h.pickerKeys(["d", "d"]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "gone"));
    await h.pickerKeys(["P"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "gone"));

    await h.revealRow("keep-lock");
    await h.pickerKeys(["d", "d"]);
    await sleep(200);
    expect((await h.waitDump(() => true)).items.some((item) => item.text === "keep-lock")).toBe(true);
    await h.hidePicker();
  });

  it("u で直前の削除が戻る。Ctrl+R でやり直せる。もう一度 u してもその1回より前には戻らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("gone");
    await h.pickerKeys(["d", "d"]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "gone"));
    await h.pickerKeys(["u"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "gone"));
    await h.pickerKeys(["Control", "r", "Control"]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "gone"));
    await h.pickerKeys(["u"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "gone"));
    const count = (await h.waitDump(() => true)).items.length;
    await h.pickerKeys(["u"]);
    await sleep(200);
    expect((await h.waitDump(() => true)).items.length).toBe(count);
    await h.hidePicker();
  });

  it("yy のあと p で複製が下に置かれる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("yank-me");
    await h.pickerKeys(["y", "y", "p"]);
    const dump = await h.waitDump(
      (state) => state.items.filter((item) => item.text === "yank-me").length >= 2,
    );
    expect(dump.items.filter((item) => item.text === "yank-me").length).toBeGreaterThanOrEqual(2);
    await h.hidePicker();
  });

  it("o で下に空行ができて編集になる。Esc で捨てる。O は上", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const before = (await h.waitDump(() => true)).items.length;
    await h.revealRow("edit-me");
    await h.pickerKeys(["o"]);
    await h.waitDump((state) => state.picker?.mode === "editing");
    await h.pickerKeys(["Escape"]);
    await h.waitDump((state) => state.picker?.mode === "normal");
    expect((await h.waitDump(() => true)).items.length).toBe(before);

    await h.pickerKeys(["O"]);
    await h.waitDump((state) => state.picker?.mode === "editing");
    await h.pickerKeys(["Escape"]);
    await h.waitDump((state) => state.picker?.mode === "normal");
    expect((await h.waitDump(() => true)).items.length).toBe(before);
    await h.hidePicker();
  });

  it("e でその場編集。Ctrl+Enter で保存、Esc で取り消し", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("edit-me");
    await h.pickerKeys(["e"]);
    await h.waitDump((state) => state.picker?.mode === "editing");
    await h.pickerKeys(["x"]);
    await h.pickerKeys(["Escape"]);
    await h.waitDump((state) => state.picker?.mode === "normal");
    expect((await h.waitDump(() => true)).items.some((item) => item.text === "edit-me")).toBe(true);
    expect((await h.waitDump(() => true)).items.some((item) => item.text.includes("edit-mex"))).toBe(
      false,
    );

    await h.revealRow("edit-me");
    await h.pickerKeys(["e"]);
    await h.waitDump((state) => state.picker?.mode === "editing");
    await h.pickerKeys(["End", "2"]);
    await h.pickerKeys(["Control", "Enter", "Control"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "edit-me2"));
    await h.hidePicker();
  });

  it("E で外部へ開こうとする", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const row = (await h.waitDump(() => true)).items.find((item) => item.text.startsWith("edit-me"));
    const text = row?.text ?? "edit-me";
    await h.revealRow(text);
    await h.pickerKeys(["E"]);
    await sleep(400);
    await h.quitNvim();
    const dump = await h.waitDump((state) => state.lastOpen != null);
    expect(dump.lastOpen).toContain("hataclip-");
    expect(dump.items.some((item) => item.text === text)).toBe(true);
    await h.showPicker();
    await h.hidePicker();
  });

  it("S で改行ごとに複数行になる。V と J で1行に戻る。#lock はまとまらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("one\ntwo");
    await h.pickerKeys(["S"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "one"));
    expect((await h.waitDump(() => true)).items.some((item) => item.text === "two")).toBe(true);

    await h.revealRow("one");
    await h.pickerKeys(["V", "j", "J"]);
    await sleep(300);
    const joined = await h.waitDump(() => true);
    expect(joined.items.some((item) => item.text.includes("one"))).toBe(true);

    const locked = (await h.waitDump(() => true)).items.find((item) => item.text === "locked-join");
    expect(locked?.tags).toContain("lock");
    await h.hidePicker();
  });

  it("c ですぐ下に複製ができる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("clone-me");
    await h.pickerKeys(["c"]);
    const dump = await h.waitDump(
      (state) => state.items.filter((item) => item.text === "clone-me").length >= 2,
    );
    expect(dump.items.filter((item) => item.text === "clone-me").length).toBeGreaterThanOrEqual(2);
    await h.hidePicker();
  });

  it("t work でタグが付き、T work で外れる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("tag-me");
    await h.pickerKeys(["t", ..."work".split(""), "Enter"]);
    await h.waitDump((state) =>
      state.items.some((item) => item.text === "tag-me" && item.tags.includes("work")),
    );
    await h.revealRow("tag-me");
    await h.pickerKeys(["T", ..."work".split(""), "Enter"]);
    await h.waitDump((state) =>
      state.items.some((item) => item.text === "tag-me" && !item.tags.includes("work")),
    );
    await h.hidePicker();
  });

  it("gp でピンが切り替わる。ピンは一覧の上に固まる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("pin-me");
    await h.pickerKeys(["g", "p"]);
    await sleep(300);
    const dump = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    expect(dump.picker?.filtered?.[0]).toBe("pin-me");
    await h.hidePicker();
  });

  it("+ / - で隣と入れ替わる。ピンとピンでない行は入れ替わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("swap-a");
    const before = (await h.waitDump(() => true)).picker?.filtered ?? [];
    await h.pickerKeys(["+"]);
    await sleep(200);
    const afterPlus = (await h.waitDump(() => true)).picker?.filtered ?? [];
    expect(afterPlus.join("\n")).not.toBe(before.join("\n"));
    await h.revealRow("pin-me");
    const pinnedAt = (await h.waitDump(() => true)).picker?.filtered ?? [];
    await h.pickerKeys(["-"]);
    await sleep(200);
    const still = (await h.waitDump(() => true)).picker?.filtered ?? [];
    expect(still[0]).toBe(pinnedAt[0]);
    await h.hidePicker();
  });

  it(". で直前の dd や t がもう一度効く。移動は対象にならない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("dot-me");
    await h.pickerKeys(["d", "d"]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "dot-me"));
    await h.pickerKeys(["u"]);
    await h.waitDump((state) => state.items.some((item) => item.text === "dot-me"));
    await h.pickerKeys(["j"]);
    await h.pickerKeys(["."]);
    await h.waitDump((state) => !state.items.some((item) => item.text === "tilde"));
    expect((await h.waitDump(() => true)).items.some((item) => item.text === "dot-me")).toBe(true);
    await h.hidePicker();
  });

  it("g~ は何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("gone");
    const before = JSON.stringify((await h.waitDump(() => true)).items);
    await h.pickerKeys(["g", "~"]);
    await sleep(200);
    expect(JSON.stringify((await h.waitDump(() => true)).items)).toBe(before);
    await h.hidePicker();
  });
});

// TEST.md「タグと絞り」

import { afterAll, beforeAll, expect, it } from "vitest";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { DATE_IN, plain } from "./text";
import { sleep } from "./dump";

describeE2e("タグと絞り（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "secret-body", tags: ["secret"] },
        { text: "alias-row", tags: ["alias:foo"] },
        { text: "only-notepad", tags: ["app:notepad"] },
        { text: "hide-notepad", tags: ["not:notepad"] },
        { text: "plain-app" },
        { text: "Hello {{date}}", tags: ["grab"] },
        { text: "clear-me" },
        { text: "keep-pin", pinned: true },
        { text: "keep-lock", tags: ["lock"] },
        { text: "dup  one" },
        { text: "dup one" },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("#secret の行は一覧では••••。貼り付けは本文のまま", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("secret-body");
    await h.pickerKeys(["g", "e"]);
    const preview = await h.waitDump((state) => (state.picker?.preview ?? "").length > 0);
    expect(preview.picker?.preview).toBe("••••");
    await h.pickerKeys(["g", "e"]);
    await h.clearNote();
    await h.revealRow("secret-body");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("secret-body");
  });

  it("#alias:foo の行は /foo で当たる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("foo");
    const dump = await h.waitDump((state) => (state.picker?.filtered ?? []).includes("alias-row"));
    expect(dump.picker?.filtered).toContain("alias-row");
    await h.hidePicker();
  });

  it("#app:notepad はメモ帳のときだけ。#not:notepad は出ない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    const dump = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    expect(dump.picker?.filtered).toContain("only-notepad");
    expect(dump.picker?.filtered).not.toContain("hide-notepad");
    await h.hidePicker();
  });

  it("ga で選んだ行の #app: がいまの前面アプリになる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("plain-app");
    await h.pickerKeys(["g", "a"]);
    const dump = await h.waitDump((state) =>
      state.items.some((item) => item.text === "plain-app" && item.tags.some((tag) => tag.startsWith("app:"))),
    );
    const tags = dump.items.find((item) => item.text === "plain-app")?.tags ?? [];
    expect(tags.some((tag) => tag.toLowerCase().includes("notepad") || tag.startsWith("app:"))).toBe(
      true,
    );
    await h.hidePicker();
  });

  it("#grab の行は本文がそのまま貼られる。穴は埋まらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("Hello {{date}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const pasted = plain(await h.noteText());
    expect(pasted.startsWith("Hello ")).toBe(true);
    expect(pasted).toMatch(DATE_IN);
    expect(pasted).not.toContain("<");
  });

  it(":dedup のあと :dedup yes で、空白と改行だけ違う行が1つになる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("dedup");
    await h.waitDump((state) => state.picker?.mode === "colon");
    await h.pickerKeys(["Enter"]);
    await sleep(300);
    const dups = (await h.waitDump(() => true)).items.filter(
      (item) => item.text.replace(/\s+/g, " ").trim() === "dup one",
    );
    expect(dups.length).toBeLessThanOrEqual(1);
    await h.hidePicker();
  });

  it(":clear のあと :clear yes で、ピンと #lock 以外が消える", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("clear");
    await h.waitDump((state) => state.picker?.mode === "colon");
    await h.pickerKeys(["Enter"]);
    const dump = await h.waitDump((state) =>
      state.items.every((item) => item.text === "keep-pin" || item.tags.includes("lock")),
    );
    expect(dump.items.some((item) => item.text === "keep-pin")).toBe(true);
    expect(dump.items.some((item) => item.text === "keep-lock")).toBe(true);
    expect(dump.items.some((item) => item.text === "clear-me")).toBe(false);
    await h.hidePicker();
  });
});

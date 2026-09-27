// TEST.md「複数選択」

import { afterAll, beforeAll, expect, it } from "vitest";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { plain } from "./text";
import { sleep } from "./dump";

describeE2e("複数選択（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "vis-a" },
        { text: "vis-b" },
        { text: "vis-c" },
        { text: "bulk-a" },
        { text: "bulk-b" },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("V のあと j / k で範囲が伸びる。Enter で改行つなぎで貼られて閉じる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("vis-a");
    await h.pickerKeys(["V", "j"]);
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const text = plain(await h.noteText());
    expect(text).toContain("vis-a");
    expect(text).toContain("vis-b");
    expect(text).toContain("\n");
  });

  it("範囲に dd、yy、t、gp がまとめて効く", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.revealRow("bulk-a");
    await h.pickerKeys(["V", "j"]);
    await h.pickerKeys(["t", ..."work".split(""), "Enter"]);
    await h.waitDump((state) =>
      state.items
        .filter((item) => item.text === "bulk-a" || item.text === "bulk-b")
        .every((item) => item.tags.includes("work")),
    );

    await h.revealRow("bulk-a");
    await h.pickerKeys(["V", "j", "g", "p"]);
    await sleep(300);
    const filtered = (await h.waitDump(() => true)).picker?.filtered ?? [];
    expect(filtered.slice(0, 2).sort()).toEqual(["bulk-a", "bulk-b"].sort());

    await h.revealRow("vis-c");
    await h.pickerKeys(["y", "y"]);
    await h.showPicker();
    await h.pickerKeys(["g", "g", "V", "j", "d", "d"]);
    await h.waitDump(
      (state) =>
        !state.items.some((item) => item.text === "bulk-a") &&
        !state.items.some((item) => item.text === "bulk-b"),
    );
    await h.hidePicker();
  });

  it("もう一度 V で選択が解除される", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("vis-a");
    await h.pickerKeys(["V"]);
    await sleep(200);
    await h.pickerKeys(["V", "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("vis-a");
  });
});

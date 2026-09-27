// TEST.md「移動と検索」

import { afterAll, beforeAll, expect, it } from "vitest";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { sleep } from "./dump";

const history = [
  { text: "apple" },
  { text: "apricot" },
  { text: "avocado" },
  { text: "banana" },
  { text: "cherry" },
  { text: "date" },
  { text: "echo" },
  { text: "fig" },
  { text: "grape" },
  { text: "honey" },
  { text: "work-note", tags: ["work"] },
  { text: "work-task", tags: ["work"] },
];

describeE2e("移動と検索（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({ history });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("j / k と矢印で1行動き、端で止まり、選んだ行が見える", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["g", "g"]);
    const first = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    const top = first.picker?.filtered?.[0];
    await h.pickerKeys(["j"]);
    const down = await h.waitDump((state) => state.picker?.selected !== top);
    expect(down.picker?.selected).toBe(first.picker?.filtered?.[1]);
    expect(down.picker?.visible).toContain(down.picker?.selected);

    await h.pickerKeys(["ArrowDown"]);
    const arrow = await h.waitDump((state) => state.picker?.selected === first.picker?.filtered?.[2]);
    expect(arrow.picker?.selected).toBe(first.picker?.filtered?.[2]);

    await h.pickerKeys(["k"]);
    await h.pickerKeys(["ArrowUp"]);
    const back = await h.waitDump((state) => state.picker?.selected === top);
    expect(back.picker?.selected).toBe(top);

    await h.pickerKeys(["k"]);
    const still = await h.waitDump((state) => true);
    expect(still.picker?.selected).toBe(top);
    await h.hidePicker();
  });

  it("gg で先頭、G で末尾", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["G"]);
    const last = await h.waitDump((state) => {
      const rows = state.picker?.filtered ?? [];
      return rows.length > 0 && state.picker?.selected === rows[rows.length - 1];
    });
    const rows = last.picker?.filtered ?? [];
    expect(last.picker?.selected).toBe(rows[rows.length - 1]);
    await h.pickerKeys(["g", "g"]);
    const first = await h.waitDump((state) => state.picker?.selected === rows[0]);
    expect(first.picker?.selected).toBe(rows[0]);
    await h.hidePicker();
  });

  it("Ctrl+D / Ctrl+U で半ページ動き、端で止まる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["g", "g"]);
    const start = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) > 0);
    const top = start.picker?.filtered?.[0];
    const bottom = start.picker?.filtered?.[start.picker.filtered.length - 1];
    await h.pickerKeys(["Control", "d", "Control"]);
    const paged = await h.waitDump((state) => state.picker?.selected !== top);
    expect(paged.picker?.selected).not.toBe(top);
    for (let i = 0; i < 20; i += 1) {
      await h.pickerKeys(["Control", "d", "Control"]);
    }
    const end = await h.waitDump((state) => state.picker?.selected === bottom);
    expect(end.picker?.selected).toBe(bottom);
    for (let i = 0; i < 20; i += 1) {
      await h.pickerKeys(["Control", "u", "Control"]);
    }
    const home = await h.waitDump((state) => state.picker?.selected === top);
    expect(home.picker?.selected).toBe(top);
    await h.hidePicker();
  });

  it("/work で先頭が work の行だけ残り、Esc で絞りが外れ、もう一度で閉じる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("work");
    const filtered = await h.waitDump((state) =>
      (state.picker?.filtered ?? []).every((text) => text.startsWith("work")),
    );
    expect(filtered.picker?.filtered.every((text) => text.startsWith("work"))).toBe(true);
    expect(filtered.picker?.filtered.length).toBeGreaterThan(0);
    await h.pickerKeys(["Escape"]);
    const cleared = await h.waitDump(
      (state) => state.picker?.open === true && (state.picker.filtered?.length ?? 0) > 2,
    );
    expect(cleared.picker?.query).toBe("");
    await h.pickerKeys(["Escape"]);
    await h.waitDump((state) => state.picker?.open === false);
  });

  it("検索中も Ctrl+N / Ctrl+P で移動できる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("work");
    const start = await h.waitDump((state) => (state.picker?.filtered?.length ?? 0) >= 2);
    const first = start.picker?.filtered?.[0];
    const second = start.picker?.filtered?.[1];
    await h.pickerKeys(["Control", "n", "Control"]);
    const next = await h.waitDump((state) => state.picker?.selected === second);
    expect(next.picker?.selected).toBe(second);
    await h.pickerKeys(["Control", "p", "Control"]);
    const prev = await h.waitDump((state) => state.picker?.selected === first);
    expect(prev.picker?.selected).toBe(first);
    await h.hidePicker();
  });

  it("Tab でよく使うタグの絞りが切り替わる。検索中の Tab は一覧へ戻る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["Tab"]);
    const tagged = await h.waitDump((state) => (state.picker?.query ?? "").includes("#"));
    expect(tagged.picker?.query).toMatch(/^#work /);
    expect(tagged.picker?.filtered.every((text) => text.startsWith("work"))).toBe(true);
    await h.pickerKeys(["Escape"]);
    await h.search("a");
    const searching = await h.waitDump((state) => state.picker?.mode === "search");
    expect(searching.picker?.mode).toBe("search");
    await h.pickerKeys(["Tab"]);
    const back = await h.waitDump((state) => state.picker?.mode === "normal");
    expect(back.picker?.mode).toBe("normal");
    expect(back.picker?.query).toBe("a");
    await h.hidePicker();
  });

  it("f のあと1文字でその文字へ行き、; で次、, で前", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["G"]);
    await h.waitDump((state) => {
      const rows = state.picker?.filtered ?? [];
      return rows.length > 0 && state.picker?.selected === rows[rows.length - 1];
    });
    // いまの行の次から探す。末尾からなら先頭の apple に当たる。
    await h.pickerKeys(["f", "a"]);
    const first = await h.waitDump((state) => state.picker?.selected === "apple");
    expect(first.picker?.selected).toBe("apple");
    await h.pickerKeys([";"]);
    const second = await h.waitDump((state) => state.picker?.selected === "apricot");
    expect(second.picker?.selected).toBe("apricot");
    await h.pickerKeys([","]);
    const back = await h.waitDump((state) => state.picker?.selected === "apple");
    expect(back.picker?.selected).toBe("apple");
    await h.hidePicker();
  });

  it("a でいまの貼り付け先の行だけになり、もう一度で戻る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("apple");
    await h.waitDump((state) => state.picker?.selected === "apple");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(await h.noteText()).toContain("apple");
    await h.showPicker();
    await h.pickerKeys(["a"]);
    const only = await h.waitDump((state) => state.picker?.contextOnly === true);
    expect(only.picker?.filtered).toEqual(["apple"]);
    await h.pickerKeys(["a"]);
    const all = await h.waitDump((state) => state.picker?.contextOnly === false);
    expect((all.picker?.filtered?.length ?? 0) > 1).toBe(true);
    await h.hidePicker();
  });

  it("g e で選択行の全文が出て、もう一度で閉じる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.search("apple");
    await h.waitDump((state) => state.picker?.selected === "apple");
    await h.pickerKeys(["Escape"]);
    await h.pickerKeys(["g", "e"]);
    const open = await h.waitDump((state) => state.picker?.preview === "apple");
    expect(open.picker?.preview).toBe("apple");
    await h.pickerKeys(["g", "e"]);
    const closed = await h.waitDump((state) => state.picker?.preview === "");
    expect(closed.picker?.preview).toBe("");
    await h.hidePicker();
  });

  it("g ? で貼り付け先とタグが出て、行を変えるとその行の情報になる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["G"]);
    await h.waitDump((state) => {
      const rows = state.picker?.filtered ?? [];
      return rows.length > 1 && state.picker?.selected === rows[rows.length - 1];
    });
    await h.pickerKeys(["k"]);
    await h.waitDump((state) => state.picker?.selected === "work-note");
    await h.pickerKeys(["g", "?"]);
    const info = await h.waitDump((state) => (state.picker?.info ?? "").includes("#work"));
    expect(info.picker?.info).toContain("#work");
    await h.pickerKeys(["j"]);
    const next = await h.waitDump((state) => state.picker?.selected !== "work-note");
    expect(next.picker?.info.length).toBeGreaterThan(0);
    expect(next.picker?.selected).not.toBe("work-note");
    await h.hidePicker();
  });

  it("g のあと 400ms 待つと次のキーが出て、すぐ次を押せば出ない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.pickerKeys(["g"]);
    await sleep(500);
    const shown = await h.waitDump((state) => (state.picker?.which ?? []).includes("e"));
    expect(shown.picker?.which).toContain("e");
    await h.pickerKeys(["Escape"]);
    await h.pickerKeys(["g", "e"]);
    const skipped = await h.waitDump((state) => state.picker?.preview === "apple" || state.picker?.preview !== "");
    expect(skipped.picker?.which ?? []).not.toContain("e");
    await h.hidePicker();
  });
});

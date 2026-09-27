// TEST.md「パイプ」

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { plain } from "./text";
import { sleep } from "./dump";

describeE2e("パイプ（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "HelloWorld" },
        { text: "foo_bar" },
        { text: "keep-me" },
        { text: "alpha" },
        { text: "bravo" },
        { text: "hit line\nskip me" },
        { text: "other" },
        { text: ":sh echo hi | quote" },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it(":upper は大文字、:lower は小文字", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("HelloWorld");
    await h.colon("upper");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("HELLOWORLD");

    await h.clearNote();
    await h.revealRow("HelloWorld");
    await h.colon("lower");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("helloworld");
  });

  it(":camel :pascal :snake :kebab は名前の区切りを組み直す", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("foo_bar");
    await h.colon("camel");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("fooBar");

    await h.clearNote();
    await h.revealRow("foo_bar");
    await h.colon("pascal");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("FooBar");

    await h.clearNote();
    await h.revealRow("HelloWorld");
    await h.colon("snake");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("hello_world");

    await h.clearNote();
    await h.revealRow("HelloWorld");
    await h.colon("kebab");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("hello-world");
  });

  it(":echo 255 | hex は 0xff。:echo 0xff | bin は 0b11111111。読めないときは何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 255 | hex");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("0xff");

    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 0xff | bin");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("0b11111111");

    await h.clearNote();
    await h.showPicker();
    await h.colon("echo nope | hex");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });

  it(":echo 3+4 | clip は前面に貼らずクリップボードが 7", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 3+4 | clip");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
    await h.notepad.keys(["Control", "v", "Control"]);
    await sleep(200);
    expect(plain(await h.noteText())).toBe("7");
  });

  it(":echo 3+4 | add は一覧に 7 が1件。クリップボードは変わらない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("KEEPCLIP"));
    await h.selectCopyClear();
    await h.clearNote();
    await h.showPicker();
    await h.colon("echo 3+4 | add");
    const dump = await h.waitDump((state) => state.items.some((item) => item.text === "7"));
    expect(dump.items.filter((item) => item.text === "7")).toHaveLength(1);
    await h.hidePicker();
    await h.notepad.keys(["Control", "v", "Control"]);
    await sleep(200);
    expect(plain(await h.noteText())).toBe("KEEPCLIP");
  });

  it(":echo 3+4 | show はヘルプに 7 が出る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("echo 3+4 | show");
    const help = await h.waitDump((state) => state.picker?.mode === "help");
    expect(help.picker?.mode).toBe("help");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
  });

  it("filter は含む行だけ、filter not は除く。当たらなければ何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("hit line\nskip me");
    await h.colon("filter hit");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("hit line");

    await h.clearNote();
    await h.revealRow("hit line\nskip me");
    await h.colon("filter not hit");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("skip me");

    await h.clearNote();
    await h.revealRow("keep-me");
    await h.colon("filter missing");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });

  it("先頭の . は選択行、先頭の clip はクリップボードが流れ", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("HelloWorld");
    await h.colon(". | lower");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("helloworld");

    await h.clearNote();
    await h.typeNote(chars("CLIPSRC"));
    await h.selectCopyClear();
    await h.clearNote();
    await h.showPicker();
    await h.colon("clip | lower");
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("clipsrc");
  });

  it("sel を段に書くとパイプ全体が何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("keep-me");
    await h.colon("sel | upper");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });

  it("1行の :sh echo hi | quote を Ctrl+8 すると引用付きになる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars(":sh echo hi | quote"));
    await h.notepad.keys(["Shift", "Home", "Shift"]);
    await h.expand();
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Enter"]);
    await sleep(800);
    expect(plain(await h.noteText()).trim()).toBe("> hi");
  });

  it("知らない段は何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("keep-me");
    await h.colon("nonesuch");
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
    await h.hidePicker();
  });
});

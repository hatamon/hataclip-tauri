// TEST.md「テンプレート」。履歴に書いて Enter で貼る。

import { afterAll, beforeAll, expect, it } from "vitest";
import { writeFileSync } from "node:fs";
import { containerPath, hostPath } from "./paths";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { DATE, TIME, UUID, plain } from "./text";
import { sleep } from "./dump";

const fileBody = "filebody";

describeE2e("テンプレート（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    writeFileSync(containerPath("tmpl.txt"), fileBody);
    h = await startHarness({
      history: [
        { text: "{{date}}" },
        { text: "{{time}}" },
        { text: "{{date-1d}}" },
        { text: "{{date+1m}}" },
        { text: "{{clip}}" },
        { text: "{{sel}}" },
        { text: "{{sel|clip}}" },
        { text: "{{ask:名前}}" },
        { text: "{{pick list: prod, stg}}" },
        { text: "{{app}}" },
        { text: "{{front}}" },
        { text: "{{when app: notepad}}yes{{when}}no" },
        { text: "{{when focus: Edit}}" },
        { text: "{{uuid}}" },
        { text: "{{user}}" },
        { text: "{{host}}" },
        { text: "{{var:a}}" },
        { text: "{{var:a:2}}" },
        { text: "{{tag:work}}" },
        { text: "id{{type:<Tab>}}pass" },
        { text: "pre{{wait:200}}post" },
        { text: "{{sh: echo hi}}", tags: ["run"] },
        { text: "confirm-me", tags: ["confirm"] },
        { text: hostPath("tmpl.txt").replace(/\\/g, "/"), tags: ["file"] },
        { text: "{{n}}" },
        { text: "tagged-work", tags: ["work"] },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("{{date}} は YYYY/MM/DD。{{time}} は HH:MM", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{date}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toMatch(DATE);

    await h.clearNote();
    await h.revealRow("{{time}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toMatch(TIME);
  });

  it("{{date-1d}} は昨日。{{date+1m}} は来月", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{date-1d}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const yesterday = plain(await h.noteText());
    expect(yesterday).toMatch(DATE);

    await h.clearNote();
    await h.revealRow("{{date+1m}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const next = plain(await h.noteText());
    expect(next).toMatch(DATE);
    expect(next).not.toBe(yesterday);
  });

  it("{{clip}} はいまのクリップボード。空なら空", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("clipval"));
    await h.selectCopyClear();
    await h.clearNote();
    await h.revealRow("{{clip}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("clipval");
  });

  it("{{sel}} と {{sel|clip}} はそのまま貼る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{sel}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("{{sel}}");

    await h.clearNote();
    await h.revealRow("{{sel|clip}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("{{sel|clip}}");
  });

  it("{{ask:名前}} は貼る前に聞く。同じ名前は1回だけ", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{ask:名前}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "ask");
    await h.pickerKeys([...chars("taro"), "Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("taro");
  });

  it("{{pick list: prod, stg}} は選んだ語が貼られる。Esc では貼らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{pick list: prod, stg}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "pick");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
    expect(plain(await h.noteText())).toBe("");

    await h.revealRow("{{pick list: prod, stg}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "pick");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("prod");
  });

  it("{{app}} と {{front}} は取れなければ空、取れれば文字がある", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{app}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).length).toBeGreaterThanOrEqual(0);

    await h.clearNote();
    await h.revealRow("{{front}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).length).toBeGreaterThanOrEqual(0);
  });

  it("{{when app: notepad}} はメモ帳なら yes", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{when app: notepad}}yes{{when}}no");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("yes");
  });

  it("{{when focus: Edit}} は枝にならず文字が残る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{when focus: Edit}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("{{when focus: Edit}}");
  });

  it("{{uuid}} {{user}} {{host}} が入る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{uuid}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toMatch(UUID);

    await h.clearNote();
    await h.revealRow("{{user}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).length).toBeGreaterThan(0);

    await h.clearNote();
    await h.revealRow("{{host}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).length).toBeGreaterThan(0);
  });

  it(":set a のあと {{var:a}} は今日の日付。set 時点では展開しない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.typeNote(chars("{{date}}"));
    await h.selectCopyClear();
    await h.showPicker();
    await h.pickerKeys([":", ..."set a=".split(""), "Control", "v", "Control", "Enter"]);
    await sleep(300);
    await h.colon("set");
    const listed = await h.waitDump((state) => state.picker?.mode === "help");
    expect(listed.picker?.mode).toBe("help");
    await h.pickerKeys(["Escape"]);
    await h.clearNote();
    await h.revealRow("{{var:a}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    await h.waitNote((text) => DATE.test(plain(text)));
  });

  it("{{var:a:2}} は整数なら0埋め、でなければ幅を無視", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.showPicker();
    await h.colon("set a=3");
    await sleep(200);
    await h.clearNote();
    await h.revealRow("{{var:a:2}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    await h.waitNote((text) => plain(text) === "03");

    await h.showPicker();
    await h.colon("set a=hello");
    await sleep(200);
    await h.clearNote();
    await h.revealRow("{{var:a:2}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    await h.waitNote((text) => plain(text) === "hello");
  });

  it("{{tag:work}} はタグ work の本文を改行でつなぐ", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{tag:work}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toContain("tagged-work");
  });

  it("id{{type:<Tab>}}pass は id、Tab、pass の順", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("id{{type:<Tab>}}pass");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    const text = await h.noteText();
    expect(plain(text).startsWith("id")).toBe(true);
    expect(plain(text).endsWith("pass")).toBe(true);
    expect(text).not.toContain("v");
  });

  it("{{wait:200}} の前後で待ってから続きが貼られる", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("pre{{wait:200}}post");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("prepost");
  });

  it("#run の {{sh: echo hi}} は確認のあと hi。Esc では貼らない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("{{sh: echo hi}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Escape"]);
    await h.hidePicker();
    expect(plain(await h.noteText())).toBe("");

    await h.revealRow("{{sh: echo hi}}");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText()).trim()).toBe("hi");
  });

  it("#confirm は展開後を出してから Enter で貼る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.revealRow("confirm-me");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.mode === "help");
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe("confirm-me");
  });

  it("#file は本文のパスのファイル内容を貼る", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    const path = hostPath("tmpl.txt").replace(/\\/g, "/");
    await h.revealRow(path);
    await h.pickerKeys(["Enter"]);
    await h.waitDump((state) => state.picker?.open === false);
    expect(plain(await h.noteText())).toBe(fileBody);
  });
});

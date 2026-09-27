// TEST.md「速貼」

import { afterAll, beforeAll, expect, it } from "vitest";
import { describeE2e, startHarness, stopHarness, type Harness } from "./harness";
import { plain } from "./text";
import { sleep } from "./dump";

describeE2e("速貼（TEST.md）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness({
      history: [
        { text: "first" },
        { text: "second" },
        { text: "third" },
        { text: "slotted", tags: ["slot:3"] },
        { text: ":sel | snake", tags: ["slot:1"] },
      ],
    });
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("#slot:3 の行は Ctrl+Shift+3。無ければ3番目", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.openNote();
    await h.notepad.keys(["Control", "Shift", "3", "Shift", "Control"]);
    await h.waitNote((text) => plain(text) === "slotted");
  });

  it("#slot:1 の本文が :sel | snake のとき Ctrl+Shift+1 は何もしない", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    await h.clearNote();
    await h.openNote();
    await h.notepad.keys(["Control", "Shift", "1", "Shift", "Control"]);
    await sleep(400);
    expect(plain(await h.noteText())).toBe("");
  });
});

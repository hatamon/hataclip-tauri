// TEST.md の「Ctrl」節、先頭の1件だけを自動化するサンプル。
//
//   メモ帳は2行にして、カーソルは2行目の末尾、選択はしない。
//     keep
//     {{date}}
//   - [ ] Ctrl を押したまま 4。`4` も `c` も入らない。2行目の `{{date}}` が登録される。1行目の `keep` はそのまま
//
// 「選択はしない」だが、登録は選択が無ければクリップボードを見る（capture_register_text）。
// 手で試す人は 2 行目を一度コピーしてから選択を外す、という手順を自然にたどるはず。
// この自動化もその手順をなぞり、クリップボードに `{{date}}` を積んでから選択を外す。

import { afterAll, beforeAll, expect, it } from "vitest";
import { chars, describeE2e, startHarness, stopHarness, type Harness } from "./harness";

describeE2e("Ctrl を押したまま 4（TEST.md の Ctrl 節、先頭）", () => {
  let h: Harness | undefined;

  beforeAll(async () => {
    h = await startHarness();
    await h.typeNote([...chars("keep"), "Enter", ...chars("{{date}}")]);
    await h.selectCopyClear();
  });

  afterAll(async () => {
    await stopHarness(h);
  });

  it("メモ帳に4もcも増えず、{{date}}が先頭に登録される", async () => {
    if (!h) {
      throw new Error("harness が無い");
    }
    const before = await h.noteText();
    await h.register();
    const after = await h.noteText();
    expect(after).toBe(before);
    expect(after).not.toContain("4");
    expect(after.split(/\r\n|\n|\r/)[0]).toBe("keep");

    const dump = await h.waitDump((state) => state.items.length > 0);
    expect(dump.items[0]?.text).toBe("{{date}}");
  });
});

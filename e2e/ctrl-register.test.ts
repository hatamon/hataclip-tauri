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
//
// WinAppDriver はホストにしか居ないので、WINAPPDRIVER_HOST が無ければこのファイルは skip する。
// npm test（ユニットテスト）には含めない。

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { closeNotepad, closeSession, launchHataclip, launchNotepad, type Session } from "./driver";
import { containerPath, hataclipGuiPath, hostPath, winAppDriverHost } from "./paths";
import { waitForDump } from "./dump";

const hasHost = Boolean(winAppDriverHost());
const describeIfHost = hasHost ? describe : describe.skip;

if (!hasHost) {
  console.warn(
    "WINAPPDRIVER_HOST が無いので e2e/ctrl-register.test.ts は skip する。" +
      "ホストで WinAppDriver を起動し、e2e.env を --env-file で渡すと動く。",
  );
}

describeIfHost("Ctrl を押したまま 4（TEST.md の Ctrl 節、先頭）", () => {
  let notepad: Session | undefined;
  let hataclip: Session | undefined;
  // describe.skip でもこの関数本体は評価されるので、環境変数を読む処理は
  // beforeAll の中まで遅らせる（hasHost が偽のときは beforeAll 自体が動かない）。
  let dumpContainerPath: string;
  let dumpHostPath: string;
  let noteName = "";

  beforeAll(async () => {
    dumpContainerPath = containerPath("state.json");
    dumpHostPath = hostPath("state.json");
    const testDirHostPath = hostPath("data");
    const testDirContainerPath = containerPath("data");

    // マウントのルート自体は消せない（EBUSY）。前回分の中身だけ掃除する。
    rmSync(dumpContainerPath, { force: true });
    rmSync(testDirContainerPath, { recursive: true, force: true });
    mkdirSync(testDirContainerPath, { recursive: true });
    noteName = `hataclip-e2e-${Date.now()}.txt`;
    writeFileSync(containerPath(noteName), "");

    notepad = await launchNotepad(hostPath(noteName));
    await notepad.openTab(noteName);
    await notepad.keys([
      "k",
      "e",
      "e",
      "p",
      "Enter",
      "{",
      "{",
      "d",
      "a",
      "t",
      "e",
      "}",
      "}",
    ]);
    // 2行目（{{date}}）を選択してコピーし、クリップボードへ積む。そのあと選択を外す。
    await notepad.keys(["Shift", "Home", "Shift"]);
    await notepad.keys(["Control", "c", "Control"]);
    await notepad.keys(["End"]);

    const gui = await launchHataclip(hataclipGuiPath(), [
      "--test-dir",
      testDirHostPath,
      "--test-reset-db",
      "--test-dump-state",
      dumpHostPath,
    ]);
    hataclip = gui;
    const started = await waitForDump(dumpContainerPath, (dump) => dump.ready);
    if (started.shortcuts !== "ok") {
      throw new Error(
        "ショートカットを取れない。普段使っている hataclip-gui を終了してから再実行する",
      );
    }
    // セッション作成のため一度見せたウィンドウが前面だと、登録の Ctrl+C が一覧に当たる。
    await gui.minimize();

    // グローバルショートカットは前面のウィンドウで発火する。テスト用タブを前面へ戻す。
    await notepad.openTab(noteName);
  });

  afterAll(async () => {
    await closeNotepad(notepad, noteName);
    await closeSession(hataclip);
  });

  it("メモ帳に4もcも増えず、{{date}}が先頭に登録される", async () => {
    if (!notepad) {
      throw new Error("notepad セッションが無い");
    }
    const edit = await notepad.openTab(noteName);
    const before = await edit.getText();

    await notepad.keys(["Control", "4", "Control"]);
    await new Promise((resolve) => setTimeout(resolve, 300));

    const after = await edit.getText();
    expect(after).toBe(before);
    expect(after).not.toContain("4");
    expect(after.split(/\r\n|\n|\r/)[0]).toBe("keep");

    const dump = await waitForDump(dumpContainerPath, (state) => state.items.length > 0);
    expect(dump.items[0]?.text).toBe("{{date}}");
  });
});

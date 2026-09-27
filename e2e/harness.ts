// 節ごとのテストが同じ起動・掃除を繰り返さないようにする。
// describe.skip でも本体は評価されるので、環境変数を読む処理は startHarness の中だけ。

import { mkdirSync, readdirSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { describe } from "vitest";
import { closeNotepad, closeSession, launchHataclip, launchNotepad, type Session } from "./driver";
import { quitHataclipNvim } from "./nvim";
import { containerPath, hataclipGuiPath, hostPath, winAppDriverHost } from "./paths";
import { sleep, waitForDump, type DumpState } from "./dump";

const NOTE_NAME = "hataclip-e2e.txt";

export type HistoryRow = {
  text: string;
  tags?: string[];
  contexts?: string[];
  pinned?: boolean;
};

export type Harness = {
  notepad: Session;
  hataclip: Session;
  noteName: string;
  dumpPath: string;
  testDirHost: string;
  testDirContainer: string;
  openNote: () => Promise<void>;
  typeNote: (keys: string[]) => Promise<void>;
  clearNote: () => Promise<void>;
  noteText: () => Promise<string>;
  selectCopyClear: () => Promise<void>;
  register: () => Promise<void>;
  showPicker: () => Promise<DumpState>;
  hidePicker: () => Promise<void>;
  search: (query: string) => Promise<DumpState>;
  pick: (query: string, selected: string) => Promise<void>;
  revealRow: (selected: string) => Promise<void>;
  colon: (line: string) => Promise<void>;
  expand: () => Promise<void>;
  pickerKeys: (keys: string[]) => Promise<void>;
  waitDump: (predicate: (dump: DumpState) => boolean, timeoutMs?: number) => Promise<DumpState>;
  waitNote: (predicate: (text: string) => boolean, timeoutMs?: number) => Promise<string>;
  quitNvim: () => Promise<void>;
};

export function describeE2e(name: string, body: () => void): void {
  const hasHost = Boolean(winAppDriverHost());
  if (!hasHost) {
    console.warn(
      `WINAPPDRIVER_HOST が無いので ${name} は skip する。` +
        "ホストで WinAppDriver を起動し、e2e.env を --env-file で渡すと動く。",
    );
  }
  (hasHost ? describe : describe.skip)(name, body);
}

export function chars(text: string): string[] {
  return [...text];
}

export async function startHarness(options?: {
  history?: HistoryRow[];
}): Promise<Harness> {
  const dumpPath = containerPath("state.json");
  const dumpHostPath = hostPath("state.json");
  const testDirHost = hostPath("data");
  const testDirContainer = containerPath("data");
  const seedContainer = containerPath("data", "seed.json");
  const seedHost = hostPath("data", "seed.json");

  rmSync(dumpPath, { force: true });
  rmSync(testDirContainer, { recursive: true, force: true });
  mkdirSync(testDirContainer, { recursive: true });
  removeOldNoteFiles();

  const args = ["--test-dir", testDirHost, "--test-reset-db", "--test-dump-state", dumpHostPath];
  if (options?.history) {
    writeFileSync(
      seedContainer,
      JSON.stringify({ version: 1, items: options.history }, null, 2),
    );
    args.splice(3, 0, "--test-inject-history", seedHost);
  }

  const noteName = NOTE_NAME;
  writeFileSync(containerPath(noteName), "");

  const notepad = await launchNotepad(hostPath(noteName));
  await notepad.openTab(noteName);

  const hataclip = await launchHataclip(hataclipGuiPath(), args);
  const started = await waitForDump(dumpPath, (dump) => dump.ready);
  if (started.shortcuts !== "ok") {
    throw new Error("ショートカットを取れない。普段使っている hataclip-gui を終了してから再実行する");
  }
  if (!started.picker) {
    throw new Error(
      "dump に picker が無い。debug の exe は使えない。HATACLIP_GUI を dist\\windows\\hataclip-gui.exe にする",
    );
  }
  await hataclip.minimize();
  await notepad.openTab(noteName);

  const harness: Harness = {
    notepad,
    hataclip,
    noteName,
    dumpPath,
    testDirHost,
    testDirContainer,
    async openNote() {
      await notepad.openTab(noteName);
    },
    async typeNote(keys) {
      await notepad.openTab(noteName);
      await notepad.keys(keys);
    },
    async clearNote() {
      await notepad.openTab(noteName);
      await notepad.keys(["Control", "a", "Control"]);
      await notepad.keys(["Delete"]);
    },
    async noteText() {
      const edit = await notepad.openTab(noteName);
      return edit.getText();
    },
    async selectCopyClear() {
      await notepad.openTab(noteName);
      await notepad.keys(["Shift", "Home", "Shift"]);
      await notepad.keys(["Control", "c", "Control"]);
      await notepad.keys(["End"]);
    },
    async register() {
      await notepad.openTab(noteName);
      await notepad.keys(["Control", "4", "Control"]);
      await sleep(300);
    },
    async showPicker() {
      await notepad.openTab(noteName);
      await sleep(250);
      await notepad.keys(["Control", "7", "Control"]);
      return waitForDump(dumpPath, (dump) => dump.picker?.open === true);
    },
    async search(query) {
      const now = await waitForDump(dumpPath, () => true, 2_000).catch(() => undefined);
      if (!now?.picker?.open) {
        await harness.showPicker();
      }
      await hataclip.keys(["/", ...chars(query)]);
      return waitForDump(dumpPath, (dump) => {
        if (!dump.picker?.open) {
          return false;
        }
        if (query.length === 0) {
          return dump.picker.mode === "search";
        }
        return (dump.picker.query ?? "").includes(query);
      });
    },
    async hidePicker() {
      for (let i = 0; i < 3; i += 1) {
        const dump = await waitForDump(dumpPath, () => true, 2_000).catch(() => undefined);
        if (!dump?.picker?.open) {
          break;
        }
        await hataclip.keys(["Escape"]);
        await sleep(200);
      }
      await waitForDump(dumpPath, (next) => next.picker?.open === false);
      await notepad.openTab(noteName);
    },
    async pick(query, selected) {
      await harness.showPicker();
      await harness.search(query);
      await waitForDump(dumpPath, (dump) => dump.picker?.selected === selected);
      await hataclip.keys(["Tab"]);
      await waitForDump(
        dumpPath,
        (dump) => dump.picker?.mode === "normal" && dump.picker.selected === selected,
      );
    },
    async revealRow(selected) {
      await harness.showPicker();
      await hataclip.keys(["g", "g"]);
      const list = await waitForDump(dumpPath, (dump) =>
        (dump.picker?.filtered ?? []).includes(selected),
      );
      const at = list.picker?.filtered.indexOf(selected) ?? -1;
      for (let i = 0; i < at; i += 1) {
        await hataclip.keys(["j"]);
      }
      await waitForDump(dumpPath, (dump) => dump.picker?.selected === selected);
    },
    async colon(line) {
      await hataclip.keys([":"]);
      await waitForDump(dumpPath, (dump) => dump.picker?.mode === "colon");
      if (line.length > 0) {
        await hataclip.keys([...chars(line)]);
      }
      await hataclip.keys(["Enter"]);
    },
    async expand() {
      await notepad.openTab(noteName);
      await sleep(400);
      await notepad.keys(["Control", "8", "Control"]);
      await sleep(500);
    },
    async waitNote(predicate, timeoutMs = 8_000) {
      const deadline = Date.now() + timeoutMs;
      let last = "";
      while (Date.now() < deadline) {
        last = await harness.noteText();
        if (predicate(last)) {
          return last;
        }
        await sleep(100);
      }
      throw new Error(`時間切れ: メモ帳が条件を満たさない。最後: ${JSON.stringify(last)}`);
    },
    async pickerKeys(keys) {
      await hataclip.keys(keys);
    },
    waitDump(predicate, timeoutMs) {
      return waitForDump(dumpPath, predicate, timeoutMs);
    },
    quitNvim() {
      return quitHataclipNvim();
    },
  };
  return harness;
}

export async function stopHarness(harness: Harness | undefined): Promise<void> {
  if (!harness) {
    return;
  }
  try {
    await harness.clearNote();
    writeFileSync(containerPath(harness.noteName), "");
    await harness.notepad.keys(["Control", "s", "Control"]);
    await sleep(200);
  } catch {
    // タブが無いときは、そのまま閉じる。
  }
  await closeNotepad(harness.notepad, harness.noteName);
  await closeSession(harness.hataclip);
}

function removeOldNoteFiles(): void {
  const root = containerPath();
  for (const name of readdirSync(root)) {
    if (/^hataclip-e2e-\d+\.txt$/.test(name)) {
      unlinkSync(containerPath(name));
    }
  }
}

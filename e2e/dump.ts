// `--test-dump-state` が書く JSON を読む。exe が書くタイミングと Vitest が読むタイミングが
// ずれるので、条件が満たされるまで短く待ち直す。

import { existsSync, readFileSync } from "node:fs";

export type DumpItem = {
  text: string;
  tags: string[];
};

export type DumpPicker = {
  open: boolean;
  mode: string;
  query: string;
  selected: string | null;
  visible: string[];
  filtered: string[];
  preview: string;
  info: string;
  which: string[];
  contextOnly: boolean;
};

export type DumpState = {
  ready: boolean;
  shortcuts: "ok" | "failed";
  items: DumpItem[];
  clipboard?: string;
  picker?: DumpPicker;
  lastOpen?: string | null;
};

export function readDump(path: string): DumpState | undefined {
  if (!existsSync(path)) {
    return undefined;
  }
  try {
    return JSON.parse(readFileSync(path, "utf8")) as DumpState;
  } catch {
    // 書き込みの途中を読んでしまったときは、次のポーリングに任せる。
    return undefined;
  }
}

export async function waitForDump(
  path: string,
  predicate: (dump: DumpState) => boolean,
  timeoutMs = 10_000,
): Promise<DumpState> {
  const deadline = Date.now() + timeoutMs;
  let last = "";
  while (Date.now() < deadline) {
    const dump = readDump(path);
    if (dump && predicate(dump)) {
      return dump;
    }
    if (dump) {
      last = JSON.stringify(dump);
    }
    await sleep(100);
  }
  const detail = last ? ` 最後: ${last}` : " ファイルが読めない";
  throw new Error(`時間切れ: ${path} が条件を満たさない。${detail}`);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ホストで動く WinAppDriver への窓口。パターン1（Docker のクライアント → ホストの
// WinAppDriver → ホストの exe）はこのファイルだけが知っている。
//
// WinAppDriver は JSON Wire の POST /session だけを受ける。/wd/hub は 404。

import { winAppDriverHost } from "./paths";

const PORT = 4723;

const KEY: Record<string, string> = {
  Control: "\uE009",
  Shift: "\uE008",
  Alt: "\uE00A",
  Enter: "\uE007",
  Home: "\uE011",
  End: "\uE010",
};

/** 押したままの修飾キーを全部離す。WinAppDriver はこれを送るまで Shift / Ctrl を保持する。 */
const RELEASE_MODIFIERS = "\uE000";

type WireValue = {
  sessionId?: string;
  status?: number;
  value?: unknown;
};

export class Element {
  constructor(
    private readonly session: Session,
    readonly id: string,
  ) {}

  async click(): Promise<void> {
    await this.session.post(`/element/${encodeURIComponent(this.id)}/click`, {});
  }

  async getText(): Promise<string> {
    const body = await this.session.get(`/element/${encodeURIComponent(this.id)}/text`);
    return typeof body.value === "string" ? body.value : "";
  }
}

export class Session {
  constructor(
    readonly id: string,
    private readonly base: string,
  ) {}

  async findDisplayedByClass(className: string): Promise<Element> {
    const body = await this.post("/elements", { using: "class name", value: className });
    const list = elementIds(body.value);
    for (const id of list) {
      const shown = await this.get(`/element/${encodeURIComponent(id)}/displayed`);
      if (shown.value === true) {
        return new Element(this, id);
      }
    }
    throw new Error(`${className} が見つからない`);
  }

  async clickNamed(name: string): Promise<void> {
    const body = await this.post("/elements", { using: "name", value: name });
    const list = elementIds(body.value);
    const id = list[0];
    if (!id) {
      throw new Error(`${name} が見つからない`);
    }
    await new Element(this, id).click();
  }

  /** 復元された他のタブではなく、このファイルのタブを前面にして本文欄を返す。 */
  async openTab(fileName: string): Promise<Element> {
    const deadline = Date.now() + 5_000;
    let last: unknown;
    while (Date.now() < deadline) {
      try {
        await this.clickNamed(fileName);
        const edit = await this.findDisplayedByClass("RichEditD2DPT");
        await edit.click();
        return edit;
      } catch (error) {
        last = error;
        await new Promise((resolve) => setTimeout(resolve, 200));
      }
    }
    throw last instanceof Error ? last : new Error(`${fileName} を開けない`);
  }

  async findByName(name: string): Promise<Element> {
    const body = await this.post("/element", { using: "name", value: name });
    const id = elementIds(body.value)[0];
    if (!id) {
      throw new Error(`${name} が見つからない`);
    }
    return new Element(this, id);
  }

  async keys(sequence: string[]): Promise<void> {
    const value = sequence.map((key) => KEY[key] ?? key);
    value.push(RELEASE_MODIFIERS);
    await this.post("/keys", { value });
  }

  async releaseModifiers(): Promise<void> {
    await this.post("/keys", { value: [RELEASE_MODIFIERS] });
  }

  /** テスト用に見せたウィンドウを前面から外す。前面のままだと登録のコピーがそっちに当たる。 */
  async minimize(): Promise<void> {
    try {
      await this.post("/window/minimize", {});
      return;
    } catch {
      // WinAppDriver 1.2 には minimize が無い。
    }
    try {
      await this.post("/window/position", { x: -20000, y: -20000 });
    } catch {
      // 外せなくても、あとの openTab でメモ帳を前面にする。
    }
  }

  async deleteSession(): Promise<void> {
    await fetch(`${this.base}/session/${this.id}`, { method: "DELETE" }).catch(() => undefined);
  }

  async post(path: string, payload: unknown): Promise<WireValue> {
    return request("POST", `${this.base}/session/${this.id}${path}`, payload);
  }

  async get(path: string): Promise<WireValue> {
    return request("GET", `${this.base}/session/${this.id}${path}`);
  }
}

function elementIds(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value.flatMap((item) => elementIds(item));
  }
  if (value && typeof value === "object" && "ELEMENT" in value) {
    const id = (value as { ELEMENT?: unknown }).ELEMENT;
    return typeof id === "string" ? [id] : [];
  }
  return [];
}

function baseUrl(): string {
  const host = winAppDriverHost() ?? "host.docker.internal";
  return `http://${host}:${PORT}`;
}

async function request(method: string, url: string, payload?: unknown): Promise<WireValue> {
  const response = await fetch(url, {
    method,
    headers: payload === undefined ? undefined : { "Content-Type": "application/json" },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  let body: WireValue = {};
  if (text) {
    try {
      body = JSON.parse(text) as WireValue;
    } catch {
      body = { value: text };
    }
  }
  if (!response.ok || (typeof body.status === "number" && body.status !== 0)) {
    throw new Error(`${method} ${url} が失敗: ${response.status} ${text}`);
  }
  return body;
}

async function start(capabilities: Record<string, string>): Promise<Session> {
  const base = baseUrl();
  const body = await request("POST", `${base}/session`, { desiredCapabilities: capabilities });
  if (!body.sessionId) {
    throw new Error("セッション ID が無い");
  }
  return new Session(body.sessionId, base);
}

/** 指定ファイルをメモ帳で開く。復元された他のタブとは別に、このファイルのタブができる。 */
export function launchNotepad(file: string): Promise<Session> {
  return start({
    app: "C:\\Windows\\System32\\notepad.exe",
    appArguments: quoteArg(file),
    deviceName: "WindowsPC",
    platformName: "Windows",
  });
}

/** hataclip-gui.exe をテスト用フラグ付きで開く。 */
export function launchHataclip(exePath: string, args: string[]): Promise<Session> {
  return start({
    app: exePath,
    appArguments: args.map(quoteArg).join(" "),
    deviceName: "WindowsPC",
    platformName: "Windows",
  });
}

function quoteArg(arg: string): string {
  return arg.includes(" ") ? `"${arg}"` : arg;
}

/** テスト用ファイルのタブだけ閉じる。未保存なら「保存しない」。他のタブは閉じない。 */
export async function closeNotepad(session: Session | undefined, fileName: string): Promise<void> {
  if (!session) {
    return;
  }
  try {
    await session.releaseModifiers();
    await session.clickNamed(fileName);
    await session.keys(["Control", "w"]);
    await new Promise((resolve) => setTimeout(resolve, 400));
    const discard = await session.findByName("保存しない");
    await discard.click();
  } catch {
    // タブが無い、または変更が無ければ確認は出ない。
  }
  try {
    await session.releaseModifiers();
  } catch {
    // セッションが既に閉じていても、修飾キーは上げておく。
  }
  await session.deleteSession();
}

/** 落ちても後続には影響させない。無関係な既存プロセスは触らない。 */
export async function closeSession(session: Session | undefined): Promise<void> {
  await session?.deleteSession();
}

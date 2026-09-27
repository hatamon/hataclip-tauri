// TEST.md「コマンドライン」。ホストの hataclip.exe を cmd で一度走らせる。
// Windows の hataclip.exe は引数無しでは一覧を出さず終了コード 1（一覧は gui）。

import { afterAll, expect, it } from "vitest";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { describeE2e } from "./harness";
import { runCli } from "./cli-run";
import { closeSession, launchCmd } from "./driver";
import { containerPath, hostPath } from "./paths";
import { sleep } from "./dump";

describeE2e("コマンドライン（TEST.md）", () => {
  afterAll(() => {
    rmSync(containerPath("cli-appdata"), { recursive: true, force: true });
  });

  it('hataclip.exe "echo 3+4" は標準出力に 7。ウィンドウは開かない', async () => {
    const result = await runCli(["echo 3+4"]);
    expect(result.code).toBe(0);
    expect(result.stdout.trim()).toBe("7");
  });

  it("壊れた式は何も出さず終了コード 1。--showerror のときだけ理由が出る", async () => {
    const silent = await runCli(["nonesuch"]);
    expect(silent.code).toBe(1);
    expect(silent.stdout.trim()).toBe("");
    expect(silent.stderr.trim()).toBe("");

    const shown = await runCli(["--showerror", "nonesuch"]);
    expect(shown.code).toBe(1);
    expect(shown.stdout.trim()).toBe("");
    expect(shown.stderr.trim().length).toBeGreaterThan(0);
  });

  it("--help は一覧の :help と同じ文章", async () => {
    const result = await runCli(["--help"]);
    expect(result.code).toBe(0);
    expect(result.stdout).toContain("引数なしの :help は目次");
    expect(result.stdout).toContain("echo");
  });

  it("hataclip.exe add は標準入力を一覧へ足す。clip はクリップボードへ書く", async () => {
    const added = await runCli(["add"], { stdin: "cli-add-row\n" });
    expect(added.code).toBe(0);
    const itemsPath = containerPath("cli-appdata", "com.hataclip.app", "items.json");
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline && !existsSync(itemsPath)) {
      await sleep(100);
    }
    expect(existsSync(itemsPath)).toBe(true);
    expect(readFileSync(itemsPath, "utf8")).toContain("cli-add-row");

    const clipped = await runCli(["clip"], { stdin: "cli-clip-row" });
    expect(clipped.code).toBe(0);
    const clipOut = containerPath("cli", "clipboard.txt");
    rmSync(clipOut, { force: true });
    const script = [
      "@echo off",
      `powershell -NoProfile -Command "Get-Clipboard | Set-Content -Encoding utf8 ${hostPath("cli", "clipboard.txt")}"`,
    ].join("\r\n");
    const clipScript = containerPath("cli", "clip.cmd");
    writeFileSync(clipScript, `${script}\r\n`);
    const session = await launchCmd(hostPath("cli", "clip.cmd"));
    const waitUntil = Date.now() + 8_000;
    while (Date.now() < waitUntil && !existsSync(clipOut)) {
      await sleep(100);
    }
    await closeSession(session);
    expect(readFileSync(clipOut, "utf8")).toContain("cli-clip-row");
  });

  it("引数が無いときは Windows では終了コード 1（一覧は hataclip-gui）", async () => {
    const result = await runCli([]);
    expect(result.code).toBe(1);
    expect(result.stderr).toContain("式を1つ渡す");
  });
});

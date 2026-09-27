// ホストの hataclip.exe を cmd 経由で一度走らせ、標準入出力を e2e-workspace に残す。
// APPDATA を作業フォルダへ向けるので、本番の一覧は触らない。

import { mkdirSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { closeSession, launchCmd } from "./driver";
import { containerPath, hataclipCliPath, hostPath } from "./paths";
import { sleep } from "./dump";

export type CliResult = {
  code: number;
  stdout: string;
  stderr: string;
};

export async function runCli(
  args: string[],
  options?: { stdin?: string },
): Promise<CliResult> {
  const dir = containerPath("cli");
  mkdirSync(dir, { recursive: true });
  mkdirSync(containerPath("cli-appdata"), { recursive: true });
  const stdoutPath = containerPath("cli", "stdout.txt");
  const stderrPath = containerPath("cli", "stderr.txt");
  const codePath = containerPath("cli", "code.txt");
  const stdinPath = containerPath("cli", "stdin.txt");
  const scriptPath = containerPath("cli", "run.cmd");
  rmSync(stdoutPath, { force: true });
  rmSync(stderrPath, { force: true });
  rmSync(codePath, { force: true });

  if (options?.stdin !== undefined) {
    writeFileSync(stdinPath, options.stdin);
  }

  const exe = hataclipCliPath();
  const quoted = args.map(winQuote).join(" ");
  const appdata = hostPath("cli-appdata");
  const redirectIn =
    options?.stdin !== undefined ? ` < ${winQuote(hostPath("cli", "stdin.txt"))}` : "";
  const lines = [
    "@echo off",
    `set APPDATA=${appdata}`,
    `${winQuote(exe)} ${quoted}${redirectIn} > ${winQuote(hostPath("cli", "stdout.txt"))} 2> ${winQuote(hostPath("cli", "stderr.txt"))}`,
    `echo %ERRORLEVEL% > ${winQuote(hostPath("cli", "code.txt"))}`,
  ];
  writeFileSync(scriptPath, `${lines.join("\r\n")}\r\n`);

  const session = await launchCmd(hostPath("cli", "run.cmd"));
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline && !existsSync(codePath)) {
    await sleep(100);
  }
  await closeSession(session);
  if (!existsSync(codePath)) {
    throw new Error("hataclip.exe の終了コードが書けない");
  }
  return {
    code: Number.parseInt(readFileSync(codePath, "utf8").trim(), 10),
    stdout: existsSync(stdoutPath) ? readFileSync(stdoutPath, "utf8") : "",
    stderr: existsSync(stderrPath) ? readFileSync(stderrPath, "utf8") : "",
  };
}

function winQuote(value: string): string {
  if (!/[ \t"]/.test(value)) {
    return value;
  }
  return `"${value.replace(/"/g, '\\"')}"`;
}

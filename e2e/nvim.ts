// hataclip が開いた nvim だけを :q、だめなら :wq で閉じる。
// コンソールの nvim は WinAppDriver が取れないので、ホストの PowerShell から送る。
// コマンドラインに hataclip- / e2e-workspace が無い nvim は触らない。

import { mkdirSync, writeFileSync } from "node:fs";
import { closeSession, launchCmd } from "./driver";
import { containerPath, hostPath } from "./paths";
import { sleep } from "./dump";

export async function quitHataclipNvim(): Promise<void> {
  mkdirSync(containerPath("nvim"), { recursive: true });
  const script = containerPath("nvim", "quit.ps1");
  writeFileSync(script, QUIT_PS1.replace(/\n/g, "\r\n"));
  const session = await launchCmd(
    `powershell.exe -NoProfile -ExecutionPolicy Bypass -File ${hostPath("nvim", "quit.ps1")}`,
  );
  await sleep(800);
  await closeSession(session);
}

const QUIT_PS1 = `
$ErrorActionPreference = 'Continue'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName Microsoft.VisualBasic

function Get-HataclipNvim {
  Get-CimInstance Win32_Process -Filter "Name = 'nvim.exe'" |
    Where-Object { $_.CommandLine -match 'hataclip-|e2e-workspace' }
}

function Send-ToPid([int]$ProcessId, [string]$Keys) {
  try {
    [Microsoft.VisualBasic.Interaction]::AppActivate($ProcessId) | Out-Null
  } catch {
    return
  }
  Start-Sleep -Milliseconds 200
  [System.Windows.Forms.SendKeys]::SendWait($Keys)
}

function Wait-Gone([int]$ProcessId, [int]$Ms) {
  $deadline = (Get-Date).AddMilliseconds($Ms)
  while ((Get-Date) -lt $deadline) {
    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
      return $true
    }
    Start-Sleep -Milliseconds 150
  }
  return -not [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

foreach ($proc in @(Get-HataclipNvim)) {
  $id = [int]$proc.ProcessId
  Send-ToPid $id '{ESC}'
  Start-Sleep -Milliseconds 200
  Send-ToPid $id ':q{ENTER}'
  if (Wait-Gone $id 3000) {
    continue
  }
  Send-ToPid $id '{ESC}:wq{ENTER}'
  [void](Wait-Gone $id 4000)
}
`;

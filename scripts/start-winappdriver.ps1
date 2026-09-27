#!/usr/bin/env pwsh
#Requires -Version 7
# ホストで動かす。Docker からは届かないので 127.0.0.1 ではなく全インターフェイスで開く。
# HTTP.sys は 0.0.0.0 をホスト名として受け付けない。`+` が予約 http://+:4723/ と一致する。
# WinAppDriver 本体は https://github.com/microsoft/WinAppDriver からインストールする。
$ErrorActionPreference = "Stop"

$candidates = @(
    "$env:ProgramFiles\Windows Application Driver\WinAppDriver.exe",
    "${env:ProgramFiles(x86)}\Windows Application Driver\WinAppDriver.exe"
)
$exe = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $exe) {
    Write-Error "WinAppDriver.exe が見つからない。https://github.com/microsoft/WinAppDriver からインストールする"
    exit 1
}

function Test-IsAdmin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]$id
    $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-UrlAcl {
    $out = netsh http show urlacl
    ($out | Out-String) -match "http://\+:4723/"
}

if (-not (Test-UrlAcl)) {
    if (Test-IsAdmin) {
        Write-Host "http://+:4723/ の URL 予約が無いので追加する。"
        netsh http add urlacl url=http://+:4723/ user=Everyone
    }
    else {
        Write-Error @"
WinAppDriver を全インターフェイスで開く予約が無い。

管理者の PowerShell で次のどちらか:

  ./scripts/start-winappdriver.ps1

または一度だけ:

  netsh http add urlacl url=http://+:4723/ user=Everyone

予約のあと、このスクリプトは管理者でなくてよい。
"@
        exit 1
    }
}

Write-Host "WinAppDriver を http://+:4723/ で起動する。Ctrl+C で終了。"
& $exe + 4723

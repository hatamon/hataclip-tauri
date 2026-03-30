# hataclip

## ビルド環境（Docker）

このリポジトリでは `tauri` アプリを **Docker 上でビルド**します。

- Linux 用: `scripts/build-linux.ps1`（`x86_64-unknown-linux-gnu`）
- Windows 用（クロスコンパイル）: `scripts/build-windows.ps1`（`x86_64-pc-windows-gnu`）

注意:

- `tauri` アプリ本体は `./app` 配下にあります（今回 `tauri v2 + svelte` テンプレを生成済み）。
- Windows 側の成果物は `--no-bundle` 前提で生成します（インストーラ同梱は別途手当てが必要です）。

## 前提条件

- Docker Desktop が起動していること
- `docker` コマンドが PowerShell から利用できること

## 使い方

PowerShell から実行:

```powershell
./scripts/build-linux.ps1
./scripts/build-windows.ps1
```

## 次にやること（確認）

次のステップとして、Tauri アプリの雛形（`package.json`、`src-tauri`、`tauri` コマンド用の設定）を作って、このビルド環境で実際に `tauri build` が走る状態にします。


# hataclip

Windows 向けのテキスト専用クリップボードピッカー。常時監視はしない。

ホストに入れるのは **Docker だけ**。Rust / Node の開発パッケージは不要。成果物は Windows の `exe` のみ。

## clone 直後のビルド

```bash
git clone <this-repo>
cd hataclip-tauri
docker compose run --rm test
docker compose run --rm windows-build
```

Windows（PowerShell 7）:

```powershell
git clone <this-repo>
cd hataclip-tauri
./scripts/test.ps1
./scripts/build-windows.ps1
```

初回はイメージの構築で時間がかかる。

## 成果物

`dist/windows/hataclip.exe` と、同じフォルダの `libgcc_s_seh-1.dll` / `libstdc++-6.dll` / `libwinpthread-1.dll`

インストーラ（`.msi` など）は出さない。

## 実行

`dist/windows` ごとコピーしてから:

```powershell
.\dist\windows\hataclip.exe
```

起動後はトレイに常駐せず、ショートカットで出す。

- `Ctrl+Shift+Y` いまのクリップボードを登録
- `Ctrl+Shift+L` ドロップダウンを表示
- `j` / `k` 移動、`Enter` 貼り付け、`Esc` 閉じる
- `/` 検索（IME で日本語変換してから絞る。`tokyo` では `東京` に当たらない）
- `dd` 削除、`e` その場編集（`Ctrl+Enter` 保存）

## 実行時ランタイムが無いとき

[WebView2 Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) を入れる。Windows 10/11 には大体入っている。

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

`node_modules` と `.svelte-kit`、`src-tauri/target` は Docker のボリュームに置く。バインドマウント越しの書き込みが遅いため。ホスト側にこれらは増えないので、作り直したいときはボリュームを消す:

```bash
docker compose down -v
```

## 成果物

`dist/windows/hataclip.exe` と、同じフォルダの `WebView2Loader.dll` / `libgcc_s_seh-1.dll` / `libstdc++-6.dll` / `libwinpthread-1.dll`

インストーラ（`.msi` など）は出さない。

## 実行

`dist/windows` ごとコピーしてから:

```powershell
.\dist\windows\hataclip.exe
```

起動後はトレイに常駐する。ショートカットでも出す。トレイの右クリックから設定と終了。

- `Ctrl+4` いまのクリップボードを登録
- `Ctrl+7` ドロップダウンを表示
- トレイを左クリックしてもドロップダウンを表示

`Ctrl+4` と `Ctrl+7` はトレイの `Settings` から変更できる。設定は `settings.json` に残る。ほかのアプリに取られているキーは登録に失敗して、元のキーに戻る。

## キー操作

### 移動

| キー | 動き |
| --- | --- |
| `j` / `k`、`↓` / `↑` | 移動。先頭と末尾で止まる |
| `gg` / `G` | 先頭 / 末尾へ |
| `/` | 検索。IME で日本語変換してから絞る（`tokyo` では `東京` に当たらない） |
| `Ctrl+N` / `Ctrl+P` | 検索中でも移動 |
| `Esc` / `Ctrl+[` | 閉じる。選択中なら選択だけ解除 |

### 貼り付け・コピー

| キー | 動き |
| --- | --- |
| `Enter` | 貼り付けて閉じる |
| `Ctrl+Enter` | 貼り付けて一覧を残す（続けて貼れる） |
| `Shift+Enter` | 整形して貼り付ける |
| `1`〜`9` | その番号の行を貼り付けて閉じる |
| `Ctrl+1`〜`9` | その番号の行を貼り付けて残す |
| `Ctrl+C` | クリップボードへコピー |

`Shift+Enter` の整形は、行頭の `>` を外す、前後の空白を落とす、連続した空行を 1 つにまとめる、全体を囲む `"` `'` `「」` を外す。中のインデントは残す。

本文の `{{date}}` は `2026/09/20`、`{{time}}` は `10:54` に、貼り付ける直前だけ置き換わる。履歴の本文は書き換わらない。

`Ctrl+4` と `Ctrl+7` は全体のショートカットが先に取るので、`Ctrl+4` / `Ctrl+7` での貼り付けは効かない。

### 編集・整理

| キー | 動き |
| --- | --- |
| `dd` | 削除。`p` / `P` で戻せる |
| `u` | 直前の `dd` を取り消す（1 段だけ） |
| `yy` / `Y` | ヤンク |
| `p` / `P` | ヤンクしたものを下 / 上に置く |
| `e` | その場編集（`Ctrl+Enter` 保存、`Esc` 取り消し） |
| `E` | nvim（無ければ `$EDITOR`）で編集。閉じると読み戻す |
| `t` / `T` | タグを付ける / 外す |
| `m` | ピン留めを切り替える |
| `.` | 直前の変更をもう一度 |

`.` が覚えるのは `dd`・`p` / `P`・`t` / `T`・`m` だけ。移動やヤンク、貼り付けは覚えない。アプリを終了すると忘れる。

### 複数選択

| キー | 動き |
| --- | --- |
| `V` | 選択の開始・解除 |
| `j` / `k` | 範囲を伸ばす |
| `Enter` | 選んだ行を改行でつないで貼り付け |
| `dd` / `yy` / `t` / `T` / `m` | 選んだ行すべてに効く |

### ウィンドウ

| キー | 動き |
| --- | --- |
| `Ctrl+矢印` | 24px 移動。画面の外には出ない |
| `Ctrl+Shift+←` / `→` | 幅を縮める / 広げる |
| `Ctrl+Shift+↑` / `↓` | 高さを縮める / 広げる |

下限は 200×140。タイトルバーはマウスでもつかめる。

## 並び順とタグ

一覧はピン留め、次にいまの貼り付け先で使ったもの、あとは新しい順。

`Ctrl+4` で登録するとき、`http://` `https://` で始まれば `#url`、改行を含めば `#multi`、`C:\...` や `/...` のようなパスなら `#path` が自動で付く。

貼り付け先は前面アプリの実行ファイル名で覚える。ブラウザ（chrome / msedge / firefox / brave / vivaldi / opera / chromium / zen）はウィンドウタイトルから取ったページ名まで見るので、ページごとに履歴が分かれる。URL は読まない。

## 実行時ランタイムが無いとき

[WebView2 Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) を入れる。Windows 10/11 には大体入っている。

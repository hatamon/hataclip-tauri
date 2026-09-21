const TOPICS: &[(&str, &str)] = &[
    (
        "keys",
        "移動  j / k 矢印。端で止まる。gg 先頭、G 末尾。Ctrl+D / Ctrl+U 半ページ。f と 1 文字で先頭文字へ。; 次、, 前。a いまの貼り付け先だけ。Tab よく使うタグ切替。/ 検索。\n見る  ge 全文。g? 回数・貼り付け先・タグ。\n編集  dd 削除。yy / Y ヤンク。p / P 置く。u / Ctrl+R 取り消し / やり直し。. 繰り返し。o 空行。e その場編集。E nvim。S 分割。M まとめ（V 中）。c 複製。t / T タグ。m ピン。+ / - ピンの順。\nその他  V 範囲。: コマンド。gf 開く。ドロップでパス登録。g / d / y / f / <leader> のあと 400ms で which-key。Esc / Ctrl+[ 閉じる。",
    ),
    (
        "paste",
        "Enter 貼り付けて閉じる。Ctrl+Enter 残す。Shift+Enter 整形。1〜9 その行。Ctrl+1〜9 残す。g Enter 本文のまま。J 空白つなぎ、gJ カンマ、gT タブ（V 中）。g> :quote 引用。g* :bullet 箇条書き。gp 直前の貼り付け。Ctrl+C コピー。{{sel}} {{ask:}} {{pick:}} {{app}} {{front}}。#tsv タブ区切り。#log:path ファイルへ追記。#type 1 文字ずつ。#once 貼ったら消す。",
    ),
    (
        "tag",
        "t で付ける。T で外す。V 中は範囲の全行。/ のあと #tag で絞る。Tab / Shift+Tab でよく使うタグ切替。編集中はチップで付け外し。\n\
自動  Ctrl+4 のとき http(s) なら #url、改行なら #multi、パスなら #path。ほかは自分で付ける。\n\
\n\
#pin相当 m で付ける。先頭に固定。+ / - で順\n\
#secret  一覧を ••••。貼り付けは普通\n\
#tmp     次回起動で消す\n\
#alias:foo  /foo でも当たる\n\
#here    覚えている貼り付け先が前面のときだけ出す\n\
#not     その貼り付け先のときは出さない。#here と両方なら #here だけ\n\
#ttl:1h  登録からその時間が過ぎていたら起動時に消す。m / h / d だけ。パースできなければ残す\n\
#run     貼るときコマンドを実行。自動では付けない。詳しくは :help sh\n\
#file    本文のパスのファイル内容を貼る。#run が先。ドロップすると #path と一緒に付く\n\
#type    Ctrl+V ではなく 1 文字ずつ送る\n\
#once    貼って成功したら消す\n\
#tsv     展開後をタブ区切り。JSON 配列か 1 行 1 値。壊れそうなら貼らない。#run が先\n\
#log:path  前面へ貼らずファイル末尾へ追記。パスに空白は使えない。#run があるときはその標準出力を追記\n\
#url #multi #path  登録時の自動タグ。動きは目印だけ",
    ),
    (
        "pin",
        "m でピン留め。+ でピンを上へ、- で下へ。ピンの並びは手動順が回数より優先。",
    ),
    (
        "visual",
        "V で選択開始。j / k で範囲。Enter 改行つなぎ。J 空白、gJ カンマ、gT タブ。g> 引用、g* 箇条書き。M 1 行にまとめる。c 複製。S 分割。dd / yy / t / T / m は範囲に効く。Esc で解除。",
    ),
    (
        "search",
        "/ で検索。#tag でタグ。a でいまの貼り付け先だけ。f と 1 文字で先頭文字へ飛ぶ。; 次、, 前。Ctrl+N / Ctrl+P で移動。#alias:foo は foo でも当たる。",
    ),
    (
        "edit",
        "e その場編集（Ctrl+Enter 保存、Esc 取り消し）。E nvim（無ければ $EDITOR）。o 空行を作って編集。S 改行で分割。M は V 中なら選んだ行を改行で 1 行に（タグは和集合、ピンはどれかにあれば残す）。c すぐ下に複製。:s/old/new 本文の置換。. は dd p P t T m S M c :s + - を繰り返す。",
    ),
    (
        "sh",
        "#run を自分で付けた行だけコマンドを実行する。{{sh: コマンド}} はその場の標準出力に置き換わる。#run 付きで {{sh:}} が無ければ本文全体がコマンド。:sh dir はその場実行して貼る。:@ は直前の :sh をもう一度。失敗したら貼らない。",
    ),
    (
        "template",
        "{{date}} {{time}} {{date:%Y%m%d}} {{clip}} {{sel}} {{n}} {{n:2}} {{uuid}} {{user}} {{host}} {{app}} {{front}} {{ask:名前}} {{pick: a, b}}。貼る直前だけ置き換わる。履歴の本文は変わらない。",
    ),
    (
        "gf",
        "gf は選択行を開く。http/https はブラウザ。パスは Explorer。無いときは何もしない。ファイルを一覧へドロップすると、パスを本文にして #path と #file を付けた行を先頭に作る。",
    ),
    (
        "window",
        "Ctrl+矢印で 24px 移動。Ctrl+Shift+←→ 幅、Ctrl+Shift+↑↓ 高さ。下限 200×140。",
    ),
    (
        "colon",
        ":help [topic] 使い方。:export / :import <path> Markdown。:clear / :dedup は yes で確認。:quote / :bullet 行頭。:s/old/new 置換。:sh 実行して貼る。:@ 直前の :sh。:map lhs rhs 付け替え（:map <leader>* <cmd>bullet）。:unmap。:map だけで一覧。:mapleader でリーダー（初期値 Space）。Tab でコマンド補完。↑↓ で入力履歴。",
    ),
    (
        "map",
        ":map lhs rhs で通常モードだけ付け替える。再帰しない。同じ lhs は上書き。:unmap lhs で消す。:map だけで今の付け替えを出す。lhs は j dd gT か <leader>*。rhs はキー列か :quote / <cmd>bullet（末尾 <CR> は要らない）。Esc と 1〜9 は lhs にできない。settings.json に残る。<leader> の初期値は Space。:mapleader , で変える。which-key は付け替えたあとのキーを出す。",
    ),
];

const OVERVIEW: &str = "\
移動   j k  矢印  gg G  Ctrl+D/U  f; ,  a  Tab  /\n\
見る   ge 全文  g? 回数・貼り付け先・タグ\n\
貼る   Enter 閉じる  Ctrl+Enter 残す  Shift+Enter 整形  1〜9  gp 再貼\n\
       J 空白  gJ カンマ  gT タブ  g> 引用  g* 箇条書き  g Enter 本文のまま\n\
編集   dd 削除  yy ヤンク  p P 置く  u Ctrl+R 取り消し  . 繰り返し\n\
       o 空行  e 編集  E nvim  S 分割  M まとめ  c 複製  t T タグ  m ピン  + -\n\
その他 V 範囲  gf 開く  : コマンド  ドロップでパス  which-key は g d y f <leader>\n\
:      help  sh  @  export  import  quote  bullet  s/  clear  dedup  map  unmap  mapleader\n\
\n\
詳しくは :help keys  :help paste  :help tag  :help edit  :help colon  :help map  のように。j / k でスクロール。\
";

pub fn topics() -> Vec<String> {
    TOPICS.iter().map(|(name, _)| (*name).to_string()).collect()
}

pub fn render(topic: Option<&str>) -> String {
    let Some(name) = topic.map(str::trim).filter(|name| !name.is_empty()) else {
        return format!(
            "{OVERVIEW}\n\nトピック: {}",
            topics().join(" ")
        );
    };
    if let Some((_, body)) = TOPICS.iter().find(|(key, _)| *key == name) {
        return format!(":{name}\n\n{body}");
    }
    format!(
        "ない: {name}\n\nトピック: {}",
        topics().join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_topics_and_looks_up_sh() {
        let overview = render(None);
        assert!(overview.contains("sh"));
        assert!(overview.contains("M まとめ"));
        assert!(overview.contains("map"));
        assert!(render(Some("tag")).contains("#run"));
        assert!(render(Some("tag")).contains(":help sh"));
        assert!(render(Some("tag")).contains("#log:path"));
        assert!(render(Some("sh")).contains("#run"));
        assert!(render(Some("edit")).contains("M は V 中"));
        assert!(render(Some("map")).contains("<leader>"));
        assert!(render(Some("keys")).contains("M まとめ"));
        assert!(render(Some("nope")).starts_with("ない"));
    }
}

const TOPICS: &[(&str, &str)] = &[
    (
        "keys",
        "j / k と矢印で移動。端で止まる。gg 先頭、G 末尾。Ctrl+D / Ctrl+U は半ページ。1〜9 でその行を貼り付けて閉じる。Ctrl+1〜9 は残す。u 取り消し、Ctrl+R やり直し。Esc / Ctrl+[ で閉じる。",
    ),
    (
        "paste",
        "Enter 貼り付けて閉じる。Ctrl+Enter 残す。Shift+Enter 整形。g Enter は本文そのまま。J / gJ / gT は V 中のつなぎ。g> :quote 引用。g* :bullet 箇条書き。gp 直前の貼り付け。{{sel}} {{ask:}} {{pick:}} {{app}} {{front}}。",
    ),
    (
        "tag",
        "t でタグを付ける。T で外す。V 選択中は範囲の全行。#url #multi #path は登録時に自動。#here はその貼り付け先のときだけ出す。#not はそのときは出さない。#ttl:1h は期限。#tsv と #log:path と #type と #once と #run は自動では付けない。",
    ),
    (
        "pin",
        "m でピン留め。+ でピンを上へ、- で下へ。ピンの並びは手動順が回数より優先。",
    ),
    (
        "visual",
        "V で選択開始。j / k で範囲。Enter で改行つなぎ。J 空白、gJ カンマ、gT タブ。M まとめ、c 複製。dd / yy / t / T / m / S は範囲に効く。Esc で解除。",
    ),
    (
        "search",
        "/ で検索。#tag でタグ。a でいまの貼り付け先だけ。f と 1 文字で先頭文字へ飛ぶ。; 次、, 前。Ctrl+N / Ctrl+P で移動。#alias:foo は foo でも当たる。",
    ),
    (
        "sh",
        "#run を自分で付けた行だけコマンドを実行する。{{sh: コマンド}} はその場の標準出力に置き換わる。#run 付きで {{sh:}} が無ければ本文全体がコマンド。:sh dir はその場実行して貼る。:@ は直前の :sh をもう一度。失敗したら貼らない。",
    ),
    (
        "template",
        "{{date}} {{time}} {{date:%Y%m%d}} {{clip}} {{sel}} {{n}} {{n:2}} {{uuid}} {{user}} {{host}} {{app}} {{front}} {{ask:名前}} {{pick: a, b}}。履歴の本文は変わらない。",
    ),
    (
        "gf",
        "gf は選択行を開く。http/https はブラウザ。パスは Explorer。無いときは何もしない。",
    ),
    (
        "window",
        "Ctrl+矢印で 24px 移動。Ctrl+Shift+←→ 幅、Ctrl+Shift+↑↓ 高さ。下限 200×140。",
    ),
    (
        "colon",
        ":help [topic] 使い方。:export / :import <path> Markdown。:clear / :dedup は yes で確認。:quote / :bullet 行頭。:s/old/new 置換。:sh 実行して貼る。:@ 直前の :sh。:map lhs rhs 付け替え。:unmap。:mapleader。Tab でコマンド補完。↑↓ で入力履歴。",
    ),
];

pub fn topics() -> Vec<String> {
    TOPICS.iter().map(|(name, _)| (*name).to_string()).collect()
}

pub fn render(topic: Option<&str>) -> String {
    let Some(name) = topic.map(str::trim).filter(|name| !name.is_empty()) else {
        return format!("トピック: {}\n\n:help sh のように指定する。", topics().join(" "));
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
        assert!(render(None).contains("sh"));
        assert!(render(Some("sh")).contains("#run"));
        assert!(render(Some("nope")).starts_with("ない"));
    }
}

const TOPICS: &[(&str, &str)] = &[
    (
        "keys",
        "j / k と矢印で移動。端で止まる。gg 先頭、G 末尾。1〜9 でその行を貼り付けて閉じる。Ctrl+1〜9 は残す。Esc / Ctrl+[ で閉じる。",
    ),
    (
        "paste",
        "Enter 貼り付けて閉じる。Ctrl+Enter 残す。Shift+Enter 整形して貼る。Ctrl+C コピー。{{date}} {{time}} {{clip}} {{n}} {{uuid}} は貼る直前だけ展開。",
    ),
    (
        "tag",
        "t でタグを付ける。T で外す。V 選択中は範囲の全行。#url #multi #path は登録時に自動。",
    ),
    ("pin", "m でピン留めを切り替える。ピン留めは一覧の先頭。"),
    (
        "visual",
        "V で選択開始。j / k で範囲。Enter で改行つなぎ貼り付け。dd / yy / t / T / m は範囲に効く。Esc で解除。",
    ),
    (
        "search",
        "/ で検索。#tag でタグ絞り込み。Ctrl+N / Ctrl+P で移動。#alias:foo を付けた行は foo でも当たる。",
    ),
    (
        "sh",
        "#run を自分で付けた行だけコマンドを実行する。{{sh: コマンド}} はその場の標準出力に置き換わる。#run 付きで {{sh:}} が無ければ本文全体がコマンド。:sh dir はその場実行して貼る。:@ は直前の :sh をもう一度。失敗したら貼らない。",
    ),
    (
        "template",
        "{{date}} は 2026/09/20、{{time}} は 10:54、{{date:%Y%m%d}} は書式、{{clip}} はいまのクリップボード、{{n}} は連番、{{n:2}} は 01、{{uuid}} {{user}} {{host}}。履歴の本文は変わらない。",
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
        ":help [topic] 使い方。:export <path> Markdown 書き出し。:clear は yes でピン以外削除。:sh <cmd> 実行して貼る。:@ 直前の :sh。",
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

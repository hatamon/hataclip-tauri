const TOPICS: &[(&str, &str)] = &[
    (
        "keys",
        "移動  j / k 矢印。端で止まる。gg 先頭、G 末尾。Ctrl+D / Ctrl+U 半ページ。f と 1 文字で先頭文字へ。; 次、, 前。a いまの貼り付け先だけ。Tab よく使うタグ切替（検索中は一覧へ）。/ 検索。Esc で絞りを外す。\n見る  ge 展開して全文。g? 回数・貼り付け先・タグ。\n編集  dd 削除（#lock は残す）。yy / Y ヤンク。p / P 置く。u / Ctrl+R 取り消し / やり直し。. 繰り返し。o 空行。e その場編集。E nvim（無ければメモ帳）。S 分割。J まとめ（V 中）。c 複製。t / T タグ。gp ピン。ga 前面の #app:。+ / - ピンの順。\nその他  V 範囲。: コマンド。ドロップでパス登録。g / d / y / f / <leader> のあと 400ms で which-key。Esc / Ctrl+[ 閉じる。一覧を開くとき選択が空でなければ Ctrl+C の 200ms 後を読んですぐ戻す。当たった #grab はピンの直下、sh を含まない sel のパイプはその次。本文は変えない。選択が空、または Ubuntu では今の順。",
    ),
    (
        "paste",
        "Enter 貼り付けて閉じる。次に一覧を出すとその行。Ctrl+Enter 残す。1〜9 その行。Ctrl+1〜9 残す。:format 整形。:raw 本文のまま。:join 区切りつなぎ（未指定は , 。1行ならその本文。V ならその範囲。タブは :join \"\\t\"。:comma と :tab は何もしない）。:quote 行頭（未指定は > 。:quote \"* \" で箇条書き）。行末も :quote \"> \" \"<\" で hello が > hello<。\"\" はその端を付けない。:type 1 文字ずつ。:open は URL/パスを開く。g. 直前の貼り付け。Ctrl+C コピー。{{sel}} {{ask:}} {{pick list:}} {{pick tag:env}} {{pick search:}} {{app}} {{front}} {{focus}} {{when app: chrome}} {{env:}} {{@foo}} {{var:a}} {{tag:work}} {{type:<Tab>}}。#confirm は貼る前に展開後を出す。#run があればそれで足りる。#tsv タブ区切り。#log:path ファイルへ追記。#once 貼ったら消す。Ctrl+Shift+1〜9 は #slot:n があればそれ。無ければ並びの番号。前面アプリのコピー／貼り付けキーは settings.json の target_keys。:set paste shift+insert。Ctrl+Shift+H は前面の {{date}} / :sh dir / :echo 2+3 を置き換える（一覧は出さない。パイプは実行しない）。本文がパイプの行は Enter / Ctrl+Enter / 1〜9 / Ctrl+1〜9 / Ctrl+Shift+1〜9 でそのパイプを実行する。例: #slot:1 の本文が :sel | snake。メモ帳で getUserName を選んで Ctrl+Shift+1 すると一覧は出ずに get_user_name に置き換わる。:sel | quote で行頭に > を付けて | clip ならクリップボードへ置き、前面には貼らない。段の名前が不正、または sel が空なら何もしない。{{ }} の中だけの | はパイプにしない。",
    ),
    (
        "tag",
        "t で付ける。T で外す。V 中は範囲の全行。/ のあと #tag で絞る。Tab / Shift+Tab でよく使うタグ切替。編集中はチップで付け外し。\n\
自動  Ctrl+4 のとき http(s) なら #url、パスなら #path。ほかは自分で付ける。2秒以内にもう一度押すと、1回目を入力、2回目を出力として :sel | snake か #grab を仮の行にする（候補は camel pascal snake kebab upper lower の順。当たらなければ共通の英数字を <a> <b> <c> にする）。Enter で残す。Esc で消す。当たらなければ2回目は普通に1件登録。一覧を閉じても仮の行は消える。\n\
\n\
#pin相当 gp で付ける。先頭に固定。+ / - で順\n\
#secret  一覧を ••••。貼り付けは普通\n\
#tmp     次回起動で消す。#lock があれば残す\n\
#alias:foo  /foo でも当たる。{{@foo}} でその本文\n\
#app:chrome  前面のアプリ名が chrome のときだけ出す。#app:code と複数ならどれか。ブラウザはページが違っても当たる\n\
#not:chrome  そのアプリのときは出さない。#app: と両方なら、アプリに当たって #not に当たらないときだけ\n\
#ttl:1h  登録からその時間が過ぎていたら起動時に消す。m / h / d だけ。パースできなければ残す。#lock があれば残す\n\
#run     貼るときコマンドを実行。自動では付けない。詳しくは :help sh\n\
#confirm 貼る前に展開後を出して Enter。#run があればそれで足りる\n\
#file    本文のパスのファイル内容を貼る。#run が先。ドロップすると #path と一緒に付く\n\
#once    貼って成功したら消す。#lock があれば残す\n\
#lock    dd / :clear / J で消えない。外すのは T\n\
#slot:3  Ctrl+Shift+3 でその行。複数なら一覧の先。無ければ今までの 3 番目\n\
#grab    1行目が型、2行目が出力。穴は <名前>。前面の選択が型に当たれば2行目を埋めて貼る。当たらなければ何もしない。値は :set に残さない。Ubuntu はクリップボードを照合する\n\
#tsv     展開後をタブ区切り。JSON 配列か 1 行 1 値。壊れそうなら貼らない。#run が先\n\
#log:path  前面へ貼らずファイル末尾へ追記。パスに空白は使えない。#run があるときはその標準出力を追記\n\
#url #path  登録時の自動タグ。動きは目印だけ",
    ),
    (
        "pin",
        "gp でピン留め。ga で選んだ行の #app: をいまの前面アプリにする。ほかの #app: は外す。+ でピンを上へ、- で下へ。ピンの並びは手動順。",
    ),
    (
        "visual",
        "V で選択開始。j / k で範囲。Enter 改行つなぎ。:join 区切りつなぎ。:quote 行頭（:quote \"* \" で箇条書き）。J 1 行にまとめる（#lock があるとまとめない）。c 複製。S 分割。:sort 本文の順。:!!sh 各行をフィルタ。dd / yy / t / T / gp / ga は範囲に効く。dd は #lock を残す。Esc で解除。",
    ),
    (
        "search",
        "/ で検索。Tab は一覧へ（絞りと検索欄は残る）。Esc / Ctrl+[ で絞りを外す。# の候補がある Tab はタグ補完。#tag でタグ。a でいまの貼り付け先だけ。f と 1 文字で先頭文字へ飛ぶ。; 次、, 前。Ctrl+N / Ctrl+P で移動。#alias:foo は foo でも当たる。",
    ),
    (
        "edit",
        "e その場編集（Ctrl+Enter 保存、Esc 取り消し）。E nvim（無ければ $EDITOR、それも無ければメモ帳）。o 空行を作って編集。S 改行で分割。J は V 中なら選んだ行を改行で 1 行に（タグは和集合、ピンはどれかにあれば残す）。c すぐ下に複製。:s/old/new 本文の置換。:!!sh は各行をコマンドで書き換える。:sort は V 中なら本文の順。. は dd p P t T gp ga S J c :s :!! :sort g~ g: + - を繰り返す。g: は行に残した式をもう一度実行する。式が無ければ何もしない。sel はいまの前面の選択、. はいまの本文。失敗したら本文はそのまま。:from はその式だけを編集する（e は本文）。| add と :!!sh でできた行に式が残る。g~ は #grab の1行目と2行目を入れ替える。2行目が無ければ何もしない。3行目以降はそのまま。u で戻せる。",
    ),
    (
        "sh",
        "#run を自分で付けた行だけコマンドを実行する。{{sh: コマンド}} はその場の標準出力に置き換わる。#run 付きで {{sh:}} が無ければ本文全体がコマンド。貼る前に出力を出して Enter で貼る。Esc は中止。:sh dir はその場実行して貼る（stdin なし）。:.!sh xxx はカレント行（V なら改行つなぎ）を stdin に流す。:!!sh xxx は各行を stdin に流して本文を書き換える（貼らない）。式が残るので g: でもう一度。失敗したら本文はそのまま。:@ は直前の :sh / :.!sh のスクリプトをもう一度（stdin はいまの選択）。失敗したら貼らない／書き換えない。| clip でクリップボードへ（貼らない）。| で左からつなぐ。次の :sh は左の結果を stdin に受け取る。",
    ),
    (
        "template",
        "貼る直前だけ置き換わる。履歴の本文は変わらない。`:` の代わりに空白でもよい（{{var a}} {{date %Y%m%d}}）。エクスプローラーの名前変更向け。\n\
\n\
{{date}}           2026/09/20\n\
{{time}}           10:54\n\
{{date:%Y%m%d}}    chrono の strftime\n\
{{date-1d}}        昨日。{{date-1d:%Y%m%d}} {{date+2d}} {{date+1w}} {{date+1m}} も可\n\
{{clip}}           いまのクリップボード。空なら空\n\
{{sel}}            前面の選択。無ければ空。{{sel|clip}} は空なら隣。隣がトークンなら展開し、違う文字ならそのまま。{{front|無題}} {{sel|clip|なし}}\n\
{{ask:名前}}       貼る前に入力。同じ名前は 1 回\n\
{{pick list: a, b}}  貼る前に候補から選ぶ。j / k と Enter。候補が 9 個までなら 1〜9 でその番。10 個以上は j / k。Esc は中止。同じ候補は 1 回。{{ask:}} があるときは先に全部聞く。{{pick: a, b}} は空。{{pick tag:env}} はタグ env の本文。{{pick search: \"xx\"}} は / と同じ当たりの行。自分自身は入れない。無ければ聞かない\n\
{{app}}            前面アプリのプロセス名。無ければ空\n\
{{front}}          前面ウィンドウのタイトル。無ければ空\n\
{{focus}}          フォーカスしている入力欄の種類。Edit Document ComboBox。取れなければ空。Ubuntu は空。常時監視はしない\n\
{{cred:github}}    貼るときに資格情報マネージャーのパスワード。履歴に残るのはこの文字だけ。無ければ貼らない。ge と Ctrl+C では展開しない。プレビューは ••••。Ubuntu はこのトークンがある行を貼らない\n\
{{when app: chrome}}  前面が chrome のときだけその区間。複数は {{when app: chrome, msedge}}。{{when var: a: \"AAA\"}} は :set のそのままの文字列。{{when focus: Edit}} は入力欄の種類。{{when}} はどれにも当たらないとき。{{when chrome}} は展開せずそのまま。Ubuntu では app と focus は空\n\
{{n}} / {{n:2}}    連番。Ctrl+Enter で増える。:n 100 で初期値。閉じると初期値に戻る。ge やコピーでは進まない\n\
{{uuid}}           UUID v4\n\
{{user}}           ログイン名\n\
{{host}}           コンピュータ名\n\
{{env:USERPROFILE}}  同名の環境変数。無ければ空\n\
{{@foo}}           #alias:foo の先の行の本文。差し込んだ本文も展開する。同じ名前を二度辿ったら空。エイリアス側の #run などは見ない\n\
{{var:a}}          :set a= の値。中の {{date}} も展開する。値が :sh dir ならそのとき実行。:set では実行しない。名前に {{var:b}} を書ける（{{var: {{var: b}}}}）。:set a+=1 は貼って成功したあと 1 増やす。無い変数は 1。数字以外には付かない\n\
{{nop: メモ}}      貼るとき消える。中の {{date}} は展開しない。| と }} は書かない\n\
{{tag:work}}       いまの一覧でタグ work の本文。並びどおり、改行つなぎ。差し込んだ本文も展開する。同じタグを二度辿ったら空。その行の #run などは見ない\n\
{{type:<Tab>}}     貼る途中でキーを送る。id{{type:<Tab>}}pass は id を貼って Tab を押して pass を貼る。{{type:<Ctrl+A>abc<Enter>}} {{type:<Ctrl+Shift+A>}} も可。矢印は <Up> <Down> <Left> <Right>。F キーは <F1>〜<F24>。文字は 1 文字ずつ。Tab Enter Esc Space BS Del Home End PgUp PgDn と Ctrl/Shift/Alt+それら\n\
{{wait:200}}       そのあと 200ms 待つ。最大 5 秒\n\
{{sh: コマンド}}   #run 付きの行だけ実行。標準出力。失敗したら貼らない",
    ),
    (
        "open",
        ":open は選択行を開く。http/https はブラウザ。パスは Explorer。無いときは何もしない。ファイルを一覧へドロップすると、パスを本文にして #path と #file を付けた行を先頭に作る。欲しければ :map gf :open など。",
    ),
    (
        "window",
        "Ctrl+矢印で 24px 移動。Ctrl+Shift+←→ 幅、Ctrl+Shift+↑↓ 高さ。下限 200×140。",
    ),
    (
        "colon",
        ":help [topic] 使い方。:export / :import <path> Markdown。:clear / :dedup は yes で確認。:clear はピンと #lock 以外。:quote 行頭（未指定は > 。:quote \"* \" で箇条書き）。行末も :quote \"> \" \"<\" で hello が > hello<。\"\" はその端を付けない。:type 1 文字ずつ。:format 整形。:raw 本文のまま。:join 区切りつなぎ（未指定は , 。1行ならその本文。V ならその範囲。タブは :join \"\\t\"。:comma と :tab は何もしない）。:open URL/パス。:echo 2+3 四則。* / が先。() で変えられる。変数も使える。:s/old/new 置換。:sort は V 中なら本文の順。:sh 実行して貼る。:.!sh はカレント行を stdin に。:!!sh は本文を書き換える。:@ 直前の :sh。| で左からつなぐ（引用符の中の | では切らない）。| clip はクリップボード（前面には貼らない。末尾の > clip は不正で何もしない）。| add は一覧へ1件。できた行には add より前の式が残る（:sel | kebab | add なら sel | kebab）。g: でもう一度実行する。sel はいまの前面の選択、. はいまの本文。失敗したら本文はそのまま。:from で式だけ編集する。| set a は変数へ。| open は URL かパスなら開く。| show はここに出す。最初の :sh は stdin なし。次の :sh は左を stdin に。:. は選択行。先頭の clip はクリップボードを読む。sel は前面の選択。Windows では Ctrl+C の 200ms 後を読んで、すぐクリップボードを戻す。空なら何もしない。Ubuntu では sel を含むパイプは実行しない。流れがあるあとの clip は不正で何もしない。例: :sel | upper は前面の hello を HELLO にして貼る。:. | upper は一覧の行を大文字にして貼る。:clip | show はクリップボードをここに出す。camel pascal snake kebab upper lower は文字の形を変える。json xml は整形する。読めなければ何もしない。split は区切り必須（タブは split \"\\t\"）。分けた列はタブでつなぐ。col は 1 始まり。足りない行は捨て、1 未満や整数でないときは何もしない。get は JSON Pointer。読めない、または無いときは何もしない。例: :sel | split , | col 2 は a,b,c の b。:clip | json | get /items/0/name | show は hatamon。diff と only の引数は . か clip。:clip | diff . | show はクリップボードにあって選択に無い行を - 、選択にあってクリップボードに無い行を + 。共通行は出さない。:. | only clip | show は選択にあってクリップボードに無い行。全部同じ、または only が 0 行なら何もしない。put は JSON Pointer の位置へ書く。引数はポインタ、空白、値。値は引用できる。JSON として読めたらその値、読めなければ文字列。流れが JSON でない、またはポインタが無いときは何もしない。例: :sel | put /n 2 | json は n が 2 の JSON を整形して貼る。each はこれより後ろの段を行ごとに実行し、成功した行だけを改行でつなぐ。後ろに sh があるとパイプ全体を何もしない。例: :sel | each | get /id は各行の id。echo は式の結果で流れを置き換える。左は見ない。計算できなければ何もしない。段が1つでも同じ。:upper は選択行の hello を HELLO にして貼る。:json は読めるときだけ整形して貼る。:map lhs rhs 付け替え（:map <leader>* :quote \"* \"）。:unmap。:map だけで一覧。:mapleader でリーダー（初期値 Space）。:settings は settings.json をエディタで開く。:set paste shift+insert はいまの前面アプリ。:set a=\"{{date}}\" は変数。引用の中の \\\" は \"、\\t はタブ、\\n は改行。:set a=\"say \\\"hi\\\"\" の値は say \"hi\"。{{var:a}} と {{var a}} で貼るとき展開。:set だけで一覧。:set a= で消す。:set a+=1 は貼って成功したあと 1 増やす。無い変数は 1。数字以外には付かない。:n 100 は {{n}} の初期値。settings.json に残る。:n だけでいまの値。:n 1 で 1 から。:tags はタグと件数。Tab でコマンド補完。↑↓ で入力履歴。hataclip に式を1つ渡すとウィンドウを開かず実行する。標準入力がパイプならそれが流れの最初。行き先が無ければ標準出力。例: hataclip \"echo 3+4\" は 7。hataclip quote は選択の代わりに標準入力を引用符付きで出す。壊れた段や空の入力は何も出さず終了コード 1。",
    ),
    (
        "map",
        ":map lhs rhs で通常モードだけ付け替える。再帰しない。同じ lhs は上書き。:unmap lhs で消す。:map だけで今の付け替えを出す。lhs は j dd gT か <leader>*。rhs はキー列か :quote / :quote \"* \"（末尾 <CR> は要らない）。Esc と 1〜9 は lhs にできない。settings.json に残る。<leader> の初期値は Space。:mapleader , で変える。which-key は付け替えたあとのキーを出す。",
    ),
];

const OVERVIEW: &str = "\
移動   j k  矢印  gg G  Ctrl+D/U  f; ,  a  Tab  /  Escで絞り解除\n\
見る   ge 展開して全文  g? 回数・貼り付け先・タグ\n\
貼る   Enter 閉じる  Ctrl+Enter 残す  1〜9  g. 再貼\n\
       :format 整形  :raw 本文のまま  :join 区切り  :quote 行頭\n\
編集   dd 削除  yy ヤンク  p P 置く  u Ctrl+R 取り消し  . 繰り返し\n\
       o 空行  e 編集  E nvim  S 分割  J まとめ  c 複製  t T タグ  gp ピン  ga #app  + -\n\
その他 V 範囲  :open 開く  : コマンド  ドロップでパス  which-key は g d y f <leader>\n\
:      help  sh  !!  @  echo  export  import  quote  type  format  raw  join  open  s/  sort  clear  dedup  map  unmap  mapleader  set  n  settings  tags\n\
\n\
詳しくは :help keys  :help paste  :help tag  :help template  :help edit  :help colon  :help map  のように。j / k でスクロール。\
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
        assert!(overview.contains("J まとめ"));
        assert!(overview.contains("map"));
        assert!(render(Some("tag")).contains("#run"));
        assert!(render(Some("tag")).contains(":help sh"));
        assert!(render(Some("tag")).contains("#log:path"));
        assert!(render(Some("tag")).contains("#app:chrome"));
        assert!(render(Some("tag")).contains("#slot:3"));
        assert!(render(Some("tag")).contains("#confirm"));
        assert!(render(Some("template")).contains("{{when app: chrome}}"));
        assert!(render(Some("template")).contains("{{when chrome}} は展開せずそのまま"));
        assert!(render(Some("paste")).contains("{{when app: chrome}}"));
        assert!(render(Some("template")).contains("{{pick tag:env}}"));
        assert!(render(Some("template")).contains("候補が 9 個までなら 1〜9"));
        assert!(render(Some("template")).contains("{{date+1w}}"));
        assert!(render(Some("keys")).contains("ga 前面"));
        assert!(render(None).contains("ga #app"));
        assert!(render(Some("paste")).contains("#slot:n"));
        assert!(render(Some("paste")).contains("#confirm"));
        assert!(render(Some("template")).contains("{{var: {{var: b}}}}"));
        assert!(!render(Some("tag")).contains("#here"));
        assert!(render(Some("sh")).contains("#run"));
        assert!(render(Some("edit")).contains("J は V 中"));
        assert!(render(Some("template")).contains("{{@foo}}"));
        assert!(render(Some("template")).contains("{{var a}}"));
        assert!(render(Some("template")).contains("{{nop: メモ}}"));
        assert!(render(Some("template")).contains("{{sh: コマンド}}"));
        assert!(render(Some("template")).contains("{{tag:work}}"));
        assert!(render(Some("template")).contains("{{wait:200}}"));
        assert!(render(Some("template")).contains("{{date-1d}}"));
        assert!(render(Some("template")).contains("{{sel|clip}}"));
        assert!(render(Some("colon")).contains(":tags"));
        assert!(render(None).contains(":help template"));
        assert!(render(Some("colon")).contains(":n 100"));
        assert!(render(Some("template")).contains(":n 100"));
        assert!(render(Some("colon")).contains(":set a="));
        assert!(render(Some("colon")).contains("say \"hi\""));
        assert!(render(Some("colon")).contains(":set a+=1"));
        assert!(render(Some("template")).contains(":set a+=1"));
        assert!(render(Some("template")).contains("{{env:USERPROFILE}}"));
        assert!(render(Some("paste")).contains("次に一覧を出すとその行"));
        assert!(render(Some("paste")).contains(":sel | snake"));
        assert!(render(Some("paste")).contains("Ctrl+Shift+H は前面の"));
        assert!(render(Some("sh")).contains(":!!sh"));
        assert!(render(Some("colon")).contains(":!!sh"));
        assert!(render(Some("edit")).contains(":!!sh"));
        assert!(!render(Some("sh")).contains("> clip"));
        assert!(render(Some("colon")).contains(":.!sh"));
        assert!(render(Some("colon")).contains("末尾の > clip は不正"));
        assert!(render(Some("colon")).contains(":quote \"* \""));
        assert!(render(Some("colon")).contains("> hello<"));
        assert!(render(Some("paste")).contains(":quote \"* \""));
        assert!(render(Some("paste")).contains(":join"));
        assert!(render(Some("colon")).contains(":join"));
        assert!(render(Some("colon")).contains(":comma と :tab は何もしない"));
        assert!(render(Some("paste")).contains(":open"));
        assert!(render(Some("open")).contains(":open"));
        assert!(render(Some("paste")).contains(":sh dir"));
        assert!(render(Some("colon")).contains(":echo"));
        assert!(render(Some("colon")).contains("sel は前面の選択"));
        assert!(render(Some("colon")).contains(":sel | upper"));
        assert!(render(Some("colon")).contains("段が1つでも同じ"));
        assert!(render(Some("colon")).contains(":type"));
        assert!(render(Some("colon")).contains(":settings"));
        assert!(render(Some("colon")).contains(":set paste"));
        assert!(render(Some("edit")).contains("メモ帳"));
        assert!(render(Some("map")).contains("<leader>"));
        assert!(render(Some("keys")).contains("J まとめ"));
        assert!(render(Some("keys")).contains("gp ピン"));
        assert!(render(None).contains("g. 再貼"));
        assert!(render(Some("search")).contains("Esc / Ctrl+[ で絞りを外す"));
        assert!(render(Some("search")).contains("Tab は一覧へ"));
        assert!(render(Some("pin")).contains("gp でピン留め"));
        assert!(render(Some("nope")).starts_with("ない"));
    }
}

struct Page {
    name: &'static str,
    group: &'static str,
    summary: &'static str,
    body: &'static str,
}

const GROUPS: &[(&str, &str)] = &[
    ("keys", "移動"),
    ("look", "見る"),
    ("paste", "貼る"),
    ("expand", "展開と補完"),
    ("edit", "編集"),
    ("tag", "タグ"),
    ("pin", "ピン"),
    ("search", "検索"),
    ("visual", "範囲"),
    ("colon", "コマンド"),
    ("pipe", "パイプ"),
    ("template", "テンプレート"),
    ("window", "ウィンドウ"),
    ("keymap", "割り当て"),
    ("other", "その他"),
];

const PAGES: &[Page] = &[
    Page {
        name: "j",
        group: "keys",
        summary: "1つ下の行へ移る",
        body: "\
打つ: j または ↓
変わるのは一覧の選択。本文・前面・クリップボードは触らない。末尾では止まる。
例: 2行目で j すると 3行目。最後の行では動かない。
詳しくは :help k :help gg :help G",
    },
    Page {
        name: "k",
        group: "keys",
        summary: "1つ上の行へ移る",
        body: "\
打つ: k または ↑
変わるのは一覧の選択。本文・前面・クリップボードは触らない。先頭では止まる。
例: 3行目で k すると 2行目。最初の行では動かない。
詳しくは :help j",
    },
    Page {
        name: "gg",
        group: "keys",
        summary: "一覧の先頭へ移る",
        body: "\
打つ: gg
変わるのは一覧の選択。本文・前面・クリップボードは触らない。
例: 最後の行で gg すると最初の行。
詳しくは :help G",
    },
    Page {
        name: "G",
        group: "keys",
        summary: "一覧の末尾へ移る",
        body: "\
打つ: G
変わるのは一覧の選択。本文・前面・クリップボードは触らない。
例: 最初の行で G すると最後の行。
詳しくは :help gg",
    },
    Page {
        name: "half",
        group: "keys",
        summary: "半ページ上下へ移る",
        body: "\
打つ: Ctrl+D で下、Ctrl+U で上
変わるのは一覧の選択。本文・前面・クリップボードは触らない。端では止まる。
例: 先頭で Ctrl+D すると、見える半分ほど下の行。
詳しくは :help j",
    },
    Page {
        name: "f",
        group: "keys",
        summary: "先頭文字へ飛ぶ",
        body: "\
打つ: f のあと 1 文字
変わるのは一覧の選択。その文字で本文が始まる行へ飛ぶ。本文は変えない。
例: f h で hello の行へ。次は :help semicolon 、前は :help comma
当たらなければ動かない。",
    },
    Page {
        name: "semicolon",
        group: "keys",
        summary: "同じ先頭文字の次へ",
        body: "\
打つ: ;
変わるのは一覧の選択。直前の f と同じ文字の次の行へ。本文は変えない。
例: f h のあと ; で、次の h で始まる行。
f の前なら動かない。; だけは検索の次ではない。文字の大きさは :help zoom
詳しくは :help f :help comma",
    },
    Page {
        name: "comma",
        group: "keys",
        summary: "同じ先頭文字の前へ",
        body: "\
打つ: ,
変わるのは一覧の選択。直前の f と同じ文字の前の行へ。本文は変えない。
例: f h のあと , で、前の h で始まる行。
f の前なら動かない。
詳しくは :help f :help semicolon",
    },
    Page {
        name: "a",
        group: "keys",
        summary: "いまの貼り付け先だけにする",
        body: "\
打つ: a
変わるのは一覧の絞り。本文は変えない。もう一度で解除。
例: メモ帳へ貼った行だけが残る。記録が無い行は隠れる。
貼り付け先の中身は :help info
詳しくは :help search",
    },
    Page {
        name: "tab",
        group: "keys",
        summary: "タグの絞りを切り替える",
        body: "\
打つ: Tab または Shift+Tab
変わるのは一覧の絞り。通常はよく使うタグを順に切り替える。検索中は一覧へ移る（絞りと検索欄は残る）。# の候補があるときはタグ名の補完。
例: Tab で work の行だけ。検索中の Tab は検索欄から一覧へ。
詳しくは :help slash :help t",
    },
    Page {
        name: "esc",
        group: "keys",
        summary: "絞り、選択、ヘルプ、待ちを外す",
        body: "\
打つ: Esc または Ctrl+[
変わるのは画面の状態。絞りがあれば絞りを外す。無ければ一覧を閉じる。V の選択中なら選択だけ解除。ヘルプ中はヘルプだけ閉じる。g などの待ちは待ちだけ解除。
例: /work のあと Esc で全行が戻る。
閉じるだけなので本文は変えない。失敗としては覚えない。
詳しくは :help slash :help visual :help whichkey",
    },
    Page {
        name: "ge",
        group: "look",
        summary: "展開した全文を見る",
        body: "\
打つ: ge
変わるのはこの画面。選んだ行を展開して全文を出す。履歴の本文は変えない。{{n}} は進まない。
例: 本文 {{date}} の行で ge すると、今日の日付が見える。
展開できなければ空に近い結果になる。貼り付けはしない。
詳しくは :help template :help info",
    },
    Page {
        name: "info",
        group: "look",
        summary: "貼り付け先とタグを見る",
        body: "\
打つ: g?
変わるのはこの画面。その行の貼り付け先とタグを出す。回数は出さない。本文は変えない。
例: メモ帳へ貼った行なら、貼り付け先に notepad とタグが出る。
行が無ければ出さない。
詳しくは :help a :help ge",
    },
    Page {
        name: "enter",
        group: "paste",
        summary: "貼り付けて一覧を閉じる",
        body: "\
打つ: Enter
変わるのは前面（展開して貼る）。一覧は閉じる。次に一覧を出すとその行。履歴の本文は変えない。
例: hello の行で Enter すると前面は hello。
本文がパイプならそのパイプを実行する。段が不正、または sel が空なら何もしない。
V なら改行つなぎ。詳しくは :help visual :help pipe :help stay",
    },
    Page {
        name: "stay",
        group: "paste",
        summary: "貼り付けて一覧を残す",
        body: "\
打つ: Ctrl+Enter
変わるのは前面。一覧は閉じない。{{n}} はこのとき進む。
例: hello で Ctrl+Enter すると前面は hello のまま、一覧も残る。
貼れないときは何もしない。
詳しくは :help enter :help n",
    },
    Page {
        name: "digit",
        group: "paste",
        summary: "先頭9件を番号で貼って閉じる",
        body: "\
打つ: 1 から 9
変わるのは前面。その番号の行を貼って一覧を閉じる。10件目以降には数字は出ない。
例: 3 で 3 行目を貼って閉じる。
行が無ければ何もしない。残すのは :help ctrldigit
詳しくは :help enter",
    },
    Page {
        name: "ctrldigit",
        group: "paste",
        summary: "番号の行を貼って一覧を残す",
        body: "\
打つ: Ctrl+1 から Ctrl+9
変わるのは前面。一覧は閉じない。
例: Ctrl+2 で 2 行目を貼り、一覧は残る。
行が無ければ何もしない。
詳しくは :help digit :help stay",
    },
    Page {
        name: "slots",
        group: "paste",
        summary: "一覧を出さずに番号の行を貼る",
        body: "\
打つ: Ctrl+Shift+1 から Ctrl+Shift+9
変わるのは前面。一覧は出さない。#slot:n があればその行。複数なら一覧の先。無ければ並びの n 番目。
例: #slot:1 の本文が :sel | snake のとき、メモ帳で getUserName を選んで Ctrl+Shift+1 すると get_user_name に置き換わる。
本文がパイプならそのパイプを実行する。段の名前が不正、または sel が空なら何もしない。
詳しくは :help slottag :help pipe",
    },
    Page {
        name: "gdot",
        group: "paste",
        summary: "直前に貼った行をもう一度貼る",
        body: "\
打つ: g.
変わるのは前面。直前の貼り付けと同じ行を貼る。
例: hello を貼ったあと g. で、もう一度 hello。
直前が無ければ何もしない。
詳しくは :help enter :help dot",
    },
    Page {
        name: "ctrlc",
        group: "paste",
        summary: "選んだ行をクリップボードへコピーする",
        body: "\
打つ: Ctrl+C
変わるのはクリップボード。一覧の選択行をコピーする。前面には貼らない。
例: hello の行で Ctrl+C すると、クリップボードは hello。
行が無ければ何もしない。
詳しくは :help clip :help yy",
    },
    Page {
        name: "format",
        group: "paste",
        summary: "空白や改行を整えて貼る",
        body: "\
打つ: :format
変わるのは前面。選択行を貼り付け向けに整える。履歴は触らない。
例: 選択が {\"a\":1} なら整形して前面へ貼る。
空なら何もしない。
詳しくは :help raw :help json",
    },
    Page {
        name: "raw",
        group: "paste",
        summary: "展開せず本文のまま貼る",
        body: "\
打つ: :raw
変わるのは前面。{{date}} などは展開しない。履歴は触らない。
例: 本文 {{date}} は前面も {{date}}。
空なら何もしない。
詳しくは :help enter :help date",
    },
    Page {
        name: "join",
        group: "paste",
        summary: "区切りでつないで貼る",
        body: "\
打つ: :join または :join \" | \"
変わるのは前面へ貼る文字。履歴は触らない。1行ならその本文を貼る。V ならその範囲を区切りでつなぐ。未指定は , 。タブは :join \"\\t\"。:comma と :tab は何もしない。
例: hello と world を V して :join \" | \" で前面は hello | world。
空の選択は何もしない。
詳しくは :help visual :help quote",
    },
    Page {
        name: "quote",
        group: "paste",
        summary: "行頭（と行末）を付けて貼る",
        body: "\
打つ: :quote または :quote \"* \" または :quote \"> \" \"<\"
変わるのは前面へ貼る文字。履歴は触らない。未指定の行頭は > 。すでにその行頭（と行末）なら付けない。\"\" はその端を付けない。失敗はしない。
例: 選択行 hello に :quote \"* \" で前面は * hello。:quote \"> \" \"<\" で hello は > hello<。
詳しくは :help join :help pipe",
    },
    Page {
        name: "type",
        group: "paste",
        summary: "1文字ずつ前面へ送る",
        body: "\
打つ: :type
変わるのは前面（1文字ずつ送る）。履歴は触らない。
例: 選択 hello を :type で前面へ1文字ずつ入る。
Ubuntu はキーを送らないので何もしない。貼る途中のキーは :help typeto
詳しくは :help enter",
    },
    Page {
        name: "ctrl8",
        group: "expand",
        summary: "前面の選択を展開して置き換える",
        body: "\
打つ: 展開キー（初期値 Ctrl+8）。設定に出るのはこの1つ。
変わるのは前面の選択。一覧は出さない。空白を除いた先頭が / なら補完で、展開しない。/ は検索語に入れない。/foo は foo。/foo #tag は本文 foo とタグ tag。/#work は #work。/ だけは何もしない。中に {{date}} や :sh があっても展開しない。/xxx {{date}} の検索語は xxx {{date}}。/:sh xxx の検索語は :sh xxx。展開が空でも補完にはしない。
選択が空なら :help home のキーで行頭まで選び、もう一度コピーする。それでも空なら何もしない。自動選択に改行があれば展開も補完もせず、理由は「行頭までの選択に改行がある」。改行が無く行に / があれば最後の / から行末だけ補完し、前は残して1回貼る。/ が無ければ行全体を展開する。自分で選んだ複数行はそのまま。Ubuntu はキーを送らないので空は何もしない。
:sh か {{sh:}} か :@ を含み、先頭が / でなければ、実行前にコマンドを出して確認する。Enter で元の前面へ戻して実行する。Esc は何もしない。選択は残し、補完には進まない。:echo や sh の無い展開は確認しない。一覧で打つ :sh と CLI は確認しない。
選択が2行以上で1行目が : のパイプなら、その式を2行目以降に実行して選択全体を置き換える。1行目は残さない。例: :s/old/new の次が abc def old ghi と oldabc ddd なら abc def new ghi と newabc ddd。:quote | upper の次が xxx と yyy なら > XXX と > YYY。:sh dir | quote は dir の各行の頭に > 。:echo 2+3 | quote \">\" \"<\" は >5<。シェルの | は :sh \"xx | yy\" | quote のように括る。括らない | は段なので、段が無ければ何もしない（:sh dir | sort は何もしない）。:quote や :sel | upper だけの1行は何もしない。複数行で1行目が :echo だけなら何もしない。
2秒の連打が比べる文字と Ctrl+Z のあとに戻る文字は、/ 付きの選択、または自動選択のときは行全体。履歴は変えない。クリップボードは戻さない。
詳しくは :help complete :help home :help sh :help s :help showerror",
    },
    Page {
        name: "complete",
        group: "expand",
        summary: "選択語で履歴を補完して貼る",
        body: "\
打つ: Ctrl+9。設定画面には出さない。settings.json の complete は残っていても、保存しても消さない。
変わるのは前面。選択語で履歴を補完し、一覧の Enter と同じく展開して貼る。{{date}} は日付。{{n}} は貼れたときだけ進む。履歴の本文は変えない。先頭の / は要らない。
例: 選択 work が本文 hello work の行に当たれば、その本文を展開して貼る。#work はタグ work の行を一覧の順。#work hello はタグと本文。#work #home は両方。本文に無くても #alias:x と #alias:y なら x y で当たる。順は問わない。
展開が空なら何もしない。{{ask:}} か {{pick:}} がある行、#run #confirm #grab の行、本文がパイプの行は何もしない。# だけと #secret は何もしない。0件も何もしない。複数ならその順の最初。2秒以内の連打は、次が展開して空でなければ Ctrl+Z のあと次の候補。次が無い、空、対象外なら Ctrl+Z しない。Ctrl+A は送らない。Ubuntu は展開した文字をクリップボードへ置く。
詳しくは :help ctrl8 :help slash :help alias",
    },
    Page {
        name: "home",
        group: "expand",
        summary: "行頭まで選ぶキーをアプリごとに書く",
        body: "\
打つ: :set home shift+home
変わるのは settings.json の target_keys。いまの前面アプリの行頭選択キー。未設定は shift+home。展開キーで選択が空のとき、このキーを送る。すでに選択があるときは送らない。どの貼り付けも、このキーを勝手には送らない。
例: メモ帳で :set home shift+home と書くと、空の展開はそのキーで行頭まで選ぶ。
知らないキー、前面アプリが取れないときは書かない。home は変数名にできない。Ubuntu はキーを送らないので、書いても行頭までは選ばない。
詳しくは :help ctrl8 :help cut :help setpaste",
    },
    Page {
        name: "cut",
        group: "expand",
        summary: "切り取りキーをアプリごとに書く",
        body: "\
打つ: :set cut ctrl+x
変わるのは settings.json の target_keys。未設定は ctrl+x。どの動作も、このキーは送らない。設定だけ置く。
例: :set cut ctrl+shift+x で、いまの前面アプリの切り取りキーが変わる。
知らないキー、前面アプリが取れないときは書かない。cut は変数名にできない。
詳しくは :help home :help setcopy",
    },
    Page {
        name: "dd",
        group: "edit",
        summary: "行を消す",
        body: "\
打つ: dd
変わるのは一覧。その行を消す。#lock は残す。V なら範囲。前面とクリップボードは触らない。
例: hello の行で dd するとその行が無くなる。#lock の行は残る。
u で直前の1回を戻せる。
詳しくは :help lock :help u :help visual :help p",
    },
    Page {
        name: "yy",
        group: "edit",
        summary: "行をヤンクする",
        body: "\
打つ: yy または Y
変わるのは一覧の中のヤンク。本文は残る。V なら範囲。
例: hello で yy したあと :help p で下に同じ行を置ける。
行が無ければ何もしない。
詳しくは :help p :help ctrlc",
    },
    Page {
        name: "p",
        group: "edit",
        summary: "ヤンクした行を置く",
        body: "\
打つ: p は下、P は上
変わるのは一覧。直前の yy または dd の行を置く。前面には貼らない。
例: hello を yy して p すると、下に hello が増える。
ヤンクが無ければ何もしない。u で直前の1回を戻せる。
詳しくは :help yy :help dd",
    },
    Page {
        name: "u",
        group: "edit",
        summary: "直前の1回を取り消す",
        body: "\
打つ: u
変わるのは一覧。直前の1回（本文、削除、タグ、ピン、式、行の移動）を戻す。もう一度 u しても、それより前には戻らない。貼るだけの操作は対象外。
例: dd のあと u で、その行が戻る。
戻すものが無ければ何もしない。やり直しは :help redo
詳しくは :help dot",
    },
    Page {
        name: "redo",
        group: "edit",
        summary: "取り消した1回をやり直す",
        body: "\
打つ: Ctrl+R
変わるのは一覧。u で戻した1回をもう一度行う。新しい変更をすると、やり直す分は捨てる。
例: dd のあと u して Ctrl+R で、また消える。
やり直すものが無ければ何もしない。
詳しくは :help u",
    },
    Page {
        name: "dot",
        group: "edit",
        summary: "直前の変更を繰り返す",
        body: "\
打つ: .
変わるのは一覧。dd p P t T gp ga S J c :!! :sort g~ g: + - を繰り返す。o は繰り返さない。貼るだけは対象外。:s は対象外。
例: dd のあと別の行で . すると、その行も消える。
直前の変更が無ければ何もしない。
詳しくは :help dd :help gcolon",
    },
    Page {
        name: "o",
        group: "edit",
        summary: "下に空行を作る",
        body: "\
打つ: o
変わるのは一覧。いまの行の下に空行を作って編集する。V なら範囲の最後の下。起点がピンなら追加した行もピンで、そのすぐ隣。ピンでなければピンにしない。一覧が空なら先頭の1件。検索中もその行の隣に残る。
例: hello の下に空行ができ、編集になる。
. では繰り返さない。u で直前の1回を戻せる。
詳しくは :help O :help e",
    },
    Page {
        name: "O",
        group: "edit",
        summary: "上に空行を作る",
        body: "\
打つ: O
変わるのは一覧。いまの行の上に空行を作って編集する。V なら範囲の最初の上。ピンの隣の規則は :help o と同じ。
例: hello の上に空行ができ、編集になる。
詳しくは :help o",
    },
    Page {
        name: "e",
        group: "edit",
        summary: "その場で本文を編集する",
        body: "\
打つ: e。保存は Ctrl+Enter。取り消しは Esc
変わるのはその行の本文。式は変えない。式だけは :help from
例: hello を e で world にすると、本文は world。タグ欄の a b c は a と b と c。すでに付いている空白入りのタグは、そのチップで外す。
空のまま閉じても、その操作として残る。u で直前の1回を戻せる。
詳しくは :help E :help o",
    },
    Page {
        name: "E",
        group: "edit",
        summary: "外のエディタで編集する",
        body: "\
打つ: E
変わるのはその行の本文。nvim、無ければ $EDITOR、それも無ければメモ帳。終了したら読み戻す。開いているあいだ一覧は隠さない。
例: hello をメモ帳で world にすると、閉じたあと本文は world。
エディタが起動しなければ何もしない。
詳しくは :help e :help settings",
    },
    Page {
        name: "S",
        group: "edit",
        summary: "改行で行を分ける",
        body: "\
打つ: S
変わるのは一覧。その行を改行で複数行に分ける。
例: hello と world が改行で1行なら、S で2行になる。
改行が無ければ分かれない。u で直前の1回を戻せる。V でもこの行の分割。
詳しくは :help J",
    },
    Page {
        name: "J",
        group: "edit",
        summary: "選んだ行を1行にまとめる",
        body: "\
打つ: J
変わるのは一覧。V 中なら選んだ行を改行で1行にする。タグは和集合。ピンはどれかにあれば残る。#lock があるとまとめない。
例: hello と world を V して J で、1行になる。
V でなければまとめない。u で直前の1回を戻せる。
詳しくは :help visual :help S :help lock",
    },
    Page {
        name: "c",
        group: "edit",
        summary: "すぐ下に複製する",
        body: "\
打つ: c
変わるのは一覧。同じ本文の行をすぐ下に作る。新しい id になる。
例: hello で c すると、下にも hello。
u で直前の1回を戻せる。V なら範囲。
詳しくは :help yy :help p",
    },
    Page {
        name: "from",
        group: "edit",
        summary: "行に残した式だけを編集する",
        body: "\
打つ: :from または :from sel | upper
変わるのはその行の式。本文は :help e で編集する。引数が無ければ編集画面。引数があればその文字が式になる。式の無い行でも書ける。
例: :from sel | upper で、その行の式が sel | upper。
u で直前の1回を戻せる。失敗したら本文はそのまま。
詳しくは :help gcolon :help add",
    },
    Page {
        name: "gcolon",
        group: "edit",
        summary: "行に残した式をもう一度実行する",
        body: "\
打つ: g:
変わるのは、その式の結果。行に残した式をもう一度実行する。sel はいまの前面の選択、. はいまの本文。失敗したら本文はそのまま。式が無ければ何もしない。
例: 式が sel | kebab の行で g: すると、前面の選択を kebab にして実行する。
詳しくは :help from :help add :help dot",
    },
    Page {
        name: "gtilde",
        group: "edit",
        summary: "#grab の1行目と2行目を入れ替える",
        body: "\
打つ: g~
変わるのはその行の本文。#grab の1行目と2行目を入れ替える。3行目以降はそのまま。
例: 型の行と出力の行が入れ替わる。
2行目が無ければ何もしない。u で直前の1回を戻せる。
詳しくは :help grab",
    },
    Page {
        name: "t",
        group: "tag",
        summary: "タグを付ける",
        body: "\
打つ: t のあとタグ名
変わるのはその行のタグ。V なら範囲の全行。本文は変えない。
例: t work でタグ work が付く。t a b c で a と b と c。空白は区切り。空の語は捨てる。
空の名前は付かない。u で直前の1回を戻せる。外すのは :help T
詳しくは :help tab :help tags",
    },
    Page {
        name: "T",
        group: "tag",
        summary: "タグを外す",
        body: "\
打つ: T のあとタグ名
変わるのはその行のタグ。V なら範囲の全行。#lock を外すのも T。
例: T work でタグ work が外れる。T a b c で a と b と c を外す。付いていない語は無視する。空白は区切り。
付いていなければ何もしない。u で直前の1回を戻せる。
詳しくは :help t :help lock",
    },
    Page {
        name: "secret",
        group: "tag",
        summary: "一覧では隠して貼るときは普通",
        body: "\
タグ: #secret
変わるのは一覧の表示。本文は •••• に見える。貼り付けは普通に本文を出す。
例: パスワードの行に #secret を付けると、一覧は ••••、Enter では本文が貼る。
検索の #secret だけでは補完しない。詳しくは :help complete
付けるのは :help t 、外すのは :help T",
    },
    Page {
        name: "alias",
        group: "tag",
        summary: "別の語でも検索に当てる",
        body: "\
タグ: #alias:foo
変わるのは検索の当たり。本文に foo が無くても /foo で当たる。#alias:x と #alias:y なら x y でも当たる。順は問わない。本文は差し込まない。
例: 本文が長い行に #alias:foo があると、/foo でその行。
詳しくは :help slash :help complete",
    },
    Page {
        name: "apptag",
        group: "tag",
        summary: "そのアプリのときだけ行を出す",
        body: "\
タグ: #app:chrome
変わるのは一覧に出すかどうか。前面のアプリ名が chrome のときだけ出す。#app:code と複数ならどれか。ブラウザはページが違っても当たる。並び順は変えない。
例: #app:chrome の行は、chrome が前面のときだけ見える。
付けるのは :help ga または :help t
詳しくは :help notag :help app",
    },
    Page {
        name: "notag",
        group: "tag",
        summary: "そのアプリのときは出さない",
        body: "\
タグ: #not:chrome
変わるのは一覧に出すかどうか。そのアプリのときは出さない。#app: と両方なら、アプリに当たって #not に当たらないときだけ出す。
例: #not:chrome の行は、chrome が前面だと隠れる。
詳しくは :help apptag",
    },
    Page {
        name: "run",
        group: "tag",
        summary: "貼るときコマンドを実行する",
        body: "\
タグ: #run
変わるのは貼る文字。自分で付けた行だけ、貼るときコマンドを実行する。自動では付けない。{{sh: コマンド}} が無ければ本文全体がコマンド。貼る前に出力を出して Enter で貼る。Esc は中止。
例: #run の本文が echo hi なら、貼る前に hi が出て、Enter で前面は hi。
失敗したら貼らない。詳しくは :help sh :help confirm",
    },
    Page {
        name: "confirm",
        group: "tag",
        summary: "貼る前に展開後を出して確認する",
        body: "\
タグ: #confirm
変わるのは貼る前の確認。展開後を出して Enter で貼る。Esc は中止。#run があれば :help run の確認で足りる。
例: {{date}} の行に #confirm があると、今日の日付を見てから Enter で貼る。
中止したら貼らない。選択は残る。
詳しくは :help ctrl8 :help run",
    },
    Page {
        name: "filetag",
        group: "tag",
        summary: "本文のパスの中身を貼る",
        body: "\
タグ: #file
変わるのは貼る文字。本文のパスのファイル内容を貼る。#run が先。ドロップすると #path と一緒に付く。
例: 本文が C:\\tmp\\a.txt で #file なら、そのファイルの中身を貼る。
読めなければ貼らない。
詳しくは :help path :help drop :help run",
    },
    Page {
        name: "lock",
        group: "tag",
        summary: "削除やまとめで残す",
        body: "\
タグ: #lock
変わるのは消えるかどうか。dd、:clear、J で消えない。外すのは :help T
例: #lock の行で dd しても残る。
移動の + / - は #lock でも動く。詳しくは :help plus
詳しくは :help dd :help clear :help J",
    },
    Page {
        name: "slottag",
        group: "tag",
        summary: "Ctrl+Shift+番号の行にする",
        body: "\
タグ: #slot:3
変わるのは Ctrl+Shift+3 が貼る行。複数なら一覧の先。無ければ並びの3番目。
例: #slot:1 を付けると Ctrl+Shift+1 はその行。
詳しくは :help slots",
    },
    Page {
        name: "grab",
        group: "tag",
        summary: "選択が型に当たれば穴を埋めて貼る",
        body: "\
タグ: #grab
変わるのは貼る文字。1行目が型、2行目が出力。穴は <名前>。前面の選択が型に当たれば2行目を埋めて貼る。当たらなければ何もしない。値は :set に残さない。Ubuntu はクリップボードを照合する。
例: 型 get<Name> と出力 name の行に、選択 getUser が当たれば user を埋める。
行の入れ替えは :help gtilde
詳しくは :help complete",
    },
    Page {
        name: "url",
        group: "tag",
        summary: "URL として付く目印",
        body: "\
タグ: #url
変わるのは登録時のタグ。Ctrl+4 の本文が http(s) なら付く。動きは目印だけ。開くのは :help open
例: https://example.com を Ctrl+4 すると #url が付く。
詳しくは :help register :help path",
    },
    Page {
        name: "path",
        group: "tag",
        summary: "パスとして付く目印",
        body: "\
タグ: #path
変わるのは登録時のタグ。Ctrl+4 の本文がパスなら付く。ドロップでも付く。動きは目印だけ。中身を貼るのは :help filetag
例: C:\\tmp\\a.txt を Ctrl+4 すると #path が付く。
詳しくは :help url :help drop",
    },
    Page {
        name: "register",
        group: "tag",
        summary: "前面の選択を一覧へ登録する",
        body: "\
打つ: Ctrl+4
変わるのは一覧。前面の選択テキストを先頭へ足す。http(s) なら #url、パスなら #path。同じ本文は先頭へ移すだけで、2件目は作らない。登録後もその文字がクリップボードに残る。画像だけのクリップボードはテキストとして読まない。
例: 選択 hello で Ctrl+4 すると、一覧の先頭が hello。
選択が空なら足さない。
詳しくは :help url :help path :help drop",
    },
    Page {
        name: "gp",
        group: "pin",
        summary: "ピン留めして先頭に残す",
        body: "\
打つ: gp
変わるのは一覧のピン。先頭に固定する。V なら範囲。もう一度で外す。
例: hello で gp すると、その行がピンの塊に入る。
u で直前の1回を戻せる。順は :help plus
詳しくは :help pin :help ga",
    },
    Page {
        name: "ga",
        group: "pin",
        summary: "前面アプリの #app: を付ける",
        body: "\
打つ: ga
変わるのはその行の #app:。いまの前面アプリにする。ほかの #app: は外す。V なら範囲。
例: chrome が前面のとき ga で #app:chrome。以前の #app:code は外れる。
前面アプリが取れなければ書かない。
詳しくは :help apptag :help gp",
    },
    Page {
        name: "plus",
        group: "pin",
        summary: "1つ上へ動かす",
        body: "\
打つ: +
変わるのは一覧の並び。ピン以外の行も1つ上へ動かす。V ならその範囲ごと。ピンの塊と、ピンでない塊は混ぜない。塊の端では止まる。本文・タグ・ピンは変えない。#lock でも動く。
例: ピンでない2行目で + すると1行目になる。最初のピンでない行の + は、最後のピンを越えない。
u で直前の1回を戻せる。
詳しくは :help minus :help gp",
    },
    Page {
        name: "minus",
        group: "pin",
        summary: "1つ下へ動かす",
        body: "\
打つ: -
変わるのは一覧の並び。1つ下へ動かす。塊の規則は :help plus と同じ。文字を小さくするのは :help zoom
例: ピンの2行目で - すると、ピンの中で1つ下。最後のピンの - は、ピンでない行へは入らない。
u で直前の1回を戻せる。
詳しくは :help plus",
    },
    Page {
        name: "slash",
        group: "search",
        summary: "本文とタグで絞る",
        body: "\
打つ: /
変わるのは一覧の絞り。照合は本文の先頭 4KB まで。それより後ろの語は当たらない。#tag でタグ。#alias:foo は foo でも当たる。#alias:x と #alias:y なら x y でも当たる。順は問わない。Tab は一覧へ（絞りと検索欄は残る）。# の候補がある Tab はタグ補完。
例: /hello で本文に hello がある行。/#work でタグ work。
0件でも失敗としては覚えない。外すのは :help esc
Ctrl+N / Ctrl+P でも移動する。
詳しくは :help tab :help alias :help complete",
    },
    Page {
        name: "V",
        group: "visual",
        summary: "範囲を選んでまとめて操作する",
        body: "\
打つ: V。広げるのは j / k。解除は Esc
変わるのは選択範囲。Enter は改行つなぎ。:join は区切り。:quote は行頭。J は1行にまとめる（#lock があるとまとめない）。c は複製。S は分割。:sort は本文の順。:!!sh は各行をフィルタ。dd / yy / t / T / gp / ga / + / - は範囲に効く。dd は #lock を残す。
例: hello と world を V して Enter で、前面は hello 改行 world。
範囲が無ければ1行の操作になる。
詳しくは :help enter :help join :help J :help sort :help bang",
    },
    Page {
        name: "help",
        group: "colon",
        summary: "この使い方を出す",
        body: "\
打つ: :help または :help quote
変わるのはこの画面。引数なしは目次。各行の :help 名前 で、その操作だけを出す。その文字をクリックしても開く。戻る・進むはボタンと Alt+← / Alt+→。j / k でスクロール。
例: :help quote で、行頭を付ける説明と、選択 hello が * hello になる例。
名前が無ければ ない と、同じ目次を出す。アプリの動きは変えない。
詳しくは :help keys :help template",
    },
    Page {
        name: "showerror",
        group: "colon",
        summary: "直前に失敗した理由を1つ出す",
        body: "\
打つ: :showerror
変わるのはこの画面。直前に期待どおりにならなかった理由を1つ出す。展開、補完、貼り付け、パイプ、:sh、読めない JSON、段が違う、選択が空、クリップボードに書けない、当たらない、時間切れ。標準エラーがあればその文字。成功すると消える。次の失敗で上書きする。空なら何も出さない。
貼らない。履歴もクリップボードも触らない。保存しない。終了で消える。| showerror という段にはしない。Esc で閉じる。一覧の検索が 0 件、:s/old/ で選択が空になるのは失敗にしない。`hataclip.exe` は `--showerror` が付いたときだけ、その実行の理由を標準エラーに出す。旗が無くても終了コードは 1。一覧が覚えている理由は出さない。ファイルには残さない。Ubuntu も同じ。
例: :sh で失敗したあと :showerror でその理由。
詳しくは :help show :help sh",
    },
    Page {
        name: "sh",
        group: "colon",
        summary: "コマンドを実行して貼る",
        body: "\
打つ: :sh dir
変わるのは前面（標準出力を貼る）。履歴は触らない。一覧で打つ最初の :sh は stdin なし。Ctrl+8 の1行で後ろに段があれば stdin なしで実行して選択を置き換える。シェルの | は引用符の中（:sh \"xx | yy\" | quote）。括らない段が読めなければ何もしない（:sh dir | sort は何もしない）。複数行で1行目なら、2行目以降を stdin に受け取る。次の :sh は左の結果を stdin に受け取る。
失敗したら貼らない。確認は、展開キーのときだけ。一覧で打つ :sh と CLI は確認しない。Ubuntu でもシェルは動く。
例: :sh echo hi で前面は hi。
行に付けて実行するのは :help run 。本文を書き換えるのは :help bang 。選択行を stdin にするのは :help dotsh 。直前の再実行は :help at
詳しくは :help ctrl8 :help pipe",
    },
    Page {
        name: "dotsh",
        group: "colon",
        summary: "選択行を stdin にして実行する",
        body: "\
打つ: :.!sh xxx
変わるのは前面。カレント行（V なら改行つなぎ）を stdin に流して、標準出力を貼る。履歴は触らない。
例: hello の行で :.!sh のコマンドが、その文字を受け取って結果を貼る。
失敗したら貼らない。
詳しくは :help sh :help bang",
    },
    Page {
        name: "bang",
        group: "colon",
        summary: "各行をコマンドで書き換える",
        body: "\
打つ: :!!sh xxx
変わるのは一覧の本文。各行を stdin に流して本文を書き換える。貼らない。式が残るので :help gcolon でもう一度。V ならその範囲。
例: 各行を大文字にするコマンドなら、本文がその結果に変わる。
失敗したら本文はそのまま。u で直前の1回を戻せる。
詳しくは :help sh :help from",
    },
    Page {
        name: "at",
        group: "colon",
        summary: "直前の :sh をもう一度実行する",
        body: "\
打つ: :@
変わるのは前面。直前の :sh / :.!sh のスクリプトをもう一度。stdin はいまの選択。
例: :sh echo hi のあと :@ で、もう一度 hi。
失敗したら貼らない。展開キーで :@ を含む範囲は、実行前に確認する。
詳しくは :help sh :help ctrl8",
    },
    Page {
        name: "echo",
        group: "colon",
        summary: "四則の結果を貼る",
        body: "\
打つ: :echo 2+3
変わるのは前面（計算結果を貼る）。履歴は触らない。* と / が先。() と :set の変数が使える。計算できなければ何もしない。
例: :echo 3+4 で前面は 7。:echo 2+3 | quote は計算してから段を掛ける。
複数行で1行目が :echo だけなら何もしない（下の行は残る）。:echo 2+3 の次が notes なら置き換えない。パイプの echo は左を見ず、式の結果で流れを置き換える。
詳しくは :help show :help set",
    },
    Page {
        name: "export",
        group: "colon",
        summary: "一覧を Markdown で書く",
        body: "\
打つ: :export path
変わるのはそのファイル。一覧は残る。
例: :export C:\\tmp\\hataclip.md
書けなければ何もしない。
詳しくは :help import",
    },
    Page {
        name: "import",
        group: "colon",
        summary: "Markdown から一覧へ足す",
        body: "\
打つ: :import path
変わるのは一覧。Markdown から行を足す。
例: :import C:\\tmp\\hataclip.md で、ファイルの行が足される。
読めなければ何もしない。
詳しくは :help export",
    },
    Page {
        name: "log",
        group: "colon",
        summary: "選択行をファイルへ追記する",
        body: "\
打つ: :log または :log notes.log
変わるのはそのファイル（選択行を末尾へ追記）。前面には貼らない。履歴は触らない。未指定の出力先は変数 defaultLogFileName。未定義か空、選択が空、展開できない、書けない、なら何もしない。V は改行つなぎ。既存ファイルの改行に合わせる。無いファイルは CRLF。| log も同じ。:echo 3+4 | log は 7 を足して前面には貼らない。Ctrl+8 で1行目が log、次が abc と def ならその2行を足す。選択は置き換えない。
例: :set defaultLogFileName=notes.log のあと hello を :log で notes.log に hello。
詳しくは :help set :help ctrl8",
    },
    Page {
        name: "clear",
        group: "colon",
        summary: "ピンと #lock 以外を消す",
        body: "\
打つ: :clear のあと :clear yes
変わるのは一覧。ピンと #lock 以外を消す。yes が無いと確認の文字を出すだけ。
例: :clear yes で、ピンでも #lock でもない行が消える。
yes が無いと消えない。
詳しくは :help lock :help dedup",
    },
    Page {
        name: "dedup",
        group: "colon",
        summary: "同じ本文を1件にまとめる",
        body: "\
打つ: :dedup のあと :dedup yes
変わるのは一覧。同じ本文は1件にまとめる。yes が無いと確認だけ。
例: hello が2行あるとき :dedup yes で1行になる。
yes が無いとまとまらない。
詳しくは :help clear",
    },
    Page {
        name: "settings",
        group: "colon",
        summary: "settings.json をエディタで開く",
        body: "\
打つ: :settings
変わるのは外のエディタ。settings.json を開く。一覧の本文は触らない。
例: :settings で設定ファイルが開く。
エディタが無ければ何もしない。
詳しくは :help E :help set",
    },
    Page {
        name: "tags",
        group: "colon",
        summary: "タグと件数を出す",
        body: "\
打つ: :tags
変わるのはこの画面。タグ一覧と件数を出す。一覧自体は変わらない。
例: work が3行なら、work と 3 が出る。
タグが無ければ空に近い。
詳しくは :help t :help tab",
    },
    Page {
        name: "set",
        group: "colon",
        summary: "変数を残す",
        body: "\
打つ: :set a=hello または :set a= または | set a
変わるのは変数。貼らない。:set だけで一覧。:set a= で消す。引用の中の \\\" は \"、\\t はタブ、\\n は改行。:set a=\"say \\\"hi\\\"\" の値は say \"hi\"。:set a+=1 は貼って成功したあと 1 増やす。無い変数は 1。数字以外には付かない。| set a はパイプの結果をその変数へ。名前が不正なら何もしない。paste、copy、home、cut は変数名にできない。
例: :set a=\"{{date}}\" のあと、貼るときに :help var で展開する。:set では :sh を実行しない。
詳しくは :help setcopy :help setpaste :help home :help cut :help var",
    },
    Page {
        name: "setcopy",
        group: "colon",
        summary: "前面アプリのコピーキーを書く",
        body: "\
打つ: :set copy ctrl+shift+c
変わるのは settings.json の target_keys。いまの前面アプリのコピー。未設定は ctrl+c。
例: 端末で :set copy ctrl+shift+c
知らないキー、前面アプリが取れないときは書かない。copy は変数名にできない。
詳しくは :help setpaste :help home",
    },
    Page {
        name: "setpaste",
        group: "colon",
        summary: "前面アプリの貼り付けキーを書く",
        body: "\
打つ: :set paste shift+insert
変わるのは settings.json の target_keys。いまの前面アプリの貼り付け。未設定は ctrl+v。
例: PuTTY で :set paste shift+insert
知らないキー、前面アプリが取れないときは書かない。paste は変数名にできない。
詳しくは :help setcopy :help home",
    },
    Page {
        name: "n",
        group: "colon",
        summary: "連番の初期値を決める",
        body: "\
打つ: :n 100 または :n または :n 1
変わるのは {{n}} の初期値。settings.json に残る。:n だけでいまの値。貼ったあと Ctrl+Enter で増える。閉じると初期値に戻る。ge やコピーでは進まない。
例: :n 100 のあと {{n}} を Ctrl+Enter で貼ると 100、次は 101。
詳しくは :help stay :help template",
    },
    Page {
        name: "s",
        group: "colon",
        summary: "選択の文字を置き換えて貼る",
        body: "\
打つ: :s/old/new
変わるのは前面へ貼る文字。履歴は触らない。選択行の old を全部 new にする。正規表現は使わない。old が空なら何もしない。old が無くても残りを貼る。結果が空でも選択は空になる。V は改行つなぎ。u では戻らない。本文を直すのは :help e 。. の対象外。
例: 選択行 hello に :s/ell/ipp で前面は hippo。
段 :sel | s/old/new は前面の選択を置換して貼る。new は2つ目の / の後ろ全部。Ctrl+8 で1行目が :s/old/new なら2行目以降を置換して選択全体を置き換える。1行だけは何もしない。
詳しくは :help ctrl8 :help sel",
    },
    Page {
        name: "sort",
        group: "colon",
        summary: "選んだ行を本文の順にする",
        body: "\
打つ: :sort
変わるのは V した行の並び（本文の順）。ピンとタグは、行について移動する。1行だけなら何もしない。u で戻せる。
例: b と a を V して :sort で a が先。
V でなければ何もしない。
詳しくは :help visual :help plus",
    },
    Page {
        name: "open",
        group: "colon",
        summary: "URL かパスを開く",
        body: "\
打つ: :open
変わるのは別アプリ。http/https はブラウザ。パスは Explorer。一覧とクリップボードは触らない。URL でもパスでもなければ何もしない。| open も同じ。
例: https://example.com を :open でブラウザが開く。
キーに付けるなら :help map 。ドロップは :help drop
詳しくは :help url :help path",
    },
    Page {
        name: "sel",
        group: "pipe",
        summary: "前面の選択を流れに入れる",
        body: "\
打つ: :sel | upper のようにパイプの段。{{sel}} は貼る直前に同じ選択へ変わる。
変わるのはそのパイプの結果（行き先が無ければ前面）。履歴は触らない。一覧では sel は前面の選択、. は一覧の行。Ctrl+8 の複数行ではどちらも2行目以降。Windows は Ctrl+C の 200ms 後を読む。クリップボードは戻さない。空なら何もしない。Ubuntu では履歴の行にある sel は実行しない。Ctrl+8 の複数行では選択はもう読んであるので sel があっても実行し、結果はクリップボードに置く。
{{sel|clip}} は空なら隣。隣がトークンなら展開し、違う文字ならそのまま。
例: 前面の hello に :sel | upper で前面は HELLO。
詳しくは :help clip :help s :help ctrl8",
    },
    Page {
        name: "clip",
        group: "pipe",
        summary: "クリップボードを読む、または書く",
        body: "\
打つ: :clip | show のように先頭の段、行き先の | clip、または途中の clip。{{clip}} は貼る直前のクリップボード。空なら空。
先頭で流れが無ければクリップボードを読む。流れがある途中の clip は、その文字をクリップボードへ書いて同じ文字を次へ渡す。空ならクリップボードは変えない。| clip は結果をクリップボードへ書き、前面には貼らない。末尾の > clip も何もしない。画像だけのクリップボードはテキストとして読まない。どの動作でも、読んだあとに元の文字へは戻さない。
例: :sel | clip | upper は選択をクリップボードへ置き、大文字を前面へ貼る。:clip | upper | clip はクリップボードの hello を HELLO にして戻す。
詳しくは :help add :help show :help sel",
    },
    Page {
        name: "add",
        group: "pipe",
        summary: "結果を一覧へ1件足す",
        body: "\
打つ: :sel | kebab | add
変わるのは一覧（結果を1件足す）。前面とクリップボードは触らない。できた行には add より前の式が残る（:sel | kebab | add なら sel | kebab）。空なら足さない。途中の add は同じ文字を次へ渡す。流れが無い途中の add は何もしない。
例: 前面 userName で本文 user-name、式 sel | kebab。:sel | kebab | add | quote は一覧に user-name、前面は > user-name。
g: でもう一度。式の編集は :help from
詳しくは :help gcolon :help clip",
    },
    Page {
        name: "show",
        group: "pipe",
        summary: "結果をこの画面に出す",
        body: "\
打つ: :echo 3+4 | show
変わるのはこのヘルプ画面。前面には貼らない。空なら出さない。hataclip.exe の echo 3+4 | show は標準出力に出す。端末に収まらないときは `j` / `k` で1行ずつ。端で止まる。`q` か Esc で終わる。パイプしたときは全文を出してキーは読まない。
例: :echo 3+4 | show で 7。:clip | show はクリップボードをここに出す。
入力中は結果を出さない。Ctrl+Enter で結果の先頭12行をここに出す。式を書き換えたら消える。sel を含む式は出さない。: では一覧を隠さず Ctrl+C は送らない。入力欄とプレビューで一覧が短くなっても、選択行（V なら範囲の先頭）が見えるまでスクロールする。Enter で sel が要るときはその場で読む。sh を含む、段が不正、json が読めない、入力が空のときは出さない。12行を超えたら末尾に ...。
詳しくは :help showerror :help echo :help scroll",
    },
    Page {
        name: "camel",
        group: "pipe",
        summary: "camelCase にする",
        body: "\
打つ: :camel または :sel | camel
変わるのは前面。履歴は触らない。
例: foo_bar は fooBar。
入力が空なら何もしない。
詳しくは :help pascal :help snake :help kebab",
    },
    Page {
        name: "pascal",
        group: "pipe",
        summary: "PascalCase にする",
        body: "\
打つ: :pascal または :sel | pascal
変わるのは前面。履歴は触らない。
例: foo_bar は FooBar。
入力が空なら何もしない。
詳しくは :help camel",
    },
    Page {
        name: "snake",
        group: "pipe",
        summary: "snake_case にする",
        body: "\
打つ: :snake または :sel | snake
変わるのは前面。履歴は触らない。
例: getUserName は get_user_name。
入力が空なら何もしない。
詳しくは :help kebab :help slots",
    },
    Page {
        name: "kebab",
        group: "pipe",
        summary: "kebab-case にする",
        body: "\
打つ: :kebab または :sel | kebab
変わるのは前面。履歴は触らない。
例: helloWorld は hello-world。前面が helloWorld のとき :sel | kebab は hello-world。
入力が空なら何もしない。
詳しくは :help snake :help add",
    },
    Page {
        name: "upper",
        group: "pipe",
        summary: "大文字にする",
        body: "\
打つ: :upper または :sel | upper
変わるのは前面。履歴は触らない。段が1つでも同じ。:. | upper は一覧の行を大文字にして貼る。
例: hello は HELLO。
入力が空なら何もしない。
詳しくは :help lower :help sel",
    },
    Page {
        name: "lower",
        group: "pipe",
        summary: "小文字にする",
        body: "\
打つ: :lower または :sel | lower
変わるのは前面。履歴は触らない。
例: AbC は abc。
入力が空なら何もしない。
詳しくは :help upper",
    },
    Page {
        name: "json",
        group: "pipe",
        summary: "JSON を整形する",
        body: "\
打つ: :json
変わるのは前面。履歴は触らない。読めなければ何もしない。
例: {\"a\":1} は改行付きになる。
詳しくは :help xml :help get :help put",
    },
    Page {
        name: "xml",
        group: "pipe",
        summary: "XML を整形する",
        body: "\
打つ: :xml
変わるのは前面。履歴は触らない。読めなければ何もしない。
例: 1行の XML が改行付きになる。
詳しくは :help json",
    },
    Page {
        name: "split",
        group: "pipe",
        summary: "区切りで列に分ける",
        body: "\
打つ: :sel | split ,
区切りは必須。タブは split \"\\t\"。変わるのは次の段へ渡す文字（列はタブ1つでつなぐ）。区切りが無ければ何もしない。
例: a,b,c は a と b と c がタブでつながる。
詳しくは :help col",
    },
    Page {
        name: "col",
        group: "pipe",
        summary: "分けた列の1つを取る",
        body: "\
打つ: col 2
変わるのはその列だけ。1始まり。短い行は落ちる。1未満や整数でないなら何もしない。
例: a,b,c を split , | col 2 で b。
詳しくは :help split",
    },
    Page {
        name: "get",
        group: "pipe",
        summary: "JSON の値を取り出す",
        body: "\
打つ: get /name
変わるのはその値。文字列はそのまま、数と真偽と null は文字、オブジェクトと配列は JSON。読めないか無いなら何もしない。
例: {\"name\":\"hata\"} の get /name は hata。:clip | json | get /items/0/name | show は hatamon。
詳しくは :help put :help json :help each",
    },
    Page {
        name: "diff",
        group: "pipe",
        summary: "行の差を出す",
        body: "\
打つ: :sel | diff clip または diff .
変わるのは前面（- の行のあと + の行）。引数は . か clip。同じ、または片側が空なら何もしない。履歴は触らない。
例: 選択 a 改行 b とクリップボード a 改行 c で、- b と + c。:clip | diff . | show はクリップボードにあって選択に無い行を - 、選択にあってクリップボードに無い行を + 。共通行は出さない。
詳しくは :help only",
    },
    Page {
        name: "only",
        group: "pipe",
        summary: "片方にだけある行を出す",
        body: "\
打つ: :sel | only clip
変わるのは前面（左にだけある行）。右にもある行は出さない。全部同じなら何もしない。
例: a 改行 b と a なら b。:. | only clip | show は選択にあってクリップボードに無い行。
詳しくは :help diff",
    },
    Page {
        name: "filter",
        group: "pipe",
        summary: "含む行だけ残す",
        body: "\
打つ: :filter \".txt\" または :filter not \".txt\"
変わるのは前面（含む行だけ、not は含まない行）。履歴は触らない。正規表現は使わない。大文字と小文字は区別する。括らなければ空白の後ろ全部。針が not なら :filter \"not\"。1行も残らない、引数が空、流れが空なら何もしない。選択は空にしない。
例: a.txt と b.rs に :filter \".txt\" で前面は a.txt。:sh dir | filter \".txt\" も同じ。Ctrl+8 の1行は sh か echo が先にあるときだけ。:filter だけの1行は何もしない。
詳しくは :help sh",
    },
    Page {
        name: "put",
        group: "pipe",
        summary: "JSON の位置へ値を書く",
        body: "\
打つ: put /n 2
変わるのはその JSON。引数はポインタ、空白、値。値は引用できる。JSON として読めたらその値、読めなければ文字列。流れが JSON でない、またはポインタが無いときは何もしない。
例: {\"name\":\"x\"} に put /n 2 で n が 2。:sel | put /n 2 | json は n が 2 の JSON を整形して貼る。
詳しくは :help get :help json",
    },
    Page {
        name: "each",
        group: "pipe",
        summary: "行ごとに後ろの段を実行する",
        body: "\
打つ: :sel | each | upper
変わるのは前面。後ろの段を行ごとに実行し、成功した行だけを改行でつなぐ。失敗した行は落ちる。全部失敗なら何もしない。後ろに sh があると不正で何もしない。
例: a 改行 b を each | upper で A 改行 B。:sel | each | get /id は各行の id。
詳しくは :help upper :help get",
    },
    Page {
        name: "date",
        group: "template",
        summary: "今日の日付に置き換わる",
        body: "\
書く: {{date}} または {{date %Y%m%d}} または {{date-1d}}
貼る直前だけ置き換わる。履歴の本文は変わらない。`:` の代わりに空白でもよい。{{date}} は 2026/09/20 のような今日。{{date:%Y%m%d}} は chrono の strftime。{{date-1d}} は昨日。{{date-1d:%Y%m%d}} {{date+2d}} {{date+1w}} {{date+1m}} も可。
例: {{date:%Y}} は 2026。
失敗はしない。Ubuntu でも動く。
詳しくは :help time :help template",
    },
    Page {
        name: "time",
        group: "template",
        summary: "いまの時刻に置き換わる",
        body: "\
書く: {{time}}
貼る直前だけ置き換わる。履歴は変わらない。
例: 10:54。
詳しくは :help date",
    },
    Page {
        name: "ask",
        group: "template",
        summary: "貼る前に入力を聞く",
        body: "\
書く: {{ask:名前}}
貼る前に入力を聞く。同じ名前は1回。履歴は、トークンのまま残る。
例: {{ask:name}} に hata と入れるとその文字になる。
Esc は中止で貼らない。{{pick:}} より先に全部聞く。
詳しくは :help pick",
    },
    Page {
        name: "pick",
        group: "template",
        summary: "貼る前に候補から選ぶ",
        body: "\
書く: {{pick list: prod, stg}} または {{pick tag:env}} または {{pick search: \"xx\"}}
貼る前に候補から選ぶ。j / k と Enter。候補が 9 個までなら 1〜9 でその番。10 個以上は j / k。Esc は中止。同じ候補は1回。{{ask:}} があるときは先に全部聞く。{{pick tag:env}} はタグ env の本文。{{pick search: \"xx\"}} は / と同じ当たりの行。自分自身は入れない。無ければ聞かない。昔の {{pick: a, b}} は空で聞かない。
例: {{pick list: prod, stg}} で prod を選ぶと prod。
詳しくは :help ask :help slash",
    },
    Page {
        name: "app",
        group: "template",
        summary: "前面アプリのプロセス名",
        body: "\
書く: {{app}}
貼る直前に前面アプリのプロセス名へ変わる。取れなければ空。履歴は変わらない。Ubuntu は空。
例: メモ帳なら notepad。
行をアプリで出し分けるのは :help apptag
詳しくは :help front :help when",
    },
    Page {
        name: "front",
        group: "template",
        summary: "前面ウィンドウのタイトル",
        body: "\
書く: {{front}}
貼る直前に前面ウィンドウのタイトルへ変わる。取れなければ空。Ubuntu は空。{{front|無題}} は空なら 無題。
例: タイトルが無ければ、{{front|無題}} は 無題。
詳しくは :help app",
    },
    Page {
        name: "focus",
        group: "template",
        summary: "入力欄の種類",
        body: "\
書く: {{focus}}
貼るときに、フォーカスしている入力欄の種類。Edit、Document、ComboBox。常時監視はしない。取れなければ空。Ubuntu は空。
例: テキスト欄なら Edit。
詳しくは :help when",
    },
    Page {
        name: "when",
        group: "template",
        summary: "条件に当たる区間だけ残す",
        body: "\
書く: {{when app: chrome}} または {{when var: a: \"AAA\"}} または {{when focus: Edit}} または {{when}}
その条件のときだけ区間を残す。複数は {{when app: chrome, msedge}}。{{when var: a: \"AAA\"}} は :set のそのままの文字列。{{when}} はどれにも当たらないとき。{{when chrome}} は展開せずそのまま。Ubuntu では app と focus は空なので、その枝は残らない。
例: chrome のときだけ {{when app: chrome}}中{{when}} の「中」が残る。
詳しくは :help app :help focus :help var",
    },
    Page {
        name: "uuid",
        group: "template",
        summary: "UUID を作る",
        body: "\
書く: {{uuid}}
貼る直前に UUID v4 へ変わる。履歴は変わらない。
例: 貼るたびに別の UUID。
詳しくは :help template",
    },
    Page {
        name: "user",
        group: "template",
        summary: "ログイン名",
        body: "\
書く: {{user}}
貼る直前にログイン名へ変わる。取れなければ空。履歴は変わらない。
例: hatamon。
詳しくは :help host :help env",
    },
    Page {
        name: "host",
        group: "template",
        summary: "コンピュータ名",
        body: "\
書く: {{host}}
貼る直前にコンピュータ名へ変わる。取れなければ空。履歴は変わらない。
例: その PC の名前。
詳しくは :help user",
    },
    Page {
        name: "env",
        group: "template",
        summary: "環境変数の値",
        body: "\
書く: {{env:USERPROFILE}}
その環境変数。無ければ空。履歴は変わらない。
例: {{env:USERPROFILE}} はユーザのホーム。
詳しくは :help var",
    },
    Page {
        name: "var",
        group: "template",
        summary: "変数の値を展開する",
        body: "\
書く: {{var:a}} または {{var a}}
:set a= の値。中の {{date}} も展開する。値が :sh dir ならそのとき実行。:set では実行しない。名前に {{var:b}} を書ける（{{var: {{var: b}}}}）。無い変数は空。
例: :set a=hello のあと {{var:a}} は hello。:set a+=1 は貼って成功したあと 1 増やす。
詳しくは :help set :help n",
    },
    Page {
        name: "wait",
        group: "template",
        summary: "貼る途中で待つ",
        body: "\
書く: {{wait:200}}
貼る途中で 200ms 待つ。最大 5 秒。それより大きいと 5 秒。
例: id{{wait:200}}pass は id のあと 200ms 待って pass。
詳しくは :help typeto",
    },
    Page {
        name: "typeto",
        group: "template",
        summary: "貼る途中でキーを送る",
        body: "\
書く: {{type:<Tab>}}
貼る途中でキーを送る。id{{type:<Tab>}}pass は id を貼って Tab を押して pass を貼る。{{type:<Ctrl+A>abc<Enter>}} {{type:<Ctrl+Shift+A>}} も可。矢印は <Up> <Down> <Left> <Right>。F キーは <F1>〜<F24>。文字は 1 文字ずつ。Tab Enter Esc Space BS Del Home End PgUp PgDn と Ctrl/Shift/Alt+それら。
例: id{{type:<Tab>}}pass で、id、Tab、pass。
Ubuntu はキーを送らない。1文字ずつ貼るコマンドは :help type
詳しくは :help wait :help type",
    },
    Page {
        name: "tagbody",
        group: "template",
        summary: "そのタグの本文を差し込む",
        body: "\
書く: {{tag:work}}
いまの一覧でタグ work の本文。並びどおり、改行つなぎ。差し込んだ本文も展開する。同じタグを二度辿ったら空。その行の #run などは見ない。
例: work の行が a と b なら、{{tag:work}} は a 改行 b。
詳しくは :help t :help pick",
    },
    Page {
        name: "winmove",
        group: "window",
        summary: "一覧を動かし、大きさを変える",
        body: "\
打つ: Ctrl+矢印で 24px 移動。Ctrl+Shift+←→ で幅、Ctrl+Shift+↑↓ で高さ。
変わるのは一覧ウィンドウ。下限 200×140。本文は変えない。閉じても位置と大きさは残る。
例: Ctrl+→ で右へ 24px。
ドラッグでも動く。ドラッグのあいだは隠さない。
詳しくは :help zoom :help blur",
    },
    Page {
        name: "zoom",
        group: "window",
        summary: "一覧の文字を大きく、小さくする",
        body: "\
打つ: Ctrl+; で大きく、Ctrl+- で小さく
変わるのは一覧ウィンドウの文字。本文・検索・ヘルプ・タグなど全部。1px ずつ。9〜32。閉じても settings.json に残る。行を動かす + / - とは別。; だけは :help semicolon
例: Ctrl+; で文字が1px大きくなる。
端ではそれ以上変わらない。
詳しくは :help winmove :help plus",
    },
    Page {
        name: "map",
        group: "keymap",
        summary: "通常モードのキーを付け替える",
        body: "\
打つ: :map lhs rhs または :map
変わるのはキー割り当て。再帰しない。同じ lhs は上書き。lhs は j dd gT か <leader>*。rhs はキー列か :quote / :quote \"* \"（末尾 <CR> は要らない）。Esc と 1〜9 は lhs にできない。settings.json に残る。:map だけで今の付け替えを出す。which-key は付け替えたあとのキーを出す。
例: :map <leader>x :echo 1 。:map <leader>* :quote \"* \"
書き方が違えば何もしない。一覧の本文は触らない。
詳しくは :help unmap :help mapleader :help whichkey",
    },
    Page {
        name: "unmap",
        group: "keymap",
        summary: "付け替えを消す",
        body: "\
打つ: :unmap lhs
変わるのはキー割り当て。その lhs を消す。
例: :unmap <leader>x で、その付け替えが無くなる。
無ければ何もしない。
詳しくは :help map",
    },
    Page {
        name: "mapleader",
        group: "keymap",
        summary: "リーダーキーを変える",
        body: "\
打つ: :mapleader ,
変わるのは <leader>。初期値は Space。settings.json に残る。
例: :mapleader , のあと、, がリーダー。
1文字で無いと何もしない。
詳しくは :help map :help whichkey",
    },
    Page {
        name: "scroll",
        group: "other",
        summary: "入力中も選択行が見えるようにする",
        body: "\
変わるのは一覧のスクロール。: の入力欄で一覧が短くなっても、選択行（V なら範囲の先頭）が見えるまでスクロールする。本文は変えない。
例: 末尾の行を選んで : を開いても、その行が見える。
詳しくは :help show :help colon",
    },
    Page {
        name: "blur",
        group: "other",
        summary: "一覧から離れたら隠す",
        body: "\
変わるのは一覧の表示。フォーカスが外れたら隠す。クリックで前面へ戻ったとき、別のアプリへ移ったときは隠す。編集、:、問い合わせ、ヘルプ、タグ入力のあいだは残る。E で外のエディタを開いているあいだは隠さない。ドラッグで動かしているあいだは隠さない。隠したあとは展開キーが前面に効く。Ubuntu も同じ。
例: 一覧を出したまま別のアプリをクリックすると、一覧は隠れる。
Esc で閉じるのは :help esc
詳しくは :help ctrl8 :help winmove",
    },
    Page {
        name: "whichkey",
        group: "other",
        summary: "待ちのキーの続きを出す",
        body: "\
打つ: g d y f <leader> のあと、400ms 待つ
変わるのはこの画面。続きのキーを出す。付け替えたあとのキーも出す。接頭辞以外は出さない。
例: g のあと、g. や gp が出る。
Esc / Ctrl+[ で待ちだけ解除。
詳しくは :help map :help esc",
    },
    Page {
        name: "drop",
        group: "other",
        summary: "ドロップしたパスを登録する",
        body: "\
操作: ファイルを一覧へドロップする
変わるのは一覧。パスを本文にして #path と #file を付けた行を先頭に作る。フォルダでもよい。
例: a.txt を落とすと、本文がそのパスで、#path と #file が付く。
常時監視はしない。開くのは :help open 。中身を貼るのは :help filetag
詳しくは :help register :help path",
    },
    Page {
        name: "cli",
        group: "other",
        summary: "ウィンドウを開かず式を実行する",
        body: "\
打つ: hataclip \"echo 3+4\"
変わるのは標準出力、または式の行き先。ウィンドウを開かない。標準入力がパイプならそれが流れの最初。行き先が無ければ標準出力。例: hataclip \"echo 3+4\" は 7。Windows のパイプは hataclip.exe（コンソール付き）。トレイは hataclip-gui.exe。Linux は hataclip 1本で両方。段だけの clip は標準入力をクリップボードへ、add は一覧へ。一覧はファイルが新しければ読み直す。開いているあいだも同じで、そのあと削除やタグ変更をしても足した行は残る。dir | hataclip.exe \"quote | clip\" は各行の頭に > を付けてクリップボードへ。式が無い、または2つ以上なら終了コード 1。hataclip quote は選択の代わりに標準入力を引用符付きで出す。壊れた段や空の入力は何も出さず終了コード 1。`--showerror` が付いたときだけ理由を標準エラーに出す。標準入力は旗が付いていても読む。式は1つ。旗だけ、または式が2つなら失敗し、トレイは開かない。`--help` は式ではない。引数なしは目次。`hataclip.exe --help quote` は `:help quote`。名前が無ければ `ない` と目次。終了コードは 0。収まらないときは `j` / `k`。`| show` も同じ。GUI が覚えた :help showerror は出さない。
詳しくは :help pipe :help sh",
    },
];

pub fn topics() -> Vec<String> {
    let mut names: Vec<String> = GROUPS.iter().map(|(id, _)| (*id).to_string()).collect();
    names.extend(PAGES.iter().map(|page| page.name.to_string()));
    names
}

pub fn render(topic: Option<&str>) -> String {
    let Some(name) = topic.map(str::trim).filter(|name| !name.is_empty()) else {
        return index();
    };
    let key = name
        .trim_start_matches(':')
        .trim_matches(|ch: char| ch == '{' || ch == '}' || ch.is_whitespace());
    if GROUPS.iter().any(|(id, _)| *id == key) {
        return group_page(key);
    }
    if let Some(page) = PAGES.iter().find(|page| page.name == key) {
        return format!(":{}\n\n{}", page.name, page.body);
    }
    format!("ない: {name}\n\n{}", index())
}

fn index() -> String {
    let mut out = String::from(
        "引数なしの :help は目次。各行の :help 名前 で、その操作だけを開く。j / k でスクロール。\n",
    );
    for (id, title) in GROUPS {
        out.push_str(&format!("\n{title}    :help {id}\n"));
        for page in PAGES.iter().filter(|page| page.group == *id) {
            out.push_str(&format!("{}    {}    :help {}\n", page.name, page.summary, page.name));
        }
    }
    out
}

fn group_page(id: &str) -> String {
    let title = GROUPS
        .iter()
        .find(|(group, _)| *group == id)
        .map(|(_, title)| *title)
        .unwrap_or(id);
    let mut out = format!(":{id}\n\n{title}。各行の :help 名前 を開く。\n");
    for page in PAGES.iter().filter(|page| page.group == id) {
        out.push_str(&format!("{}    {}    :help {}\n", page.name, page.summary, page.name));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_points_at_every_page() {
        let overview = render(None);
        assert!(overview.contains("引数なしの :help は目次"));
        assert!(overview.contains(":help keys"));
        assert!(overview.contains(":help pin"));
        assert!(overview.contains(":help search"));
        assert!(overview.contains(":help template"));
        assert!(overview.contains(":help sh"));
        assert!(overview.contains(":help filter"));
        assert!(!overview.contains("Escで絞り解除"));
        for page in PAGES {
            assert!(
                overview.contains(&format!(":help {}", page.name)),
                "missing {}",
                page.name
            );
        }
    }

    #[test]
    fn groups_are_lists_and_pages_have_examples() {
        let keys = render(Some("keys"));
        assert!(keys.contains(":help j"));
        assert!(keys.contains(":help G"));
        assert!(!keys.contains("打つ:"));
        assert!(render(Some("paste")).contains(":help quote"));
        assert!(render(Some("paste")).contains(":help enter"));
        assert!(render(Some("tag")).contains(":help run"));
        assert!(render(Some("tag")).contains(":help lock"));
        assert!(render(Some("tag")).contains(":help apptag"));
        assert!(render(Some("pin")).contains(":help gp"));
        assert!(render(Some("template")).contains(":help date"));
        assert!(render(Some("template")).contains(":help when"));
        assert!(render(Some("colon")).contains(":help sh"));
        assert!(render(Some("pipe")).contains(":help sel"));
        assert!(render(Some("j")).contains("3行目"));
        assert!(render(Some("dd")).contains("#lock"));
        assert!(render(Some("dd")).contains("残る"));
        assert!(render(Some("quote")).contains("* hello"));
        assert!(!render(Some("quote")).contains("ない:"));
    }

    #[test]
    fn operation_pages_keep_the_spec_examples() {
        assert!(render(Some("s")).contains("前面は hippo"));
        assert!(render(Some("s")).contains("履歴は触らない"));
        assert!(!render(Some("s")).contains("u で戻せる"));
        assert!(render(Some("s")).contains("Ctrl+8"));
        assert_eq!(render(Some("s")).matches("打つ:").count(), 1);
        assert!(render(Some("ctrl8")).contains(":quote | upper"));
        assert!(render(Some("ctrl8")).contains(":sh dir | quote"));
        assert!(render(Some("ctrl8")).contains(":sh \"xx | yy\" | quote"));
        assert!(render(Some("ctrl8")).contains("行頭までの選択に改行がある"));
        assert!(render(Some("sh")).contains("2行目以降を stdin"));
        assert!(render(Some("sh")).contains(":help run"));
        assert!(render(Some("sh")).contains(":sh \"xx | yy\""));
        assert!(!render(Some("sh")).contains("> clip"));
        assert!(render(Some("run")).contains("#run"));
        assert!(render(Some("lock")).contains("#lock"));
        assert!(render(Some("apptag")).contains("#app:chrome"));
        assert!(render(Some("slottag")).contains("#slot:3"));
        assert!(render(Some("confirm")).contains("#confirm"));
        assert!(render(Some("when")).contains("{{when app: chrome}}"));
        assert!(render(Some("when")).contains("{{when chrome}} は展開せずそのまま"));
        assert!(render(Some("pick")).contains("{{pick tag:env}}"));
        assert!(render(Some("pick")).contains("候補が 9 個までなら 1〜9"));
        assert!(render(Some("date")).contains("{{date+1w}}"));
        assert!(render(Some("{{date}}")).contains("{{date}}"));
        assert!(render(Some("var")).contains("{{var a}}"));
        assert!(render(Some("var")).contains("{{var: {{var: b}}}}"));
        assert!(render(Some("var")).contains(":set a+=1"));
        assert!(render(Some("tagbody")).contains("{{tag:work}}"));
        assert!(render(Some("wait")).contains("{{wait:200}}"));
        assert!(render(Some("sel")).contains("履歴の行にある sel"));
        assert!(render(Some("sel")).contains("クリップボードが変わらなければ空") || render(Some("sel")).contains("空なら何もしない"));
        assert!(render(Some("sel")).contains("{{sel|clip}}"));
        assert!(render(Some("J")).contains("V 中"));
        assert!(render(Some("E")).contains("メモ帳"));
        assert!(render(Some("join")).contains("1行ならその本文"));
        assert!(render(Some("join")).contains(":comma と :tab は何もしない"));
        assert!(render(Some("log")).contains("defaultLogFileName"));
        assert!(render(Some("echo")).contains(":echo 2+3 | quote"));
        assert!(render(Some("set")).contains(":set a="));
        assert!(render(Some("set")).contains("say \"hi\""));
        assert!(render(Some("n")).contains(":n 100"));
        assert!(render(Some("show")).contains("Ctrl+C は送らない"));
        assert!(render(Some("show")).contains("選択行（V なら範囲の先頭）が見えるまでスクロール"));
        assert!(render(Some("clip")).contains("同じ文字を次へ渡す"));
        assert!(render(Some("clip")).contains("末尾の > clip"));
        assert!(render(Some("add")).contains("流れが無い途中の add"));
        assert!(render(Some("add")).contains("sel | kebab"));
        assert!(render(Some("filter")).contains("a.txt"));
        assert!(render(Some("filter")).contains("not"));
        assert!(render(Some("slots")).contains(":sel | snake"));
        assert!(render(Some("slots")).contains("#slot:n") || render(Some("complete")).contains("#slot") || render(Some("slottag")).contains("#slot"));
        assert!(render(Some("complete")).contains("一覧の Enter と同じく展開して貼る"));
        assert!(render(Some("complete")).contains("#work はタグ work"));
        assert!(render(Some("complete")).contains("本文に無くても #alias:x"));
        assert!(render(Some("alias")).contains("x y でも当たる"));
        assert!(render(Some("slash")).contains("x y でも当たる"));
        assert!(render(Some("slash")).contains("Tab は一覧へ"));
        assert!(render(Some("esc")).contains("絞りを外す"));
        assert!(render(Some("gp")).contains("ピン"));
        assert!(render(Some("ga")).contains("前面"));
        assert!(render(Some("enter")).contains("次に一覧を出すとその行"));
        assert!(render(Some("dotsh")).contains(":.!sh"));
        assert!(render(Some("bang")).contains(":!!sh"));
        assert!(render(Some("cli")).contains("トレイは hataclip-gui.exe"));
        assert!(render(Some("cli")).contains("ファイルが新しければ読み直す"));
        assert!(render(Some("map")).contains("<leader>"));
        assert!(render(Some("home")).contains("shift+home"));
        assert!(render(Some("cut")).contains("送らない"));
        assert!(!render(Some("tag")).contains("#here"));
        assert!(!render(None).contains("{{cred}}"));
        assert!(!render(None).contains("#once"));
        assert!(!render(None).contains("paste_count"));
    }

    #[test]
    fn unknown_name_shows_the_same_index() {
        let missing = render(Some("nope"));
        assert!(missing.starts_with("ない"));
        assert!(missing.contains(":help j"));
        assert!(missing.contains(":help quote"));
        assert!(!missing.contains("トピック: keys"));
    }
}

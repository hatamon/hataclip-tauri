const TOPICS: &[(&str, &str)] = &[
    (
        "keys",
        "移動  j / k 矢印。端で止まる。gg 先頭、G 末尾。Ctrl+D / Ctrl+U 半ページ。f と 1 文字で先頭文字へ。; 次、, 前。a いまの貼り付け先だけ。Tab よく使うタグ切替（検索中は一覧へ）。/ 検索。Esc で絞りを外す。\n見る  ge 展開して全文。g? 回数・貼り付け先・タグ。\n編集  dd 削除（#lock は残す）。yy / Y ヤンク。p / P 置く。u / Ctrl+R 直前の 1 回を取り消し / やり直し。もう一度 u してもそれより前には戻らない。. 繰り返し。o 空行。e その場編集。E nvim（無ければメモ帳）。S 分割。J まとめ（V 中）。c 複製。t / T タグ。gp ピン。ga 前面の #app:。+ / - 1つ上/下（ピン同士とピン以外同士。塊はまたがない。#lock でも動く）。Ctrl+; / Ctrl+- は一覧の文字を1px大きく/小さく（9〜32。閉じても残る。; だけは検索の次）。\nその他  V 範囲。: コマンド。入力欄で一覧が短くなっても選択行（V なら範囲の先頭）が見えるまでスクロール。ドロップでパス登録。g / d / y / f / <leader> のあと 400ms で which-key。Esc / Ctrl+[ 閉じる。",
    ),
    (
        "paste",
        "Enter 貼り付けて閉じる。次に一覧を出すとその行。Ctrl+Enter 残す。1〜9 その行。Ctrl+1〜9 残す。:format 整形。:raw 本文のまま。:join 区切りつなぎ（未指定は , 。1行ならその本文。V ならその範囲。タブは :join \"\\t\"。:comma と :tab は何もしない）。:quote 行頭（未指定は > 。:quote \"* \" で箇条書き）。行末も :quote \"> \" \"<\" で hello が > hello<。\"\" はその端を付けない。:type 1 文字ずつ。:open は URL/パスを開く。g. 直前の貼り付け。Ctrl+C コピー。{{sel}} {{ask:}} {{pick list:}} {{pick tag:env}} {{pick search:}} {{app}} {{front}} {{focus}} {{when app: chrome}} {{env:}} {{var:a}} {{tag:work}} {{type:<Tab>}}。#confirm は貼る前に展開後を出す。#run があればそれで足りる。Ctrl+Shift+1〜9 は #slot:n があればそれ。無ければ並びの番号。前面アプリのコピー／貼り付けキーは settings.json の target_keys。:set copy ctrl+shift+c。:set paste shift+insert。Ctrl+8 は前面の {{date}} / :sh dir / :echo 2+3 を置き換える（一覧は出さない）。選択が2行以上で1行目が : のパイプなら、その式を2行目以降に実行して選択全体を置き換える。1行目は残さない。例: :s/old/new の次が abc def old ghi と oldabc ddd なら abc def new ghi と newabc ddd。:quote | upper の次が xxx と yyy なら > XXX と > YYY。: で始まらない、または段として読めない1行目は今までの展開。1行で sh か echo が自分で結果を作るパイプも置き換える。:sh dir | quote は dir の各行の頭に > 。:echo 2+3 | quote \">\" \"<\" は >5<。シェルの | は :sh \"xx | yy\" | quote のように括る。括らない | は段なので、段が無ければ何もしない（:sh dir | sort は何もしない）。:quote や :sel | upper だけの1行は何もしない。複数行で1行目が :echo だけなら何もしない。入力が要るのに2行目が無い、段が不正、選択が空、sh が失敗、展開結果が空、クリップボードに置けないなら何もしない。選択は残す。履歴は変えない。. と sel は2行目以降。流れがあるので1行目の :sh はそれを stdin に受け取る。| clip | show | add | set | open は今のパイプと同じ。Ubuntu はクリップボードを読み、結果はクリップボードに置く。Ctrl+9 は前面の選択語で履歴を補完し、一覧の Enter と同じく展開して貼る。{{date}} は日付。{{n}} は貼れたときだけ進む。履歴の本文は変えない。展開が空なら何もしない。{{ask:}} か {{pick:}} がある行、#run #confirm #grab の行、本文がパイプの行は何もしない。当たりは一覧の / と同じ。#work はタグ work の行を一覧の順。#work hello はタグと本文。#work #home は両方。本文に無くても #alias:x と #alias:y なら x y で当たる。順は問わない。# だけと #secret は何もしない。#run #confirm #grab、{{ask:}} {{pick:}}、本文がパイプの行は出さない。複数ならその順の最初。2秒以内の連打は、次が展開して空でなければ Ctrl+Z のあと次の候補。次が無い、空、対象外なら Ctrl+Z しない。Undo が戻らなければ何もしない。空や当たり無しも何もしない。Ctrl+A は送らない。Ubuntu は展開した文字をクリップボードへ置く。本文がパイプの行は Enter / Ctrl+Enter / 1〜9 / Ctrl+1〜9 / Ctrl+Shift+1〜9 でそのパイプを実行する。例: #slot:1 の本文が :sel | snake。メモ帳で getUserName を選んで Ctrl+Shift+1 すると一覧は出ずに get_user_name に置き換わる。:sel | quote で行頭に > を付けて | clip ならクリップボードへ置き、前面には貼らない。段の名前が不正、または sel が空なら何もしない。{{ }} の中だけの | はパイプにしない。",
    ),
    (
        "tag",
        "t で付ける。T で外す。V 中は範囲の全行。/ のあと #tag で絞る。Tab / Shift+Tab でよく使うタグ切替。編集中はチップで付け外し。\n\
自動  Ctrl+4 のとき http(s) なら #url、パスなら #path。ほかは自分で付ける。同じ本文は先頭へ移すだけで、2件目は作らない。\n\
\n\
#pin相当 gp で付ける。先頭に固定。+ / - でピン同士の順\n\
#secret  一覧を ••••。貼り付けは普通\n\
#alias:foo  /foo でも当たる。#alias:x と #alias:y なら x y でも当たる。順は問わない。\n\
#app:chrome  前面のアプリ名が chrome のときだけ出す。#app:code と複数ならどれか。ブラウザはページが違っても当たる\n\
#not:chrome  そのアプリのときは出さない。#app: と両方なら、アプリに当たって #not に当たらないときだけ\n\
#run     貼るときコマンドを実行。自動では付けない。詳しくは :help sh\n\
#confirm 貼る前に展開後を出して Enter。#run があればそれで足りる\n\
#file    本文のパスのファイル内容を貼る。#run が先。ドロップすると #path と一緒に付く\n\
#lock    dd / :clear / J で消えない。外すのは T\n\
#slot:3  Ctrl+Shift+3 でその行。複数なら一覧の先。無ければ今までの 3 番目\n\
#grab    1行目が型、2行目が出力。穴は <名前>。前面の選択が型に当たれば2行目を埋めて貼る。当たらなければ何もしない。値は :set に残さない。Ubuntu はクリップボードを照合する\n\
#url #path  登録時の自動タグ。動きは目印だけ",
    ),
    (
        "pin",
        "gp でピン留め。ga で選んだ行の #app: をいまの前面アプリにする。ほかの #app: は外す。+ で1つ上、- で1つ下。ピン同士と、ピンでない行同士。塊はまたがない。#lock でも動く。ピンの並びは手動順。ピンの下も動かした順が残る。",
    ),
    (
        "visual",
        "V で選択開始。j / k で範囲。Enter 改行つなぎ。:join 区切りつなぎ。:quote 行頭（:quote \"* \" で箇条書き）。J 1 行にまとめる（#lock があるとまとめない）。c 複製。S 分割。:sort 本文の順。:!!sh 各行をフィルタ。dd / yy / t / T / gp / ga / + / - は範囲に効く。dd は #lock を残す。Esc で解除。",
    ),
    (
        "search",
        "/ で検索。照合は本文の先頭 4KB まで。それより後ろの語は当たらない。Tab は一覧へ（絞りと検索欄は残る）。Esc / Ctrl+[ で絞りを外す。# の候補がある Tab はタグ補完。#tag でタグ。a でいまの貼り付け先だけ。f と 1 文字で先頭文字へ飛ぶ。; 次、, 前。Ctrl+N / Ctrl+P で移動。#alias:foo は foo でも当たる。#alias:x と #alias:y なら x y でも当たる。順は問わない。",
    ),
    (
        "edit",
        "e その場編集（Ctrl+Enter 保存、Esc 取り消し）。E nvim（無ければ $EDITOR、それも無ければメモ帳）。o 空行を作って編集。S 改行で分割。J は V 中なら選んだ行を改行で 1 行に（タグは和集合、ピンはどれかにあれば残す）。c すぐ下に複製。:!!sh は各行をコマンドで書き換える。:sort は V 中なら本文の順。. は dd p P t T gp ga S J c :!! :sort g~ g: + - を繰り返す。g: は行に残した式をもう一度実行する。式が無ければ何もしない。sel はいまの前面の選択、. はいまの本文。失敗したら本文はそのまま。:from はその式だけを編集する（e は本文）。| add と :!!sh でできた行に式が残る。g~ は #grab の1行目と2行目を入れ替える。2行目が無ければ何もしない。3行目以降はそのまま。u で戻せる。",
    ),
    (
        "sh",
        "#run を自分で付けた行だけコマンドを実行する。{{sh: コマンド}} はその場の標準出力に置き換わる。#run 付きで {{sh:}} が無ければ本文全体がコマンド。貼る前に出力を出して Enter で貼る。Esc は中止。:sh dir はその場実行して貼る（一覧で打つ最初の :sh は stdin なし。Ctrl+8 の1行で :sh dir | quote のように後ろに段があれば stdin なしで実行して選択を置き換える。シェルの | は :sh \"xx | yy\" のように括る。括らない段が読めなければ何もしない。複数行で1行目なら、2行目以降を stdin に受け取る）。:.!sh xxx はカレント行（V なら改行つなぎ）を stdin に流す。:!!sh xxx は各行を stdin に流して本文を書き換える（貼らない）。式が残るので g: でもう一度。失敗したら本文はそのまま。:@ は直前の :sh / :.!sh のスクリプトをもう一度（stdin はいまの選択）。失敗したら貼らない／書き換えない。| clip でクリップボードへ（貼らない）。| で左からつなぐ。次の :sh は左の結果を stdin に受け取る。",
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
{{sel}}            前面の選択。Ctrl+C の 200ms 後。クリップボードが変わらなければ空。{{sel|clip}} は空なら隣。隣がトークンなら展開し、違う文字ならそのまま。{{front|無題}} {{sel|clip|なし}}\n\
{{ask:名前}}       貼る前に入力。同じ名前は 1 回\n\
{{pick list: a, b}}  貼る前に候補から選ぶ。j / k と Enter。候補が 9 個までなら 1〜9 でその番。10 個以上は j / k。Esc は中止。同じ候補は 1 回。{{ask:}} があるときは先に全部聞く。{{pick: a, b}} は空。{{pick tag:env}} はタグ env の本文。{{pick search: \"xx\"}} は / と同じ当たりの行。自分自身は入れない。無ければ聞かない\n\
{{app}}            前面アプリのプロセス名。無ければ空\n\
{{front}}          前面ウィンドウのタイトル。無ければ空\n\
{{focus}}          フォーカスしている入力欄の種類。Edit Document ComboBox。取れなければ空。Ubuntu は空。常時監視はしない\n\
{{when app: chrome}}  前面が chrome のときだけその区間。複数は {{when app: chrome, msedge}}。{{when var: a: \"AAA\"}} は :set のそのままの文字列。{{when focus: Edit}} は入力欄の種類。{{when}} はどれにも当たらないとき。{{when chrome}} は展開せずそのまま。Ubuntu では app と focus は空\n\
{{n}} / {{n:2}}    連番。Ctrl+Enter で増える。:n 100 で初期値。閉じると初期値に戻る。ge やコピーでは進まない\n\
{{uuid}}           UUID v4\n\
{{user}}           ログイン名\n\
{{host}}           コンピュータ名\n\
{{env:USERPROFILE}}  同名の環境変数。無ければ空\n\
{{var:a}}          :set a= の値。中の {{date}} も展開する。値が :sh dir ならそのとき実行。:set では実行しない。名前に {{var:b}} を書ける（{{var: {{var: b}}}}）。:set a+=1 は貼って成功したあと 1 増やす。無い変数は 1。数字以外には付かない\n\
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
        ":help [topic] 使い方。:export / :import <path> Markdown。:clear / :dedup は yes で確認。:clear はピンと #lock 以外。:quote 行頭（未指定は > 。:quote \"* \" で箇条書き）。行末も :quote \"> \" \"<\" で hello が > hello<。\"\" はその端を付けない。:type 1 文字ずつ。:format 整形。:raw 本文のまま。:join 区切りつなぎ（未指定は , 。1行ならその本文。V ならその範囲。タブは :join \"\\t\"。:comma と :tab は何もしない）。:open URL/パス。:log は選択行をファイルへ追記して前面には貼らない。未指定は変数 defaultLogFileName。未定義か空なら何もしない。:log path でそのファイル。V は改行つなぎ。既存ファイルの改行に合わせる。無いファイルは CRLF。書けなければ何もしない。| log は同じ（:echo 3+4 | log）。未定義なら貼らない。Ctrl+8 で1行目が log なら2行目以降をそのファイルへ足す。選択は置き換えない。:echo 2+3 四則。* / が先。() で変えられる。変数も使える。:s/old/new は選択行の old を全部 new にして前面へ貼る。履歴は触らない。正規表現は使わない。old が空なら何もしない。結果が空でも選択は空。V は改行つなぎ。段の :sel | s/old/new は前面の選択。new は2つ目の / の後ろ全部。old が無くても残りを貼る。:sort は V 中なら本文の順。:sh 実行して貼る。:.!sh はカレント行を stdin に。:!!sh は本文を書き換える。:@ 直前の :sh。| で左からつなぐ（引用符の中の | では切らない）。| clip はクリップボード（前面には貼らない。末尾の > clip は不正で何もしない）。| add は一覧へ1件。できた行には add より前の式が残る（:sel | kebab | add なら sel | kebab）。途中の add は同じ文字を次へ渡す。空や流れが無い途中の add は足さない。g: でもう一度実行する。sel はいまの前面の選択、. はいまの本文。失敗したら本文はそのまま。:from で式だけ編集する。| set a は変数へ。| open は URL かパスなら開く。| show はここに出す。一覧で打つ最初の :sh は stdin なし。次の :sh は左を stdin に。Ctrl+8 の複数行では1行目の :sh が2行目以降を stdin に受け取る。1行で sh か echo が自分で結果を作るなら、後ろの段も stdin なしで実行して選択を置き換える。例: :sh dir | quote。シェルの | は引用符の中（:sh \"xx | yy\" | quote）。括らない段が読めなければ何もしない。:quote や :sel | upper だけの1行は何もしない。`.` と sel は、一覧では一覧の行と前面の選択、複数行ではどちらも2行目以降。:. は選択行。先頭の clip はクリップボードを読む。sel は前面の選択。Windows では Ctrl+C の 200ms 後を読んで、すぐクリップボードを戻す。空なら何もしない。Ubuntu では履歴の行にある sel は実行しない。Ctrl+8 の複数行では選択はもう読んであるので sel があっても実行し、結果はクリップボードに置く。流れがあるあとの clip は、その文字をクリップボードへ書いて同じ文字を次へ渡す。空ならクリップボードは変えない。例: :sel | upper は前面の hello を HELLO にして貼る。入力中は結果を出さない。Ctrl+Enter で結果の先頭12行をここに出す。式を書き換えたら消える。sel を含む式は出さない。: では一覧を隠さず Ctrl+C は送らない。入力欄とプレビューで一覧が短くなっても、選択行（V なら範囲の先頭）が見えるまでスクロールする。Enter で sel が要るときはその場で読む。sh を含む、段が不正、json が読めない、入力が空のときは出さない。12行を超えたら末尾に ...。Enter の実行は今どおり。前面が helloWorld のとき :sel | kebab は hello-world。:. | upper は一覧の行を大文字にして貼る。:clip | show はクリップボードをここに出す。camel pascal snake kebab upper lower は文字の形を変える。json xml は整形する。読めなければ何もしない。split は区切り必須（タブは split \"\\t\"）。分けた列はタブでつなぐ。col は 1 始まり。足りない行は捨て、1 未満や整数でないときは何もしない。get は JSON Pointer。読めない、または無いときは何もしない。例: :sel | split , | col 2 は a,b,c の b。:clip | json | get /items/0/name | show は hatamon。diff と only の引数は . か clip。:clip | diff . | show はクリップボードにあって選択に無い行を - 、選択にあってクリップボードに無い行を + 。共通行は出さない。:. | only clip | show は選択にあってクリップボードに無い行。filter は含む行だけ残す。:sel | filter \".txt\"。:filter not \".txt\" は含まない行。正規表現は使わない。大文字と小文字は区別する。1行も残らなければ何もしない。空の引数も何もしない。選択は空にしない。全部同じ、または only が 0 行なら何もしない。put は JSON Pointer の位置へ書く。引数はポインタ、空白、値。値は引用できる。JSON として読めたらその値、読めなければ文字列。流れが JSON でない、またはポインタが無いときは何もしない。例: :sel | put /n 2 | json は n が 2 の JSON を整形して貼る。each はこれより後ろの段を行ごとに実行し、成功した行だけを改行でつなぐ。後ろに sh があるとパイプ全体を何もしない。例: :sel | each | get /id は各行の id。echo は式の結果で流れを置き換える。左は見ない。計算できなければ何もしない。段が1つでも同じ。:upper は選択行の hello を HELLO にして貼る。:json は読めるときだけ整形して貼る。:map lhs rhs 付け替え（:map <leader>* :quote \"* \"）。:unmap。:map だけで一覧。:mapleader でリーダー（初期値 Space）。:settings は settings.json をエディタで開く。:set copy ctrl+shift+c はいまの前面アプリのコピー。:set paste shift+insert は貼り付け。:set a=\"{{date}}\" は変数。引用の中の \\\" は \"、\\t はタブ、\\n は改行。:set a=\"say \\\"hi\\\"\" の値は say \"hi\"。{{var:a}} と {{var a}} で貼るとき展開。:set だけで一覧。:set a= で消す。:set a+=1 は貼って成功したあと 1 増やす。無い変数は 1。数字以外には付かない。:n 100 は {{n}} の初期値。settings.json に残る。:n だけでいまの値。:n 1 で 1 から。:tags はタグと件数。Tab でコマンド補完。↑↓ で入力履歴。入力中は結果を出さない。Ctrl+Enter で結果の先頭12行をここに出す。式を書き換えたら消える。sel を含む式は出さない。: では一覧を隠さず Ctrl+C は送らない。入力欄とプレビューで一覧が短くなっても、選択行（V なら範囲の先頭）が見えるまでスクロールする。Enter で sel が要るときはその場で読む。sh を含む、段が不正、json が読めない、入力が空のときは出さない。12行を超えたら末尾に ...。Enter の実行は今どおり。例: 前面が helloWorld のとき :sel | kebab は hello-world。hataclip に式を1つ渡すとウィンドウを開かず実行する。標準入力がパイプならそれが流れの最初。行き先が無ければ標準出力。例: hataclip \"echo 3+4\" は 7。Windows のパイプは hataclip.exe（コンソール付き。毎回打つので短い名前）。トレイは hataclip-gui.exe。Linux は hataclip 1本で両方。段だけの clip は標準入力をクリップボードへ、add は一覧へ。一覧はファイルが新しければ読み直す。開いているあいだも同じで、そのあと削除やタグ変更をしても足した行は残る。dir | hataclip.exe \"quote | clip\" は各行の頭に > を付けてクリップボードへ。hataclip.exe \"echo 3+4\" は 7。式が無い、または2つ以上なら終了コード 1。hataclip quote は選択の代わりに標準入力を引用符付きで出す。壊れた段や空の入力は何も出さず終了コード 1。",
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
:      help  sh  !!  @  echo  export  import  quote  filter  type  format  raw  join  open  log  s/  sort  clear  dedup  map  unmap  mapleader  set  n  settings  tags\n\
\n\
詳しくは :help keys  :help paste  :help tag  :help template  :help edit  :help colon  :help map  のように。j / k でスクロール。\
";

pub fn topics() -> Vec<String> {
    TOPICS.iter().map(|(name, _)| (*name).to_string()).collect()
}

const ENTRIES: &[(&str, &str)] = &[
    ("quote", "打つ: :quote または :quote \"* \" または :quote \"> \" \"<\"。変わるのは前面へ貼る文字。履歴は触らない。未指定の行頭は > 。すでにその行頭（と行末）なら付けない。\"\" はその端を付けない。失敗はしない。例: 選択行 hello に :quote \"* \" で前面は * hello。"),
    ("join", "打つ: :join または :join \" | \"。変わるのは前面へ貼る文字。履歴は触らない。1行ならその本文を貼る。V ならその範囲を区切りでつなぐ。未指定は , 。タブは :join \"\\t\"。空の選択は何もしない。例: hello と world を V して :join \" | \" で前面は hello | world。"),
    ("sel", "打つ: :sel | upper のようにパイプの段。変わるのはそのパイプの結果（行き先が無ければ前面）。履歴は触らない。一覧では sel は前面の選択、. は一覧の行。Ctrl+8 の複数行ではどちらも2行目以降。Windows は Ctrl+C の 200ms 後を読んで、すぐクリップボードを戻す。空なら何もしない。Ubuntu では履歴の行にある sel は実行しない。Ctrl+8 の複数行では選択はもう読んであるので sel があっても実行し、結果はクリップボードに置く。例: 前面の hello に :sel | upper で前面は HELLO。"),
    ("clip", "打つ: :clip | show のように先頭の段、行き先の | clip、または途中の clip。先頭で流れが無ければクリップボードを読む。流れがある途中の clip は、その文字をクリップボードへ書いて同じ文字を次へ渡す。空ならクリップボードは変えない。| clip は結果をクリップボードへ書き、前面には貼らない。末尾の > clip も何もしない。例: :sel | clip | upper は選択をクリップボードへ置き、大文字を前面へ貼る。:clip | upper | clip はクリップボードの hello を HELLO にして戻す。"),
    ("sh", "打つ: :sh dir。変わるのは前面（標準出力を貼る）。履歴は触らない。一覧で打つ最初の :sh は stdin なし。Ctrl+8 の1行で後ろに段があれば stdin なしで実行して選択を置き換える。シェルの | は引用符の中。括らない段が読めなければ何もしない。複数行で1行目なら、2行目以降を stdin に受け取る。失敗したら貼らない。:.!sh は選択行を stdin に。:!!sh は本文を書き換え、式が残る。Ubuntu でもシェルは動く。例: :sh echo hi で前面は hi。"),
    ("echo", "打つ: :echo 2+3。変わるのは前面（計算結果を貼る）。履歴は触らない。* と / が先。() と :set の変数が使える。計算できなければ何もしない。1行なら 5。1行の :echo 2+3 | quote は計算してから段を掛ける。複数行で1行目が :echo だけなら何もしない（下の行は残る）。例: :echo 3+4 で前面は 7。:echo 2+3 の次が notes なら置き換えない。"),
    ("format", "打つ: :format。変わるのは前面。履歴は触らない。選択行を貼り付け向けに整える。空なら何もしない。例: 選択が {\"a\":1} なら整形して前面へ貼る。"),
    ("raw", "打つ: :raw。変わるのは前面。履歴は触らない。{{date}} などは展開せず本文のまま貼る。例: 本文 {{date}} は前面も {{date}}。"),
    ("type", "打つ: :type。変わるのは前面（1文字ずつ送る）。履歴は触らない。Ubuntu はキーを送らないので何もしない。例: 選択 hello を :type で前面へ1文字ずつ入る。"),
    ("open", "打つ: :open。変わるのは別アプリ（URL かパスを開く）。一覧とクリップボードは触らない。URL でもパスでもなければ何もしない。例: https://example.com を :open でブラウザが開く。"),
    ("log", "打つ: :log または :log notes.log。変わるのはそのファイル（選択行を末尾へ追記）。前面には貼らない。履歴は触らない。未指定の出力先は変数 defaultLogFileName。未定義か空、選択が空、展開できない、書けない、なら何もしない。:log path はそのファイル。V は改行つなぎ。既存ファイルの改行に合わせる。無いファイルは CRLF。| log も同じファイル。:echo 3+4 | log は 7 を足して前面には貼らない。Ctrl+8 で1行目が log、次が abc と def ならその2行を足す。選択は置き換えない。例: :set defaultLogFileName=notes.log のあと hello を :log で notes.log に hello。"),
    ("camel", "打つ: :camel または :sel | camel。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: foo_bar は fooBar。"),
    ("pascal", "打つ: :pascal または :sel | pascal。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: foo_bar は FooBar。"),
    ("snake", "打つ: :snake または :sel | snake。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: getUserName は get_user_name。"),
    ("kebab", "打つ: :kebab または :sel | kebab。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: helloWorld は hello-world。"),
    ("upper", "打つ: :upper または :sel | upper。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: hello は HELLO。"),
    ("lower", "打つ: :lower または :sel | lower。変わるのは前面。履歴は触らない。入力が空なら何もしない。例: AbC は abc。"),
    ("json", "打つ: :json。変わるのは前面。履歴は触らない。JSON を整形する。読めなければ何もしない。例: {\"a\":1} は改行付きになる。"),
    ("xml", "打つ: :xml。変わるのは前面。履歴は触らない。XML を整形する。読めなければ何もしない。"),
    ("split", "打つ: :sel | split , 。区切りは必須。タブは split \"\\t\"。変わるのは次の段へ渡す文字（列はタブ1つでつなぐ）。区切りが無ければ何もしない。例: a,b,c は a\\tb\\tc。"),
    ("col", "打つ: col 2。1始まり。変わるのはその列だけ。短い行は落ちる。1未満や整数でないなら何もしない。例: a,b,c を split , | col 2 で b。"),
    ("get", "打つ: get /name。変わるのはその値。文字列はそのまま、数と真偽と null は文字、オブジェクトと配列は JSON。読めないか無いなら何もしない。例: {\"name\":\"hata\"} の get /name は hata。"),
    ("diff", "打つ: :sel | diff clip または diff . 。変わるのは前面（- の行のあと + の行）。同じ、または only 側が空なら何もしない。履歴は触らない。例: 選択 a\\nb とクリップボード a\\nc で - b と + c。"),
    ("only", "打つ: :sel | only clip。変わるのは前面（左にだけある行）。右にもある行は出さない。全部同じなら何もしない。例: a\\nb と a なら b。"),
    ("filter", "打つ: :filter \".txt\" または :filter not \".txt\"。変わるのは前面（含む行だけ、not は含まない行）。履歴は触らない。正規表現は使わない。大文字と小文字は区別する。括らなければ空白の後ろ全部。\\\" は \"、\\t はタブ、\\n は改行。針が not なら :filter \"not\"。1行も残らない、引数が空、流れが空なら何もしない。選択は空にしない。例: a.txt と b.rs に :filter \".txt\" で前面は a.txt。:sh dir | filter \".txt\" も同じ。Ctrl+8 の1行は sh か echo が先にあるときだけ。:filter だけの1行は何もしない。"),
    ("put", "打つ: put /n 2。変わるのはその JSON。ポインタが既に無いと何もしない。値が JSON ならその値、そうでなければ文字列。例: {\"name\":\"x\"} に put /n 2 で n が 2。"),
    ("each", "打つ: :sel | each | upper。後ろの段を行ごとに実行する。失敗した行は落ちる。全部失敗なら何もしない。後ろに sh があると不正で何もしない。例: a\\nb を each | upper で A\\nB。"),
    ("add", "打つ: :sel | kebab | add。変わるのは一覧（結果を1件足す）。前面とクリップボードは触らない。式が行に残る。空なら足さない。途中の add は同じ文字を次へ渡す。流れが無い途中の add は何もしない。例: 前面 userName で本文 user-name、式 sel | kebab。:sel | kebab | add | quote は一覧に user-name、前面は > user-name。"),
    ("show", "打つ: :echo 3+4 | show。変わるのはこのヘルプ画面。前面には貼らない。空なら出さない。例: :echo 3+4 | show で 7。"),
    ("set", "打つ: :set a=hello または | set a。:set は変数を残す。貼らない。| set a はパイプの結果をその変数へ。名前が不正なら何もしない。引用は \\\" \\t \\n を読む。例: :set a=\"say \\\"hi\\\"\" で値は say \"hi\"。"),
    ("from", "打つ: :from。変わるのはその行の式だけ。本文は e で編集する。引数が無ければ編集画面。引数があればその文字が式になる。式の無い行でも書ける。例: :from sel | upper。"),
    ("s", "打つ: :s/old/new。変わるのは前面へ貼る文字。履歴は触らない。選択行の old を全部 new にする。正規表現は使わない。old が空なら何もしない。old が無くても残りを貼る。結果が空でも選択は空になる。V は改行つなぎ。u では戻らない。本文を直すのは e。例: 選択行 hello に :s/ell/ipp で前面は hippo。段 :sel | s/old/new は前面の選択を置換して貼る。Ctrl+8 で1行目が :s/old/new なら2行目以降を置換して選択全体を置き換える。1行だけは何もしない。"),
    ("sort", "打つ: :sort。変わるのは V した行の並び（本文の順）。1行だけなら何もしない。u で戻せる。例: b と a を V して :sort で a が先。"),
    ("clear", "打つ: :clear のあと :clear yes。変わるのは一覧。ピンと #lock 以外を消す。yes が無いと確認の文字を出すだけ。"),
    ("dedup", "打つ: :dedup のあと :dedup yes。変わるのは一覧。同じ本文は1件にまとめる。yes が無いと確認だけ。"),
    ("export", "打つ: :export path。変わるのはそのファイル。一覧は残る。書けなければ何もしない。例: :export C:\\tmp\\hataclip.md。"),
    ("import", "打つ: :import path。変わるのは一覧（Markdown から足す）。読めなければ何もしない。"),
    ("map", "打つ: :map lhs rhs。変わるのはキー割り当て。一覧の本文は触らない。書き方が違えば何もしない。例: :map <leader>x :echo 1。"),
    ("unmap", "打つ: :unmap lhs。その割り当てを消す。無ければ何もしない。"),
    ("mapleader", "打つ: :mapleader ,。リーダーキーを変える。1文字で無いと何もしない。"),
    ("n", "打つ: :n 100。変わるのは {{n}} の初期値。貼ったあと Ctrl+Enter で増える。閉じると初期値に戻る。"),
    ("settings", "打つ: :settings。settings.json を開く。エディタが無ければ何もしない。"),
    ("tags", "打つ: :tags。この画面にタグ一覧を出す。一覧自体は変わらない。"),
    ("help", "打つ: :help または :help quote。この画面に使い方を出す。名前が無ければない、と出す。"),
    ("date", "{{date}} は貼る直前に今日へ変わる。履歴は変わらない。{{date:%Y%m%d}} は chrono。{{date-1d}} は昨日。失敗はしない。Ubuntu でも動く。例: {{date:%Y}} は 2026。"),
    ("time", "{{time}} は貼る直前にいまの時刻へ変わる。履歴は変わらない。例: 10:54。"),
    ("ask", "{{ask:名前}} は貼る前に入力を聞く。同じ名前は1回。Esc は中止で貼らない。例: {{ask:name}} に hata と入れるとその文字になる。"),
    ("pick", "{{pick list: prod, stg}} は貼る前に候補を聞く。{{pick tag:env}} はタグ。{{pick search: \"xx\"}} は / と同じ。昔の {{pick: a, b}} は空で聞かない。無ければ聞かない。"),
    ("app", "{{app}} は前面アプリのプロセス名。取れなければ空。履歴は変わらない。Ubuntu は空。"),
    ("front", "{{front}} は前面ウィンドウのタイトル。取れなければ空。Ubuntu は空。"),
    ("focus", "{{focus}} は貼るときに入力欄の種類（Edit など）。常時監視はしない。取れなければ空。Ubuntu は空。"),
    ("when", "{{when app: chrome}} はそのアプリのときだけ区間を残す。{{when var: a: \"AAA\"}} は変数。{{when focus: Edit}} は入力欄。{{when}} はどれにも当たらないとき。{{when chrome}} はそのまま残る。Ubuntu では app と focus は空なので、その枝は残らない。"),
    ("uuid", "{{uuid}} は貼る直前に UUID v4。履歴は変わらない。"),
    ("user", "{{user}} はログイン名。取れなければ空。"),
    ("host", "{{host}} はコンピュータ名。取れなければ空。"),
    ("env", "{{env:USERPROFILE}} はその環境変数。無ければ空。履歴は変わらない。"),
    ("var", "{{var:a}} は :set a= の値。中の {{date}} も展開する。値が :sh ならそのとき実行。無い変数は空。"),
    ("wait", "{{wait:200}} は貼る途中で 200ms 待つ。最大 5 秒。それより大きいと 5 秒。"),
];

pub fn render(topic: Option<&str>) -> String {
    let Some(name) = topic.map(str::trim).filter(|name| !name.is_empty()) else {
        return format!(
            "{OVERVIEW}\n\nトピック: {}",
            topics().join(" ")
        );
    };
    let key = name
        .trim_start_matches(':')
        .trim_matches(|ch: char| ch == '{' || ch == '}' || ch.is_whitespace());
    if let Some((_, body)) = TOPICS.iter().find(|(topic, _)| *topic == key) {
        return format!(":{key}\n\n{body}");
    }
    if let Some((_, body)) = ENTRIES.iter().find(|(topic, _)| *topic == key) {
        return format!(":{key}\n\n{body}");
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
        assert!(render(Some("tag")).contains("#lock"));
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
        assert!(render(Some("paste")).contains(":s/old/new"));
        assert!(render(Some("paste")).contains(":quote | upper"));
        assert!(render(Some("s")).contains("前面は hippo"));
        assert!(render(Some("s")).contains("履歴は触らない"));
        assert!(!render(Some("s")).contains("u で戻せる"));
        assert!(!render(Some("edit")).contains(":s :!!"));
        assert!(render(Some("s")).contains("Ctrl+8"));
        assert_eq!(render(Some("s")).matches("打つ:").count(), 1);
        assert!(render(Some("colon")).contains("段の :sel | s/old/new"));
        assert!(render(Some("colon")).contains("Ctrl+C は送らない"));
        assert!(render(Some("colon")).contains("トレイは hataclip-gui.exe"));
        assert!(render(Some("colon")).contains("ファイルが新しければ読み直す"));
        assert!(render(Some("colon")).contains("選択行（V なら範囲の先頭）が見えるまでスクロール"));
        assert!(render(Some("keys")).contains("選択行（V なら範囲の先頭）が見えるまでスクロール"));
        assert!(render(Some("paste")).contains("#slot:n"));
        assert!(render(Some("paste")).contains("#confirm"));
        assert!(render(Some("template")).contains("{{var: {{var: b}}}}"));
        assert!(!render(Some("tag")).contains("#here"));
        assert!(render(Some("sh")).contains("2行目以降を stdin"));
        assert!(render(Some("sel")).contains("履歴の行にある sel"));
        assert!(render(Some("sh")).contains("#run"));
        assert!(render(Some("edit")).contains("J は V 中"));
        assert!(render(Some("template")).contains("{{var a}}"));
        assert!(render(Some("template")).contains("{{sh: コマンド}}"));
        assert!(render(Some("template")).contains("{{tag:work}}"));
        assert!(render(Some("template")).contains("{{wait:200}}"));
        assert!(render(Some("template")).contains("{{date-1d}}"));
        assert!(render(Some("template")).contains("{{sel|clip}}"));
        assert!(render(Some("template")).contains("クリップボードが変わらなければ空"));
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
        assert!(render(Some("paste")).contains("Ctrl+8 は前面の"));
        assert!(render(Some("paste")).contains("一覧の Enter と同じく展開して貼る"));
        assert!(render(Some("paste")).contains("#work はタグ work"));
        assert!(render(Some("paste")).contains("本文に無くても #alias:x"));
        assert!(render(Some("paste")).contains(":sh dir | quote"));
        assert!(render(Some("paste")).contains(":sh \"xx | yy\" | quote"));
        assert!(render(Some("sh")).contains(":sh \"xx | yy\""));
        assert!(render(Some("echo")).contains(":echo 2+3 | quote"));
        assert!(render(Some("colon")).contains(":sh \"xx | yy\" | quote"));
        assert!(render(Some("filter")).contains("a.txt"));
        assert!(render(Some("filter")).contains("not"));
        assert!(render(Some("colon")).contains(":sel | filter \".txt\""));
        assert!(render(None).contains("filter"));
        assert!(render(Some("sh")).contains(":!!sh"));
        assert!(render(Some("colon")).contains(":!!sh"));
        assert!(render(Some("edit")).contains(":!!sh"));
        assert!(!render(Some("sh")).contains("> clip"));
        assert!(render(Some("colon")).contains(":.!sh"));
        assert!(render(Some("colon")).contains("末尾の > clip は不正"));
        assert!(render(Some("clip")).contains("同じ文字を次へ渡す"));
        assert!(render(Some("add")).contains("流れが無い途中の add"));
        assert!(render(Some("colon")).contains("同じ文字を次へ渡す"));
        assert!(render(Some("colon")).contains(":quote \"* \""));
        assert!(render(Some("colon")).contains("> hello<"));
        assert!(render(Some("paste")).contains(":quote \"* \""));
        assert!(render(Some("paste")).contains(":join"));
        assert!(render(Some("colon")).contains(":join"));
        assert!(render(Some("colon")).contains(":comma と :tab は何もしない"));
        assert!(render(Some("paste")).contains(":open"));
        assert!(render(Some("open")).contains(":open"));
        assert!(render(Some("log")).contains("defaultLogFileName"));
        assert!(render(Some("paste")).contains(":sh dir"));
        assert!(render(Some("colon")).contains(":echo"));
        assert!(render(Some("colon")).contains("sel は前面の選択"));
        assert!(render(Some("colon")).contains(":sel | upper"));
        assert!(render(Some("colon")).contains("段が1つでも同じ"));
        assert!(render(Some("colon")).contains(":type"));
        assert!(render(Some("colon")).contains(":settings"));
        assert!(render(Some("colon")).contains(":set copy"));
        assert!(render(Some("colon")).contains(":set paste"));
        assert!(render(Some("quote")).contains("* hello"));
        assert!(!render(Some("quote")).contains("ない:"));
        assert!(render(Some("join")).contains("1行ならその本文"));
        assert!(render(Some("{{date}}")).contains("{{date}}"));
        assert!(render(Some("when")).contains("{{when chrome}}"));
        assert!(render(Some("edit")).contains("メモ帳"));
        assert!(render(Some("map")).contains("<leader>"));
        assert!(render(Some("keys")).contains("J まとめ"));
        assert!(render(Some("keys")).contains("gp ピン"));
        assert!(render(None).contains("g. 再貼"));
        assert!(render(Some("search")).contains("Esc / Ctrl+[ で絞りを外す"));
        assert!(render(Some("search")).contains("Tab は一覧へ"));
        assert!(render(Some("search")).contains("x y でも当たる"));
        assert!(render(Some("tag")).contains("x y でも当たる"));
        assert!(render(Some("pin")).contains("gp でピン留め"));
        assert!(render(Some("nope")).starts_with("ない"));
    }
}


# Lesson 1-22: ファイルの中身を一覧で見よう！ (LSPドキュメントシンボル)

LSPの参照の検索機能ができるようになりましたね。素晴らしいです！

次に、言語サーバーの便利なナビゲーション機能の一つである「**ドキュメントシンボル (Document Symbols)**」について学びます。エディタがドキュメント内のシンボル（関数、変数、クラスなど）のリストを要求したときに、言語サーバーがどのようにそのリストを返すのか、その形式と作成方法を学びます。

## ドキュメントシンボル (Document Symbols) とは？

ドキュメントシンボル機能は、エディタのサイドバーなどに、現在開いているファイル内の関数や変数、クラスなどの一覧を表示する機能です。これにより、ファイル全体の構造を素早く把握したり、特定のシンボルにジャンプしたりすることができます。

エディタは、ユーザーがドキュメントシンボルを要求すると、言語サーバーに `textDocument/documentSymbol` リクエストを送ります。サーバーはこれに対して、シンボル情報のリスト（`DocumentSymbol` または `SymbolInformation` オブジェクトの配列）を返します。

## `lsp-types::DocumentSymbol` の構造

LSPのドキュメントシンボルは `lsp_types::DocumentSymbol` という構造体で表現されます。主なフィールドは以下の通りです。

*   `name`: シンボルの名前（例: `"main"`, `"myVariable"`）。
*   `kind`: シンボルの種類。`SymbolKind` Enum（`Function`, `Variable`, `Class`, `Method` など）を使います。
*   `range`: シンボルが定義されているコードの全体範囲。`Range` を使います。
*   `selection_range`: シンボルの名前部分の範囲。`Range` を使います。通常、`range` の一部です。
*   `children`: オプションのフィールドで、ネストされたシンボル（例: クラス内のメソッド）を表現します。

## コードの分析とシンボルの抽出

今回のレッスンでは、シンプルに「`fn `」で始まる行を関数定義とみなし、その関数名と位置を抽出します。

1.  `document_store` から `file_uri` に対応するファイルの内容（`String`）を取得します。
2.  ファイルの内容を1行ずつ処理します。
3.  各行が `"fn "` で始まるかチェックします。
4.  もし始まっていれば、その行から関数名を抽出します。
    *   `line.strip_prefix("fn ")` で `"fn "` を取り除き、残りの文字列から最初の `(` や空白までの部分を関数名とします。
5.  抽出した情報を使って `DocumentSymbol` オブジェクトを作成し、`Vec<DocumentSymbol>` に追加します。

**ヒント:**
*   `line.find('(')` を使って、関数名の終わりを見つけることができます。
*   `Range::new(Position::new(行番号, 開始列), Position::new(行番号, 終了列))` のように `Range` を構築します。

## やってみよう！

あなたの今回のミッションは、`get_document_symbols` 関数を完成させることです。

1.  `document_store` から `file_uri` に対応するファイルの内容を取得します。見つからなければ空の `Vec<DocumentSymbol>` を返します。
2.  `file_content` を1行ずつ処理し、行番号も取得します。
3.  各行が `"fn "` で始まるかチェックします。
4.  もし始まっていれば、関数名を抽出し、その情報を使って `DocumentSymbol` を作成します。
    *   `name`: 抽出した関数名。
    *   `kind`: `SymbolKind::FUNCTION`。
    *   `range`: 行全体の範囲（`Position::new(行番号, 0)` から行末まで）。
    *   `selection_range`: 関数名の範囲（`Position::new(行番号, "fn ".len())` から関数名の終わりまで）。
5.  見つかったすべての `DocumentSymbol` を `Vec<DocumentSymbol>` として返します。

`src/lessons/lesson_1_22.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

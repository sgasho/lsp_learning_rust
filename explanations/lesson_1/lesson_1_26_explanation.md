
# Lesson 1-26: コードの同じ単語を光らせよう！ (LSPハイライト)

LSPのリネーム機能ができるようになりましたね。素晴らしいです！

次に、言語サーバーの便利なナビゲーション機能の一つである「**ハイライト (Document Highlight)**」について学びます。エディタが特定のシンボルをハイライト表示するよう要求したときに、言語サーバーがどのようにそのシンボルの出現箇所を返すのか、その形式と作成方法を学びます。

## ハイライト (Document Highlight) とは？

ハイライト機能は、コード中の変数名や関数名などを選択したり、カーソルを合わせたりすると、そのシンボルが使われている他の場所も自動的にハイライト表示される機能です。これにより、コードのどこで同じシンボルが使われているかを視覚的に素早く把握できます。

エディタは、ユーザーがハイライトを要求すると、言語サーバーに `textDocument/documentHighlight` リクエストを送ります。サーバーはこれに対して、ハイライトする範囲のリスト（`DocumentHighlight` オブジェクトの配列）を返します。

## `lsp-types::DocumentHighlight` の構造

LSPのドキュメントハイライトは `lsp_types::DocumentHighlight` という構造体で表現されます。主なフィールドは以下の通りです。

*   `range`: ハイライトするコードの範囲。`Range` を使います。
*   `kind`: オプションのフィールドで、ハイライトの種類を示します。`DocumentHighlightKind` Enum（`Text`, `Read`, `Write`）を使います。今回はシンプルに `Text` を使います。

## コードの単語を特定し、すべての出現箇所をハイライトする

ホバー機能や定義へ移動機能、参照の検索機能と同様に、まずカーソル位置にある「単語」を特定する必要があります。Lesson 1-19で実装した単語抽出ロジックを再利用できます。

今回のレッスンでは、シンプルに「`my_variable`」という単語がカーソル位置にあれば、その単語がコード全体で出現する場所をすべて見つけ出し、`DocumentHighlight` を作成します。見つかったすべての場所を `DocumentHighlight` オブジェクトとしてリストにまとめます。

**ヒント:**
*   Lesson 1-21の `find_references` 関数で使ったロジックを参考に、`"my_variable"` のすべての出現箇所を検索します。
*   見つかった各出現箇所に対して `DocumentHighlight` オブジェクトを作成し、それらを `Vec<DocumentHighlight>` にまとめて返します。
    *   `DocumentHighlight` の `range` は、出現箇所の開始位置と終了位置（単語の長さ分）を使って `Range::new(Position::new(行番号, 列番号), Position::new(行番号, 列番号 + 単語の長さ))` のように構築します。
    *   `DocumentHighlight` の `kind` は `Some(DocumentHighlightKind::TEXT)` とします。

## やってみよう！

あなたの今回のミッションは、`get_document_highlights` 関数を完成させることです。

1.  `document_store` から `file_uri` に対応するファイルの内容を取得します。見つからなければ空の `Vec<DocumentHighlight>` を返します。
2.  `position.line` と `position.character` を使って、カーソル位置にある単語を特定します。Lesson 1-19で使った単語抽出ロジックを参考にしてください。
3.  特定した単語が `"my_variable"` であれば、その単語が `file_content` 全体で出現するすべての場所を検索します。
4.  見つかった各出現箇所に対して `DocumentHighlight` オブジェクトを作成し、それらを `Vec<DocumentHighlight>` にまとめて返します。
5.  それ以外の単語、または単語が特定できない場合は、空の `Vec<DocumentHighlight>` を返します。

`src/lessons/lesson_1_26.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

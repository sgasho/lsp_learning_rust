
# Lesson 1-23: コードを自動で直そう！ (LSPコードアクション)

LSPのドキュメントシンボル機能ができるようになりましたね。素晴らしいです！

次に、言語サーバーの非常に便利な機能の一つである「**コードアクション (Code Actions)**」について学びます。エディタが特定のコード範囲に対して可能なアクション（例: クイックフィックス、リファクタリング）を要求したときに、言語サーバーがどのようにそのリストを返すのか、その形式と作成方法を学びます。

## コードアクション (Code Actions) とは？

コードアクションは、エディタがコードの特定の場所（例えば、エラーや警告が表示されている行）で、ユーザーに「こんなことができますよ」と提案する機能です。例えば、以下のようなアクションがあります。

*   **クイックフィックス (Quick Fix)**: エラーや警告を自動的に修正する提案。
*   **リファクタリング (Refactor)**: コードの構造を変更する提案（変数名の変更、関数の抽出など）。
*   **ソースアクション (Source Action)**: ファイル全体に適用されるアクション（インポートの整理など）。

エディタは、ユーザーがコードアクションを要求すると、言語サーバーに `textDocument/codeAction` リクエストを送ります。サーバーはこれに対して、`CodeAction` オブジェクトのリストを返します。

## `lsp-types::CodeAction` の構造

LSPのコードアクションは `lsp_types::CodeAction` という構造体で表現されます。主なフィールドは以下の通りです。

*   `title`: コードアクションのタイトル（エディタに表示されるテキスト）。
*   `kind`: コードアクションの種類。`CodeActionKind` Enum（`QuickFix`, `Refactor`, `Source` など）を使います。
*   `edit`: オプションのフィールドで、このアクションが適用されたときにコードに加える変更（`WorkspaceEdit` 型）。
*   `command`: オプションのフィールドで、このアクションが適用されたときに実行されるコマンド。

今回は、`edit` フィールドを使って、コードの変更を提案します。

## `WorkspaceEdit` と `TextEdit`

`WorkspaceEdit` は、複数のファイルにわたる変更を表現するための構造体です。今回は1つのファイルに対する変更なので、`changes` フィールドを使います。

*   `WorkspaceEdit`: `changes` (変更対象のURIと `TextEdit` のリストのマップ) を持ちます。
    *   `changes`: `HashMap<Url, Vec<TextEdit>>` の形式です。
*   `TextEdit`: 特定の範囲のテキストを新しいテキストに置き換えるための構造体です。
    *   `range`: 変更するコードの範囲。
    *   `new_text`: 置き換える新しいテキスト。

今回のミッションでは、「TODO」診断に対して、その診断の範囲を空の文字列で置き換える `TextEdit` を作成します。これにより、「TODO」がコードから削除されることになります。

## やってみよう！

あなたの今回のミッションは、`get_code_actions` 関数を完成させることです。

1.  入力された `diagnostics` のリストをループします。
2.  各 `Diagnostic` の `message` が `"Found a TODO item."` であるかチェックします。
3.  もしそうであれば、その診断に対応する `CodeAction` を作成します。
    *   `title`: `"Remove TODO item"`
    *   `kind`: `CodeActionKind::QUICKFIX`
    *   `edit`: `WorkspaceEdit` を作成し、その `changes` フィールドに `HashMap<Url, Vec<TextEdit>>` を設定します。
        *   `HashMap` のキーは `file_uri` です。
        *   `Vec<TextEdit>` には、`TextEdit` オブジェクトを1つ追加します。
            *   `TextEdit` の `range` は、診断の `range` をそのまま使います。
            *   `TextEdit` の `new_text` は、空の文字列 `""` です。
4.  作成した `CodeAction` を `Vec<CodeAction>` に追加して返します。

`src/lessons/lesson_1_23.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

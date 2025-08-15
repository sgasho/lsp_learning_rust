
# Lesson 1-13: コードの間違いを見つけよう！ (LSP診断)

LSPの基本的なメッセージの送受信とライフサイクル管理ができるようになりましたね。素晴らしいです！

ここからは、言語サーバーが実際にコードを分析し、エディタに情報を提供する機能の基礎となる「**診断 (Diagnostics)**」について学びます。

## 診断 (Diagnostics) とは？

診断とは、コードの中にある「**問題点**」をエディタに伝えるためのメッセージです。例えば、文法エラー、型エラー、未使用の変数、推奨されない書き方（警告）など、様々な種類の問題があります。

エディタは、言語サーバーから送られてきた診断情報を使って、コードの該当箇所に波線を表示したり、問題リストを表示したりして、開発者に問題を知らせます。これにより、開発者はコードを修正しやすくなります。

## `lsp-types::Diagnostic` の構造

LSPの診断メッセージは `lsp_types::Diagnostic` という構造体で表現されます。主なフィールドは以下の通りです。

*   `range`: 問題が見つかったコードの範囲（行と列）。`lsp_types::Range` と `lsp_types::Position` を使います。
*   `severity`: 問題の深刻度。`DiagnosticSeverity` Enum（`Error`, `Warning`, `Information`, `Hint`）を使います。
*   `message`: 問題の内容を説明するテキスト。
*   `source`: 診断を生成したツールやソース（例: `"rust-analyzer"`, `"eslint"`）。

## コードの分析と診断の生成

今回のレッスンでは、コードの文字列（`file_content`）を1行ずつ読み込み、その中に特定のキーワード（例: `"TODO"`）が含まれているかをチェックします。

Rustで文字列を1行ずつ処理するには、`lines()` メソッドが便利です。また、文字列が特定のパターンを含むかチェックするには `contains()` メソッドを使います。大文字・小文字を区別しない検索をする場合は、両方の文字列を小文字（または大文字）に変換してから比較すると良いでしょう（例: `line.to_lowercase().contains("todo")`）。

診断を生成する際には、`Diagnostic::new()` コンストラクタを使います。`Range` は `Range::new(Position::new(line_number, 0), Position::new(line_number, 0))` のように、行の先頭を指定すれば十分です。

## やってみよう！

あなたの今回のミッションは、`generate_diagnostics` 関数を完成させることです。

1.  `file_content` を1行ずつ処理します。
2.  各行に `"TODO"` (大文字・小文字を区別しない) が含まれているかチェックします。
3.  もし含まれていれば、その行に対して `DiagnosticSeverity::Warning` の診断メッセージを生成します。
    *   `range`: その行の先頭（`Position::new(行番号, 0)`）を指定します。
    *   `message`: `"Found a TODO item."`
    *   `source`: `"toy-lang-server"`
4.  見つかったすべての診断メッセージを `Vec<Diagnostic>` として返します。

`src/lessons/lesson_1_13.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

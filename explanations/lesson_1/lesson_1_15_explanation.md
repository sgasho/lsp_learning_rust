
# Lesson 1-15: コードを自動で完成させよう！ (LSPコード補完)

LSPの診断メッセージを生成し、エディタに公開できるようになりましたね。素晴らしいです！

次に、言語サーバーの最も便利でよく使われる機能の一つである「**コード補完 (Completion)**」について学びます。エディタがコード補完を要求したときに、言語サーバーがどのように候補を返すのか、その形式と作成方法を学びます。

## コード補完 (Completion) とは？

コード補完は、コードを書いている途中で、エディタが自動的に「もしかしてこれを書きたいですか？」と候補を提示してくれる機能です。これにより、タイピングの手間が省け、スペルミスを防ぎ、APIを覚えなくてもコードを書けるようになります。

エディタは、ユーザーが特定の文字を入力したり、補完をトリガーするキー（例: `.` や `Ctrl+Space`）を押したりすると、言語サーバーに `textDocument/completion` リクエストを送ります。サーバーはこれに対して、補完候補のリストを返します。

## `lsp-types::CompletionItem` の構造

LSPの補完候補は `lsp_types::CompletionItem` という構造体で表現されます。主なフィールドは以下の通りです。

*   `label`: 補完候補として表示されるテキスト（例: `"fn"`, `"println!"`）。
*   `kind`: 補完候補の種類。`CompletionItemKind` Enum（`Function`, `Variable`, `Keyword`, `Struct` など）を使います。これにより、エディタは候補の横に適切なアイコンを表示できます。
*   `detail`: 補完候補に関する追加情報（例: 関数のシグネチャ、変数の型）。
*   `documentation`: 補完候補に関する詳細な説明（例: 関数のドキュメント）。

`CompletionItem` には他にもたくさんのフィールドがありますが、今回は `label` と `kind` だけを使います。残りのフィールドは `..Default::default()` を使ってデフォルト値で埋めることができます。

## やってみよう！

あなたの今回のミッションは、`generate_completions` 関数を完成させることです。

このレッスンでは、`file_content` や `position` の値に関わらず、常に以下の3つのキーワードを補完候補として返します。

*   `"fn"` (種類: `CompletionItemKind::KEYWORD`)
*   `"let"` (種類: `CompletionItemKind::KEYWORD`)
*   `"struct"` (種類: `CompletionItemKind::KEYWORD`)

`src/lessons/lesson_1_15.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

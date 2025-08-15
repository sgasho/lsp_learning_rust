
# Lesson 1-17: ファイルが変更されたことを知ろう！ (textDocument/didChange)

ファイルが開かれたことを知る `textDocument/didOpen` 通知を処理できるようになりましたね。素晴らしいです！

しかし、エディタでコードを編集すると、ファイルの内容は常に変わります。言語サーバーが常に最新のコードを分析し、リアルタイムで診断や補完などの機能を提供するためには、ファイルの内容が変更されたときに送られてくる `textDocument/didChange` 通知を処理する必要があります。

## `textDocument/didChange` 通知とは？

ユーザーがエディタでファイルの内容を変更するたびに、エディタから言語サーバーに送られる通知です。この通知には、変更されたファイルのURIと、変更後のファイル全体の新しい内容が含まれています（LSPには差分を送る方法もありますが、今回はシンプルにファイル全体を送る形式を扱います）。

サーバーはこれを受け取ると、自分の「ドキュメントストア」に保存している該当ファイルの古い内容を、新しい内容で更新します。そして、更新された内容に対して再度診断を生成し、エディタに送り返すことで、リアルタイムなフィードバックを提供します。

## `params` の構造

`textDocument/didChange` 通知の `params` フィールドは、以下のような構造を持つオブジェクトです。

```json
{
  "textDocument": {
    "uri": "file:///path/to/file.rs",
    "version": 2 // ファイルのバージョン番号
  },
  "contentChanges": [
    {
      "text": "fn main() {\n  // 新しい内容\n}\n" // 変更後のファイル全体の新しい内容
    }
  ]
}
```

ここから `uri` と `contentChanges` の中の `text` の値を取り出す必要があります。

## RustでのJSON値の抽出と `HashMap` の更新

`params_value` から `textDocument` オブジェクト、そして `uri` を取り出す方法は `didOpen` と似ています。`contentChanges` は配列なので、`as_array()` を使って配列として取得し、その最初の要素（オブジェクト）から `text` を取り出します。

`HashMap` の値を更新するには、`insert` メソッドを再度呼び出すだけです。同じキーで `insert` すると、古い値は新しい値で上書きされます。

```rust
use std::collections::HashMap;
use lsp_types::Url;

let mut document_store: HashMap<Url, String> = HashMap::new();
let uri = Url::parse("file:///a.rs").unwrap();

// 既存の値を更新
document_store.insert(uri.clone(), "古い内容".to_string());
document_store.insert(uri.clone(), "新しい内容".to_string());

// 保存した内容を取り出すと「新しい内容」になっている
if let Some(content) = document_store.get(&uri) {
    println!("Updated content: {}", content);
}
```

## やってみよう！

あなたの今回のミッションは、`handle_did_change_notification` 関数を完成させることです。

1.  `params_value` から `textDocument` オブジェクトと `contentChanges` 配列を取得します。
2.  `textDocument` オブジェクトから `uri` を取得し、`Url` 型にパースします。
3.  `contentChanges` 配列の最初の要素から `text` （新しいファイル内容）を取得します。
4.  パースした `Url` をキー、新しい `text` を値として `document_store` の内容を更新します。
5.  更新した `text` の内容に対して、Lesson 1-13 で作成した `generate_diagnostics` 関数を使って診断を生成し、その結果を `Vec<Diagnostic>` として返します。
6.  もし `params_value` のパースに失敗したり、必要なフィールドが欠けていたり、URIが無効だったりした場合は、空の `Vec<Diagnostic>` を返します。

`src/lessons/lesson_1_17.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！


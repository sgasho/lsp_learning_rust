
# Lesson 1-8: 命令の詳細を理解しよう！ (LSPパラメータ)

LSPメッセージの「メソッド」を理解できるようになりましたね。素晴らしいです！

しかし、メソッドだけでは、エディタが何をしたいのか、具体的な情報が足りません。例えば、「ファイルを開く」というメソッドだけでは、どのファイルを開くのか分かりませんよね？

そこで登場するのが「**パラメータ (params)**」フィールドです。これは、メソッドが実行されるために必要な「**詳細情報**」や「**引数**」を格納する場所です。

## パラメータ (params) とは？

`params` フィールドは、LSPメッセージの `method` に応じて、様々な形をとります。JSONのオブジェクト（`{}`）だったり、配列（`[]`）だったり、時には単なる文字列や数値、`null` のこともあります。

例えば、`textDocument/didOpen` 通知では、開かれたファイルのURIや内容が `params` オブジェクトの中に含まれます。

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {  <-- ここがparams！
    "textDocument": {
      "uri": "file:///path/to/file.rs",
      "languageId": "rust",
      "version": 1,
      "text": "fn main() {}\n"
    }
  }
}
```

`workspace/applyEdit` リクエストでは、適用する変更のリストが `params` 配列の中に含まれることがあります。

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "workspace/applyEdit",
  "params": [  <-- ここがparams！
    {
      "changes": {
        "file:///path/to/file.rs": [
          { "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } }, "newText": "use std::io;\n" }
        ]
      }
    }
  ]
}
```

## `serde_json::Value` からパラメータを安全に抽出する

`params` フィールドは `serde_json::Value` 型としてパースされたJSONコンテンツの中にあります。これを抽出するには、`get()` メソッドを使います。

`get()` メソッドは `Option<&Value>` を返すので、`params` フィールドが存在しない場合は `None` が返されます。存在する場合は `Some(&Value)` が返され、その `&Value` が `params` の内容になります。

今回は、`params` の中身の型を特定する必要はありません。ただ `params` フィールドが存在すれば、その `Value` をそのまま返すだけでOKです。

## やってみよう！

あなたの今回のミッションは、`get_lsp_params` 関数を完成させることです。

1.  入力された `json_value` から `"params"` フィールドを取得します。
2.  もし `"params"` フィールドが存在すれば、その `Value` を `Some(Value)` として返します。
3.  それ以外の場合は `None` を返します。

`src/lessons/lesson_1_8.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！


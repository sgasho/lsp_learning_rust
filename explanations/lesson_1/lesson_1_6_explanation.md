

# Lesson 1-6: 質問と報告の違いを理解しよう！ (リクエストと通知)

LSPメッセージのパースができるようになりましたね。素晴らしいです！

LSPでは、エディタとサーバーの間でやり取りされるメッセージには、大きく分けて2つの種類があります。それが「**リクエスト (Request)**」と「**通知 (Notification)**」です。

## リクエスト (Request): 質問と回答

リクエストは、エディタがサーバーに「**質問**」をするようなものです。質問なので、サーバーは必ず「**回答**」を返さなければなりません。

例えば、「この関数の定義はどこにありますか？」という質問（`textDocument/definition` リクエスト）に対して、サーバーは「このファイルのこの行にありますよ」という回答を返します。

リクエストには、必ず「`id`」という番号（または文字列）がついています。これは、どの質問に対する回答なのかを区別するための「質問番号」のようなものです。エディタは、この `id` を使って、どの質問の回答が返ってきたのかを判断します。

JSONの例:
```json
{
  "jsonrpc": "2.0",
  "id": 1,             <-- これが質問番号！
  "method": "textDocument/definition",
  "params": {
    "textDocument": { "uri": "file:///path/to/file.rs" },
    "position": { "line": 10, "character": 5 }
  }
}
```

## 通知 (Notification): 一方的な報告

通知は、エディタがサーバーに「**報告**」をするようなものです。報告なので、サーバーは特に回答を返す必要はありません。

例えば、「このファイルを開きましたよ」という報告（`textDocument/didOpen` 通知）に対して、サーバーは「分かりました」と心の中で思うだけで、特に返事をすることはありません。

通知には `id` フィールドがありません。これがリクエストとの一番大きな違いです。

JSONの例:
```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {
    "textDocument": { "uri": "file:///path/to/file.rs" },
    "text": "fn main() {}\n"
  }
}
```

## `serde_json::Value` の操作

`serde_json::Value` は、JSONの様々な型を表現できるEnumです。特定のフィールドが存在するかどうか、そのフィールドが期待する型かどうかをチェックするには、`as_object()`, `get()`, `is_string()`, `is_number()` などのメソッドが便利です。

例えば、`id` フィールドが存在し、それが数値か文字列であるかをチェックするには、以下のように書けます。

```rust
if let Some(id_value) = json_value.get("id") {
    if id_value.is_string() || id_value.is_number() {
        // id_value は有効なID
    }
}
```

## やってみよう！

あなたの今回のミッションは、`is_request_and_get_id` 関数を完成させることです。

1.  入力された `json_value` が、LSPメッセージの基本構造（`jsonrpc: "2.0"`, `method: "..."`）を持っているか確認します。
2.  もし `id` フィールドが存在し、それが数値または文字列であれば、それはリクエストです。その `id` の `Value` を `Some(id_value)` として返します。
3.  `id` フィールドが存在しない、または `jsonrpc` や `method` がない場合は、通知または不正なメッセージなので `None` を返します。

`src/lessons/lesson_1_6.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

```
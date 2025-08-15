
# Lesson 1-9: 届いた手紙を完全に理解しよう！ (LSPメッセージの統合解析)

これまでのレッスンで、LSPメッセージの各部分（ヘッダー、コンテンツ、ID、メソッド、パラメータ）を個別に解析する方法を学びました。素晴らしいです！

今回は、これらの知識をすべて組み合わせて、受信したLSPメッセージ全体を、より意味のある構造に解析する練習をします。

## なぜ統合解析が必要なの？

言語サーバーは、エディタから様々な種類のLSPメッセージを受け取ります。それぞれのメッセージは、異なる目的と構造を持っています。例えば、`initialize` リクエストはサーバーの初期化に使われ、`textDocument/didOpen` 通知はファイルが開かれたことを伝えます。

これらのメッセージを適切に処理するためには、まずメッセージ全体を解析し、それがどのような種類のメッセージで、どのような情報を含んでいるのかを正確に把握する必要があります。この統合解析のステップが、言語サーバーの「脳」の入り口となります。

## これまでの知識を組み合わせる

`parse_full_lsp_message` 関数では、これまでのレッスンで作成した以下の関数を順番に呼び出し、結果を組み合わせていきます。

1.  **`super::lesson_1_2::parse_lsp_message`**: メッセージをヘッダーとコンテンツに分割します。
2.  **`super::lesson_1_4::parse_json_content`**: コンテンツを `serde_json::Value` にパースします。
3.  **`super::lesson_1_6::is_request_and_get_id`**: メッセージがリクエストかどうかを判断し、リクエストであれば `id` を取得します。
4.  **`super::lesson_1_7::get_lsp_method`**: メッセージの `method` を取得します。
5.  **`super::lesson_1_8::get_lsp_params`**: メッセージの `params` を取得します。

これらの関数はすべて `Option` を返すため、`?` 演算子や `and_then` を活用して、途中でエラーが発生した場合は `None` を返すようにすると、コードが簡潔になります。

## `LspMessage` Enumの定義

今回は、解析結果を `LspMessage` というEnumで表現します。これにより、リクエストと通知で異なる情報を保持しつつ、統一的に扱うことができます。

```rust
pub enum LspMessage {
    Request {
        id: Value,
        method: String,
        params: Option<Value>,
    },
    Notification {
        method: String,
        params: Option<Value>,
    },
}
```

## やってみよう！

あなたの今回のミッションは、`parse_full_lsp_message` 関数を完成させることです。

これまでのレッスンで学んだ関数を適切に呼び出し、`LspMessage` Enumの適切なバリアント（`Request` または `Notification`）を構築して返してください。

`src/lessons/lesson_1_9.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！
